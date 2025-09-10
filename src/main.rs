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

use std::io::{Read, Write};

use anyhow::Context;
use clap::{Parser, Subcommand};

use libflowerpot::crypto::*;

pub mod consts;
pub mod utils;
pub mod database;
pub mod events;
pub mod identities;
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
    }
}

impl Command {
    #[inline]
    pub async fn run(self) -> anyhow::Result<()> {
        match self {
            Self::Keypair { command } => command.run().await
        }
    }
}

#[derive(Subcommand)]
enum KeypairCommand {
    /// Create new random secret key.
    Create,

    /// Export public key from the provided secret key.
    ///
    /// If `secret` argument is not specified then stdin value will be used
    /// as input. Returns no value if secret key is not provided at all.
    Export {
        secret: Option<String>
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

            Self::Export { secret } => {
                let secret_key = match secret {
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

            let result = tui::login::render(database, &mut terminal).await;

            ratatui::restore();

            result?;

            Ok(())
        }
    }
}
