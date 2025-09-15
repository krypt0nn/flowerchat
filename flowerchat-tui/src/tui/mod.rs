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

use std::io::Stdout;

use tokio::runtime::Handle;
use tokio::task::JoinHandle;

use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::{self, Event, KeyCode};

use ratatui::layout::*;
use ratatui::widgets::*;
use ratatui::text::*;
use ratatui::style::*;

use crate::database::Database;

pub mod terminal_widget;
pub mod commands;
pub mod app;

use terminal_widget::*;

const FLOWERCHAT_LOGO: &str = r#"
  __ _                            _           _
 / _| |                          | |         | |
| |_| | _____      _____ _ __ ___| |__   __ _| |_
|  _| |/ _ \ \ /\ / / _ \ '__/ __| '_ \ / _` | __|
| | | | (_) \ V  V /  __/ | | (__| | | | (_| | |_
|_| |_|\___/ \_/\_/ \___|_|  \___|_| |_|\__,_|\__|
"#;

/// Run the flowerchat app.
///
/// This function handles keyboard / other user inputs and draws the TUI. The
/// events processing logic happens in async task spawned from the `app` mod.
///
/// Commands ran by user are spawned as separate async tasks. They can send
/// actions to the spawned background processor to e.g. print some text to the
/// terminal.
///
/// The state between this function and the background processor is shared using
/// locks. Both of these functions can edit it right now, and preferably this
/// should be changed in future (TODO).
pub async fn run_app(
    runtime: Handle,
    database: Database,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>
) -> anyhow::Result<()> {
    let state = app::AppState::new(database);

    let (actions_sender, mut updates_receiver) = app::run_actions_handler(
        runtime.clone(),
        state.clone()
    );

    let mut lock = state.terminal_widget.write();

    lock.push(FLOWERCHAT_LOGO.trim_matches('\n'));
    lock.push("\n");
    lock.push(format!("Flowerchat v{}", crate::VERSION));
    lock.push(format!("  flowerchat-protocol v{}", flowerchat_protocol::CRATE_VERSION));
    lock.push(format!("  protocol version: {}", flowerchat_protocol::PROTOCOL_VERSION));
    lock.push("\n");

    drop(lock);

    let mut running_command: Option<JoinHandle<()>> = None;
    let mut force_render = true;

    loop {
        if force_render {
            force_render = false;
        }

        else {
            while let Some(handle) = &running_command {
                if handle.is_finished() {
                    let mut terminal_widget = state.terminal_widget.write();

                    terminal_widget.push("\n");
                    terminal_widget.allow_user_input();

                    drop(terminal_widget);

                    running_command = None;
                }

                else if updates_receiver.try_recv().is_ok() {
                    break;
                }

                else {
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                }
            }
        }

        terminal.draw(|frame| {
            let block = Block::bordered();

            let terminal_area = match &*state.connection.read() {
                // Render connected chat.
                Some(_connection) => {
                    let [public_rooms_area, terminal_area] = Layout::horizontal([
                        Constraint::Percentage(20),
                        Constraint::Percentage(80)
                    ]).areas(frame.area());

                    let terminal_inner_area = block.inner(terminal_area);

                    frame.render_widget(
                        block.title_top("Terminal"), // TODO: space info
                        terminal_area
                    );

                    frame.render_widget(
                        Block::bordered().title_top("Public rooms"),
                        public_rooms_area
                    );

                    terminal_inner_area
                }

                // Render not connected chat (only terminal window).
                None => {
                    let terminal_area = block.inner(frame.area());

                    frame.render_widget(
                        block.title_top("Terminal"),
                        frame.area()
                    );

                    terminal_area
                }
            };

            // Update terminal properties and render it.

            let mut terminal_widget = state.terminal_widget.write();

            terminal_widget.width = terminal_area.width;
            terminal_widget.height = terminal_area.height;

            let stick_offset = terminal_widget.stick_offset(terminal_area.height as usize);

            let offset = match terminal_widget.offset {
                Some(offset) if offset >= stick_offset => {
                    terminal_widget.offset = None;

                    stick_offset
                }

                Some(offset) => offset,
                None => stick_offset
            };

            let list = List::new(terminal_widget.lines(offset));

            frame.render_widget(list, terminal_area);
        })?;

        // Do not handle any keyboard events while the command is running.
        // TODO: ctrl+c for interrupting the command.
        if running_command.is_none() {
            loop {
                match event::read()? {
                    Event::Key(key) => match key.code {
                        KeyCode::Esc => return Ok(()),

                        KeyCode::Char(char) => {
                            let mut terminal_widget = state.terminal_widget.write();

                            if let TerminalWidgetCurrentLine::Input(input) = &mut terminal_widget.ongoing {
                                input.push(char);

                                break;
                            }
                        }

                        KeyCode::Up | KeyCode::PageUp => {
                            let mut terminal_widget = state.terminal_widget.write();

                            let stick_offset = terminal_widget.stick_offset(terminal_widget.height as usize);

                            if let Some(offset) = &mut terminal_widget.offset {
                                *offset = offset.saturating_sub(1);
                            } else {
                                terminal_widget.offset = Some(stick_offset.saturating_sub(1));
                            }

                            break;
                        }

                        KeyCode::Down | KeyCode::PageDown => {
                            let mut terminal_widget = state.terminal_widget.write();

                            let stick_offset = terminal_widget.stick_offset(terminal_widget.height as usize);

                            if let Some(offset) = &mut terminal_widget.offset {
                                if *offset + 1 >= stick_offset {
                                    terminal_widget.offset = None;
                                } else {
                                    *offset += 1;
                                }

                                break;
                            }
                        }

                        KeyCode::Backspace => {
                            let mut terminal_widget = state.terminal_widget.write();

                            if let TerminalWidgetCurrentLine::Input(input) = &mut terminal_widget.ongoing {
                                input.pop();

                                break;
                            }
                        }

                        KeyCode::Enter => {
                            let mut terminal_widget = state.terminal_widget.write();
                            let mut command = None;

                            if let TerminalWidgetCurrentLine::Input(input) = terminal_widget.ongoing.clone() {
                                command = Some(input.clone());

                                let input = terminal_widget.prefix(input);

                                terminal_widget.push(input);
                            }

                            if let Some(command) = command {
                                terminal_widget.forbid_user_input();

                                let command = command.split_whitespace()
                                    .map(String::from)
                                    .collect::<Vec<String>>();

                                let actions_sender = actions_sender.clone();

                                let task = runtime.spawn(commands::run_command(command, move |action| {
                                    let _ = actions_sender.send(action);
                                }));

                                running_command = Some(task);

                                break;
                            }
                        }

                        _ => ()
                    }

                    Event::Paste(text) => {
                        let mut terminal_widget = state.terminal_widget.write();

                        if let TerminalWidgetCurrentLine::Input(input) = &mut terminal_widget.ongoing {
                            input.push_str(&text);

                            break;
                        }
                    }

                    Event::Resize(_, _) => {
                        force_render = true;

                        break;
                    }

                    _ => ()
                }
            }
        }
    }
}
