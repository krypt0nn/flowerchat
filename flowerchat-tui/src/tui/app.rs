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

use std::sync::Arc;
use spin::RwLock;

use tokio::runtime::Handle;
use tokio::task::JoinHandle;
use tokio::sync::mpsc::{UnboundedSender, UnboundedReceiver, unbounded_channel};
use tokio::sync::oneshot::Sender;

use libflowerpot::crypto::*;
use libflowerpot::viewer::Viewer;

use crate::database::Database;
use crate::database::space::SpaceRecord;
use crate::client::Update;

use crate::tui::terminal_widget::{TerminalWidget, TerminalWidgetCurrentLine};

#[allow(clippy::large_enum_variant)]
pub enum Action {
    /// Print text to the terminal widget.
    TerminalPush(String),

    /// Set current output line in the terminal widget.
    TerminalSetCurrentLine(String),

    /// Request space record from provided input query.
    RequestSpaceRecord(String, Sender<anyhow::Result<SpaceRecord>>),

    /// Connect to the space.
    Connect(SpaceRecord, SecretKey, Viewer)
}

#[derive(Debug)]
pub struct SpaceConnection {
    pub task: JoinHandle<anyhow::Result<()>>,
    pub space: SpaceRecord,
    pub identity: SecretKey
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub terminal_widget: Arc<RwLock<TerminalWidget>>,
    pub database: Database,
    pub connection: Arc<RwLock<Option<SpaceConnection>>>
}

impl AppState {
    pub fn new(database: Database) -> Self {
        Self {
            terminal_widget: Arc::new(RwLock::new(TerminalWidget::default())),
            database,
            connection: Arc::new(RwLock::new(None))
        }
    }
}

pub fn run_actions_handler(
    runtime: Handle,
    state: AppState
) -> (UnboundedSender<Action>, UnboundedReceiver<()>) {
    let (action_sender, mut action_receiver) = unbounded_channel();
    let (updates_sender, updates_receiver) = unbounded_channel();

    runtime.clone().spawn({
        let action_sender = action_sender.clone();

        async move {
            while let Some(action) = action_receiver.recv().await {
                match action {
                    Action::TerminalPush(text) => {
                        state.terminal_widget.write().push(text);

                        let _ = updates_sender.send(());
                    }

                    Action::TerminalSetCurrentLine(text) => {
                        state.terminal_widget.write().ongoing = TerminalWidgetCurrentLine::Output(text);

                        let _ = updates_sender.send(());
                    }

                    Action::RequestSpaceRecord(space, sender) => {
                        let space = match space.parse::<i64>() {
                            Ok(space_id) => {
                                SpaceRecord::open(state.database.clone(), space_id)
                                    .map_err(|err| {
                                        anyhow::anyhow!(err)
                                            .context("failed to open space record")
                                    })
                            }

                            Err(_) => match Hash::from_base64(space) {
                                Some(space_hash) => {
                                    match SpaceRecord::find(state.database.clone(), &space_hash) {
                                        Ok(Some(record)) => Ok(record),
                                        Ok(None) => Err(anyhow::anyhow!("there's no space record with such root block hash")),
                                        Err(err) => Err(anyhow::anyhow!(err).context("failed to find space record"))
                                    }
                                }

                                None => Err(anyhow::anyhow!("invalid space root block hash format"))
                            }
                        };

                        let _ = sender.send(space);
                    }

                    Action::Connect(space, identity, viewer) => {
                        let mut lock = state.connection.write();

                        // Destroy previous connection.
                        if let Some(prev_connection) = &*lock {
                            prev_connection.task.abort();
                        }

                        // Spawn new connection.
                        let (sender, mut receiver) = unbounded_channel();

                        let mut sender = Some(sender);

                        let task = runtime.spawn(crate::client::run(
                            state.database.clone(),
                            viewer,
                            move |update| {
                                match update {
                                    Update::Verification {
                                        block_hash,
                                        transaction_hash,
                                        block_timestamp,
                                        estimated_progress
                                    } => {
                                        if let Some(sender) = &sender {
                                            let _ = sender.send((
                                                block_hash,
                                                transaction_hash,
                                                block_timestamp,
                                                estimated_progress
                                            ));
                                        }
                                    }

                                    Update::VerificationDone => sender = None,

                                    Update::NewEvent {
                                        block_hash: _,
                                        transaction_hash: _,
                                        block_timestamp: _
                                    } => {
                                        sender = None;
                                    }
                                }
                            }
                        ));

                        lock.replace(SpaceConnection {
                            task,
                            space,
                            identity
                        });

                        // FIXME: when there's no blocks in space

                        let mut i = 0u64;

                        while let Some(update) = receiver.recv().await {
                            let (
                                block_hash,
                                transaction_hash,
                                _block_timestamp,
                                estimated_progress
                            ) = update;

                            i += 1;

                            let line = format!(
                                "connect: [{i:6}] verified tr {}, block {}",
                                transaction_hash.to_base64(),
                                block_hash.to_base64()
                            );

                            let _ = action_sender.send(Action::TerminalPush(line));

                            let width = state.terminal_widget.read().width as usize - 20;
                            let offset = (estimated_progress * width as f32).round() as usize;

                            let line = format!(
                                "connect: |{}{}| {:.2}%",
                                "#".repeat(offset),
                                " ".repeat(width - offset),
                                estimated_progress
                            );

                            let _ = action_sender.send(Action::TerminalSetCurrentLine(line));

                            let _ = updates_sender.send(());
                        }
                    }
                }
            }
        }
    });

    (action_sender, updates_receiver)
}
