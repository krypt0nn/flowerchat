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

use anyhow::Context;
use clap::{Parser, Subcommand};
use tokio::runtime::Handle;

use libflowerpot::crypto::*;
use libflowerpot::block::{Block, BlockContent};
use libflowerpot::transaction::Transaction;
use libflowerpot::storage::Storage;
use libflowerpot::storage::file_storage::FileStorage;

pub mod consts;
pub mod utils;
pub mod database;
pub mod identities;
pub mod client;
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
        }

        Ok(())
    }
}

#[tokio::main(flavor = "multi_thread")]
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
