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

use anyhow::Context;
use tokio::runtime::Handle;
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};
use tokio::sync::oneshot::{Sender, channel as oneshot_channel};

use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::{self, Event, KeyCode};

use ratatui::layout::*;
use ratatui::widgets::*;
use ratatui::text::*;
use ratatui::style::*;

use crate::consts::*;
use crate::database::Database;
use crate::database::space::SpaceRecord;
use crate::identities::Identity;

const FLOWERCHAT_LOGO: &str = r#"
  __ _                            _           _
 / _| |                          | |         | |
| |_| | _____      _____ _ __ ___| |__   __ _| |_
|  _| |/ _ \ \ /\ / / _ \ '__/ __| '_ \ / _` | __|
| | | | (_) \ V  V /  __/ | | (__| | | | (_| | |_
|_| |_|\___/ \_/\_/ \___|_|  \___|_| |_|\__,_|\__|
"#;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum TerminalWidgetCurrentLine {
    /// User's input.
    Input(String),

    /// Some running command's output.
    Output(String)
}

impl Default for TerminalWidgetCurrentLine {
    #[inline]
    fn default() -> Self {
        Self::Input(String::new())
    }
}

// TODO: inline terminal hints

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
struct TerminalWidget {
    pub history: Vec<String>,
    pub ongoing: TerminalWidgetCurrentLine,
    pub prefix: Option<String>,
    pub offset: Option<usize>,
    pub height: u16
}

impl TerminalWidget {
    pub fn prefix(&self, text: impl AsRef<str>) -> String {
        match &self.prefix {
            Some(prefix) => format!("{prefix} > {}", text.as_ref()),
            None => format!("> {}", text.as_ref())
        }
    }

    pub fn push(&mut self, text: impl AsRef<str>) {
        for line in text.as_ref().lines() {
            self.history.push(line.to_string());
        }
    }

    pub fn allow_user_input(&mut self) -> TerminalWidgetCurrentLine {
        let prev = self.ongoing.clone();

        self.ongoing = TerminalWidgetCurrentLine::Input(String::new());

        prev
    }

    pub fn forbid_user_input(&mut self) -> TerminalWidgetCurrentLine {
        let prev = self.ongoing.clone();

        self.ongoing = TerminalWidgetCurrentLine::Output(String::new());

        prev
    }

    pub fn len(&self) -> usize {
        self.history.len()
    }

    pub fn stick_offset(&self, height: usize) -> usize {
        let input_height = match &self.ongoing {
            TerminalWidgetCurrentLine::Input(text) |
            TerminalWidgetCurrentLine::Output(text) => {
                text.lines()
                    .count()
                    .max(1)
            }
        };

        let lines = self.len();

        if lines + input_height > height {
            (lines + input_height).saturating_sub(height)
        } else {
            0
        }
    }

    pub fn lines(&self, offset: usize) -> Vec<String> {
        let mut lines = self.history.iter()
            .skip(offset)
            .cloned()
            .collect::<Vec<String>>();

        match &self.ongoing {
            TerminalWidgetCurrentLine::Input(text) => {
                lines.push(self.prefix(text));
            }

            TerminalWidgetCurrentLine::Output(text) => {
                lines.push(text.to_string());
            }
        }

        lines
    }
}

fn print_help(output: impl Fn(CommandAction)) {
    output(CommandAction::Print(String::from("+---------+-------------------------+\n")));
    output(CommandAction::Print(String::from("| Command | Description             |\n")));
    output(CommandAction::Print(String::from("+---------+-------------------------+\n")));
    output(CommandAction::Print(String::from("| help    | list available commands |\n")));
    output(CommandAction::Print(String::from("| spaces  | list available spaces   |\n")));
    output(CommandAction::Print(String::from("+---------+-------------------------+\n")));
}

async fn print_spaces(output: impl Fn(CommandAction)) {
    let (send, recv) = oneshot_channel();

    output(CommandAction::RequestSpaces(send));

    match recv.await {
        Ok(spaces) => {
            let mut spaces_data = Vec::new();

            let mut max_id_len = 1;
            let mut max_title_len = 1;
            let mut max_root_block_len = 32;
            let mut max_public_key_len = 33;

            for space in spaces {
                let title = match space.title() {
                    Ok(title) if title.is_empty() => String::from("<unknown>"),
                    Ok(title) => title,
                    Err(err) => {
                        output(CommandAction::Print(format!("failed to get space title: {err}")));

                        return;
                    }
                };

                let root_block = match space.root_block() {
                    Ok(root_block) => root_block,
                    Err(err) => {
                        output(CommandAction::Print(format!("failed to get space root block: {err}")));

                        return;
                    }
                };

                let author = match space.author() {
                    Ok(author) => author,
                    Err(err) => {
                        output(CommandAction::Print(format!("failed to get space author: {err}")));

                        return;
                    }
                };

                let space_id = space.id().to_string();
                let root_block = root_block.to_base64();
                let public_key = author.to_base64();

                max_id_len = max_id_len.max(space_id.len());
                max_title_len = max_title_len.max(title.len());
                max_root_block_len = max_root_block_len.max(root_block.len());
                max_public_key_len = max_public_key_len.max(public_key.len());

                spaces_data.push((space_id, title, root_block, public_key));
            }

            if spaces_data.is_empty() {
                return;
            }

            output(CommandAction::Print(format!(
                "+-{}-+-{}-+-{}-+-{}-+\n",
                "-".repeat(max_id_len),
                "-".repeat(max_title_len),
                "-".repeat(max_root_block_len),
                "-".repeat(max_public_key_len)
            )));

            output(CommandAction::Print(format!(
                "| #{} | Title{} | Root block{} | Public key{} |\n",
                " ".repeat(max_id_len - 1),
                " ".repeat(max_title_len - 5),
                " ".repeat(max_root_block_len - 10),
                " ".repeat(max_public_key_len - 10)
            )));

            output(CommandAction::Print(format!(
                "+-{}-+-{}-+-{}-+-{}-+\n",
                "-".repeat(max_id_len),
                "-".repeat(max_title_len),
                "-".repeat(max_root_block_len),
                "-".repeat(max_public_key_len)
            )));

            for (space_id, title, root_block, public_key) in spaces_data {
                output(CommandAction::Print(format!(
                    "| {} | {} | {} | {} |\n",
                    space_id,
                    title,
                    root_block,
                    public_key
                )));
            }

            output(CommandAction::Print(format!(
                "+-{}-+-{}-+-{}-+-{}-+\n",
                "-".repeat(max_id_len),
                "-".repeat(max_title_len),
                "-".repeat(max_root_block_len),
                "-".repeat(max_public_key_len)
            )));
        }

        Err(err) => output(CommandAction::Print(format!("failed to get spaces: {err}")))
    }
}

async fn run_command(
    command: impl IntoIterator<Item = String>,
    output: impl Fn(CommandAction)
) {
    let mut command = command.into_iter();

    match command.next().as_deref() {
        Some("help") => print_help(output),
        Some("spaces") => print_spaces(output).await,

        Some(_) | None => print_help(output)
    }
}

#[derive(Debug)]
enum CommandAction {
    /// Print text to the terminal widget.
    Print(String),

    /// Request list of available spaces.
    RequestSpaces(Sender<Vec<SpaceRecord>>)
}

#[derive(Debug, Clone)]
struct Connection {
    pub space: SpaceRecord
}

pub async fn render(
    runtime: Handle,
    database: Database,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>
) -> anyhow::Result<()> {
    let mut terminal_widget = TerminalWidget::default();

    terminal_widget.push(FLOWERCHAT_LOGO.trim_matches('\n'));
    terminal_widget.push(format!("\nFlowerchat v{}", crate::VERSION));
    terminal_widget.push(format!("  flowerchat-protocol v{}", flowerchat_protocol::CRATE_VERSION));
    terminal_widget.push(format!("  protocol version: {}\n\n", flowerchat_protocol::PROTOCOL_VERSION));

    let mut running_command: Option<UnboundedReceiver<CommandAction>> = None;
    let mut connection: Option<Connection> = None;

    loop {
        if let Some(recv) = &mut running_command {
            match recv.recv().await {
                Some(action) => match action {
                    CommandAction::Print(text) => terminal_widget.push(text),
                    CommandAction::RequestSpaces(send) => {
                        let spaces = database.spaces()
                            .collect::<Vec<SpaceRecord>>();

                        let _ = send.send(spaces);
                    }
                }

                None => {
                    running_command = None;

                    terminal_widget.allow_user_input();
                    terminal_widget.push("\n");
                }
            }
        }

        terminal.draw(|frame| {
            let area = frame.area();

            terminal_widget.height = area.height;

            let stick_offset = terminal_widget.stick_offset(area.height as usize);

            let offset = match terminal_widget.offset {
                Some(offset) if offset >= stick_offset => {
                    terminal_widget.offset = None;

                    stick_offset
                }

                Some(offset) => offset,
                None => stick_offset
            };

            let list = List::new(terminal_widget.lines(offset));

            frame.render_widget(list, area);
        })?;

        // Do not handle any keyboard events while the command is running.
        // TODO: ctrl+c for interrupting the command.
        if running_command.is_none() {
            loop {
                match event::read()? {
                    Event::Key(key) => match key.code {
                        KeyCode::Esc => return Ok(()),

                        KeyCode::Char(char) => {
                            if let TerminalWidgetCurrentLine::Input(input) = &mut terminal_widget.ongoing {
                                input.push(char);

                                break;
                            }
                        }

                        KeyCode::Up | KeyCode::PageUp => {
                            let stick_offset = terminal_widget.stick_offset(terminal_widget.height as usize);

                            if let Some(offset) = &mut terminal_widget.offset {
                                *offset = offset.saturating_sub(1);
                            } else {
                                terminal_widget.offset = Some(stick_offset.saturating_sub(1));
                            }

                            break;
                        }

                        KeyCode::Down | KeyCode::PageDown => {
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
                            if let TerminalWidgetCurrentLine::Input(input) = &mut terminal_widget.ongoing {
                                input.pop();

                                break;
                            }
                        }

                        KeyCode::Enter => {
                            let mut command = None;

                            if let TerminalWidgetCurrentLine::Input(input) = terminal_widget.ongoing.clone() {
                                command = Some(input.clone());

                                terminal_widget.push(terminal_widget.prefix(input));
                            }

                            if let Some(command) = command {
                                terminal_widget.forbid_user_input();

                                let command = command.split_whitespace()
                                    .map(String::from)
                                    .collect::<Vec<String>>();

                                let (send, recv) = unbounded_channel();

                                runtime.spawn(run_command(command, move |action| {
                                    let _ = send.send(action);
                                }));

                                running_command = Some(recv);

                                break;
                            }
                        }

                        _ => ()
                    }

                    Event::Paste(text) => {
                        if let TerminalWidgetCurrentLine::Input(input) = &mut terminal_widget.ongoing {
                            input.push_str(&text);

                            break;
                        }
                    }

                    Event::Resize(_, _) => break,

                    _ => ()
                }
            }
        }
    }
}
