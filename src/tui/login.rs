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

use crate::consts::*;
use crate::database::Database;
use crate::database::space::SpaceRecord;

use super::*;

#[derive(Debug, Clone)]
struct SpaceView {
    pub title: String,
    pub emoji: &'static str,
    pub shortname: String,
    pub record: SpaceRecord
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Selection {
    Space,
    Identity,
    Buttons
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SelectedButton {
    Continue,
    Exit
}

pub async fn render(
    database: Database,
    terminal: &mut RatatuiTerminal
) -> anyhow::Result<()> {
    let mut spaces = database.spaces()
        .map(|record| {
            Ok::<_, anyhow::Error>(SpaceView {
                title: record.title()?,
                emoji: record.emoji()?,
                shortname: record.shortname()?,
                record
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut identities = crate::identities::read()?;

    let mut selection = Selection::Space;
    let mut selected_space = 0;
    let mut selected_identity = 0;
    let mut selected_button = SelectedButton::Continue;

    loop {
        terminal.draw(|frame| {
            // Calculate areas for all the widgets.

            let [_, area, _] = Layout::horizontal([
                Constraint::Percentage(15),
                Constraint::Fill(1),
                Constraint::Percentage(15)
            ]).areas(frame.area());

            let [_, space_area, _, identity_area, _, buttons_area, _] = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(5),
                Constraint::Length(2),
                Constraint::Length(5),
                Constraint::Length(2),
                Constraint::Length(3),
                Constraint::Fill(1)
            ]).areas(area);

            let [space_left_area, _, space_area, _, space_right_area] = Layout::horizontal([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Length(1)
            ]).areas(space_area);

            let [identity_left_area, _, identity_area, _, identity_right_area] = Layout::horizontal([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Length(1)
            ]).areas(identity_area);

            let [_, continue_button_area, _, exit_button_area, _] = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Length(12),
                Constraint::Length(1),
                Constraint::Length(8),
                Constraint::Fill(1)
            ]).areas(buttons_area);

            // Prepare styles.

            let primary_block = Block::bordered()
                .border_style(Style::new().fg(TUI_PRIMARY_COLOR));

            // Draw identity selection widgets.

            // Draw buttons.

            let mut continue_button = Paragraph::new("continue")
                .centered()
                .block(Block::bordered());

            let mut exit_button = Paragraph::new("exit")
                .centered()
                .block(Block::bordered());

            if selection == Selection::Buttons {
                match selected_button {
                    SelectedButton::Continue => continue_button = continue_button.block(primary_block),
                    SelectedButton::Exit => exit_button = exit_button.block(primary_block)
                }
            }

            frame.render_widget(continue_button, continue_button_area);
            frame.render_widget(exit_button, exit_button_area);
        })?;

        loop {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Up | KeyCode::PageUp => {
                        selection = match selection {
                            Selection::Space    => Selection::Space,
                            Selection::Identity => Selection::Space,
                            Selection::Buttons  => Selection::Identity
                        };

                        break;
                    }

                    KeyCode::Down | KeyCode::PageDown => {
                        selection = match selection {
                            Selection::Space    => Selection::Identity,
                            Selection::Identity => Selection::Buttons,
                            Selection::Buttons  => Selection::Buttons
                        };

                        break;
                    }

                    KeyCode::Left => match selection {
                        Selection::Space if selected_space > 0 => {
                            selected_space -= 1;

                            break;
                        }

                        Selection::Identity if selected_identity > 0 => {
                            selected_identity -= 1;

                            break;
                        }

                        Selection::Buttons if selected_button == SelectedButton::Exit => {
                            selected_button = SelectedButton::Continue;

                            break;
                        }

                        _ => ()
                    }

                    KeyCode::Right => match selection {
                        Selection::Space if selected_space + 1 < spaces.len() => {
                            selected_space += 1;

                            break;
                        }

                        Selection::Identity if selected_identity + 1 < identities.len() => {
                            selected_identity += 1;

                            break;
                        }

                        Selection::Buttons if selected_button == SelectedButton::Continue => {
                            selected_button = SelectedButton::Exit;

                            break;
                        }

                        _ => ()
                    }

                    KeyCode::Enter => match selection {
                        Selection::Buttons => match selected_button {
                            SelectedButton::Continue => (),
                            SelectedButton::Exit => return Ok(())
                        }

                        _ => ()
                    }

                    _ => ()
                }
            }
        }
    }
}
