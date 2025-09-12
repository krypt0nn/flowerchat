// SPDX-License-Identifier: GPL-3.0-or-later
//
// flowerchat
// Copyright (C) 2025  Nikita Podvirnyi <krypt0nn@vk.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

/// `flowerchat-tui` app version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

use std::io::{Read, Write};
use std::path::PathBuf;
use std::net::{SocketAddr, Ipv6Addr};

use anyhow::Context;
use clap::{Parser, Subcommand};
use tokio::runtime::{Runtime, Handle};

use libflowerpot::crypto::*;
use libflowerpot::block::{Block, BlockContent};
use libflowerpot::transaction::Transaction;
use libflowerpot::storage::Storage;
use libflowerpot::storage::file_storage::FileStorage;
use libflowerpot::client::Client;
use libflowerpot::pool::ShardsPool;
use libflowerpot::security::SecurityRules;
use libflowerpot::shard::{Shard, ShardSettings, serve as serve_shard};

pub mod consts;
pub mod utils;
pub mod database;
pub mod identities;
pub mod client;
pub mod validator;
pub mod tui;

#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>
}

#[derive(Subcommand)]
enum Command {
    /// Keys management tools.
    Keypair {
        #[command(subcommand)]
        command: KeypairCommand
    },

    /// Spaces management tools.
    Space {
        #[command(subcommand)]
        command: SpaceCommand
    }
}

impl Command {
    #[inline]
    pub async fn run(self) -> anyhow::Result<()> {
        match self {
            Self::Keypair { command } => command.run().await,
            Self::Space { command } => command.run().await
        }
    }
}

#[derive(Subcommand)]
enum KeypairCommand {
    /// Create new random secret key.
    Create,

    /// Export public key from the provided secret key.
    ///
    /// If `secret_key` argument is not specified then stdin value will be used
    /// as input. Returns no value if secret key is not provided at all.
    Export {
        #[arg(short, long)]
        secret_key: Option<String>
    }
}

impl KeypairCommand {
    #[inline]
    pub async fn run(self) -> anyhow::Result<()> {
        match self {
            Self::Create => {
                let secret_key = SecretKey::random(&mut utils::get_rng());

                let mut stdout = std::io::stdout();

                stdout.write_all(secret_key.to_base64().as_bytes())?;
                stdout.flush()?;
            }

            Self::Export { secret_key } => {
                let secret_key = match secret_key {
                    Some(secret_key) => secret_key.as_bytes().to_vec(),
                    None => {
                        let mut secret_key = Vec::new();

                        let mut stdin = std::io::stdin();

                        stdin.read_to_end(&mut secret_key)?;

                        if secret_key.is_empty() {
                            return Ok(());
                        }

                        secret_key
                    }
                };

                let secret_key = SecretKey::from_base64(secret_key)
                    .ok_or_else(|| anyhow::anyhow!("failed to decode secret key"))?;

                let public_key = secret_key.public_key();

                let mut stdout = std::io::stdout();

                stdout.write_all(public_key.to_base64().as_bytes())?;
                stdout.flush()?;
            }
        }

        Ok(())
    }
}

#[derive(Subcommand)]
enum SpaceCommand {
    /// Create new space.
    Create {
        /// Path to the folder where the space's blockchain should be stored.
        #[arg(short, long)]
        path: PathBuf,

        /// Secret key of the space's blockchain creator. If unset then will be
        /// generated randomly.
        #[arg(short, long)]
        secret_key: Option<String>
    },

    /// Serve space to other nodes (start blockchain shard).
    Serve {
        /// Path to the blockchain folder.
        #[arg(short, long)]
        path: PathBuf,

        /// Another shard node to connect with.
        #[arg(short, long = "shard")]
        shards: Vec<String>,

        /// Local address of the shard.
        #[arg(
            short, long,
            default_value_t = SocketAddr::new(
                Ipv6Addr::UNSPECIFIED.into(),
                47901
            )
        )]
        local_address: SocketAddr,

        /// Remote address which can be accessible by other shards. If provided,
        /// it will be shared with them at bootstrap stage to improve network
        /// sparsity.
        #[arg(short, long)]
        remote_address: Option<String>,

        /// Maximal amount of active shards.
        #[arg(long, default_value_t = 16)]
        max_active_shards: usize,

        /// Maximal amount of inactive shards.
        #[arg(long, default_value_t = 1024)]
        max_inactive_shards: usize
    }
}

impl SpaceCommand {
    #[inline]
    pub async fn run(self) -> anyhow::Result<()> {
        match self {
            Self::Create { path, secret_key } => {
                let secret_key = match secret_key {
                    Some(secret_key) => SecretKey::from_base64(secret_key)
                        .ok_or_else(|| anyhow::anyhow!("invalid secret key format"))?,
                    None => SecretKey::random(&mut utils::get_rng())
                };

                if !path.exists() {
                    std::fs::create_dir_all(&path)?;
                } else {
                    anyhow::bail!("path is already occupied: {path:?}");
                }

                let storage = FileStorage::open(path)
                    .context("failed to create file storage for the blockchain")?;

                let block = BlockContent::transactions::<Transaction>([]);

                let block = Block::new(&secret_key, Hash::default(), block)
                    .context("failed to sign root block of the blockchain")?;

                let block_hash = block.hash()
                    .context("failed to calculate root block hash")?;

                storage.write_block(&block)
                    .context("failed to write root block to the blockchain")?;

                println!("Space created!");
                println!("  Root block: {}", block_hash.to_base64());
                println!("  Public key: {}", secret_key.public_key().to_base64());
                println!("  Secret key: {}", secret_key.to_base64());
            }

            Self::Serve {
                path,
                shards,
                local_address,
                remote_address,
                max_active_shards,
                max_inactive_shards
            } => {
                let mut stdout = std::io::stdout();

                if !path.exists() {
                    anyhow::bail!("blockchain folder doesn't exist");
                }

                let storage = FileStorage::open(path)
                    .context("failed to open blockchain storage")?;

                let root_block = storage.root_block()
                    .context("failed to get root block from the storage")?;

                let Some(root_block) = root_block else {
                    anyhow::bail!("root block is not stored");
                };

                let root_block = storage.read_block(&root_block)
                    .context("failed to read root block from the storage")?;

                let Some(root_block) = root_block else {
                    anyhow::bail!("root block is not stored");
                };

                let (is_valid, root_block_hash, public_key) = root_block.verify()
                    .context("failed to verify root block of the blockchain")?;

                if !is_valid {
                    anyhow::bail!("root block is invalid");
                }

                let client = Client::default();
                let mut pool = ShardsPool::default();

                pool.with_max_active(max_active_shards)
                    .with_max_inactive(max_inactive_shards)
                    .add_shards(shards);

                stdout.write_all(b"Bootstrapping shards pool...")?;
                stdout.flush()?;

                pool.update(&client).await;

                stdout.write_all(format!(
                    " {} active, {} inactive\n",
                    pool.active().count(),
                    pool.inactive().count()
                ).as_bytes())?;

                stdout.flush()?;

                // TODO
                // if let Some(remote_address) = &remote_address {
                //     stdout.write_all(b"Sharing remote address to network shards...")?;
                //     stdout.flush()?;

                //     for address in pool.active() {
                //         todo!();
                //     }
                // }

                stdout.write_all(format!(
                    "Shard started at {local_address}\n"
                ).as_bytes())?;

                stdout.write_all(format!(
                    "  Root block: {}\n",
                    root_block_hash.to_base64()
                ).as_bytes())?;

                stdout.write_all(format!(
                    "  Public key: {}\n",
                    public_key.to_base64()
                ).as_bytes())?;

                stdout.flush()?;

                let runtime = Runtime::new()
                    .context("failed to create tokio runtime")?;

                let handle = runtime.handle().to_owned();

                serve_shard(Shard {
                    client,
                    shards: pool,
                    local_address,
                    remote_address,
                    storage,
                    security_rules: SecurityRules {
                        ..SecurityRules::default()
                    },
                    settings: ShardSettings::default()
                }, handle).await?;
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    std::fs::create_dir_all(consts::DATA_FOLDER.as_path())
        .map_err(|err| {
            anyhow::anyhow!(err)
                .context("failed to create flowerchat data folder")
        })?;

    match Cli::parse().command {
        Some(command) => command.run().await,
        None => {
            let database = database::Database::open(
                consts::DATABASE_PATH.as_path()
            ).context("failed to open flowerchat database")?;

            let mut terminal = ratatui::init();

            let result = tui::render(
                Handle::current(),
                database,
                &mut terminal
            ).await;

            ratatui::restore();

            result?;

            Ok(())
        }
    }
}
