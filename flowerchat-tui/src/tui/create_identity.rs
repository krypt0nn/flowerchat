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

use libflowerpot::crypto::*;

use crate::consts::*;
use crate::utils::*;
use crate::identities::Identity;

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Selection {
    Title,
    Buttons
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SelectedButton {
    Save,
    Random,
    Exit
}

pub async fn render(
    terminal: &mut RatatuiTerminal
) -> anyhow::Result<Option<Identity>> {
    let mut selection = Selection::Title;
    let mut selected_button = SelectedButton::Save;

    let mut rng = get_rng();

    let mut secret_key = SecretKey::random(&mut rng);
    let mut title = String::new();

    loop {
        terminal.draw(|frame| {
            // Calculate areas for all the widgets.

            let [_, area, _] = Layout::horizontal([
                Constraint::Percentage(20),
                Constraint::Fill(1),
                Constraint::Percentage(20)
            ]).areas(frame.area());

            let [_, secret_key_area, _, title_area, _, buttons_area, _] = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(3),
                Constraint::Length(2),
                Constraint::Length(3),
                Constraint::Length(2),
                Constraint::Length(3),
                Constraint::Fill(1)
            ]).areas(area);

            let [_, save_button_area, _, random_button_area, _, exit_button_area, _] = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Length(8),
                Constraint::Length(1),
                Constraint::Length(10),
                Constraint::Length(1),
                Constraint::Length(8),
                Constraint::Fill(1)
            ]).areas(buttons_area);

            // Prepare styles.

            let disabled_style = Style::new().fg(TUI_DISABLED_COLOR);

            let disabled_block = Block::bordered()
                .border_style(disabled_style);

            let primary_block = Block::bordered()
                .border_style(Style::new().fg(TUI_PRIMARY_COLOR));

            // Draw secret key input.

            frame.render_widget(
                Paragraph::new(secret_key.to_base64())
                    .style(disabled_style)
                    .block(disabled_block.title_top("secret key")),
                secret_key_area
            );

            // Draw title input.

            let block = if selection == Selection::Title {
                primary_block.clone()
            } else {
                Block::bordered()
            };

            frame.render_widget(
                Paragraph::new(title.as_str())
                    .block(block.title_top("title")),
                title_area
            );

            // Draw buttons.

            let mut save_button = Paragraph::new("save")
                .centered()
                .block(Block::bordered());

            let mut random_button = Paragraph::new("random")
                .centered()
                .block(Block::bordered());

            let mut exit_button = Paragraph::new("exit")
                .centered()
                .block(Block::bordered());

            if selection == Selection::Buttons {
                match selected_button {
                    SelectedButton::Save   => save_button = save_button.block(primary_block),
                    SelectedButton::Random => random_button = random_button.block(primary_block),
                    SelectedButton::Exit   => exit_button = exit_button.block(primary_block)
                }
            }

            frame.render_widget(save_button, save_button_area);
            frame.render_widget(random_button, random_button_area);
            frame.render_widget(exit_button, exit_button_area);
        })?;

        loop {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char(char) if selection == Selection::Title => {
                        title.push(char);

                        break;
                    }

                    KeyCode::Backspace if selection == Selection::Title => {
                        title.pop();

                        break;
                    }

                    KeyCode::Up | KeyCode::PageUp => {
                        selection = match selection {
                            Selection::Title   => Selection::Title,
                            Selection::Buttons => Selection::Title
                        };

                        break;
                    }

                    KeyCode::Down | KeyCode::PageDown => {
                        selection = match selection {
                            Selection::Title   => Selection::Buttons,
                            Selection::Buttons => Selection::Buttons
                        };

                        break;
                    }

                    KeyCode::Left if selection == Selection::Buttons => {
                        selected_button = match selected_button {
                            SelectedButton::Save   => SelectedButton::Save,
                            SelectedButton::Random => SelectedButton::Save,
                            SelectedButton::Exit   => SelectedButton::Random
                        };

                        break;
                    }

                    KeyCode::Right if selection == Selection::Buttons => {
                        selected_button = match selected_button {
                            SelectedButton::Save   => SelectedButton::Random,
                            SelectedButton::Random => SelectedButton::Exit,
                            SelectedButton::Exit   => SelectedButton::Exit
                        };

                        break;
                    }

                    KeyCode::Enter if selection == Selection::Buttons => {
                        match selected_button {
                            SelectedButton::Save => {
                                return Ok(Some(Identity::new(title, secret_key)));
                            }

                            SelectedButton::Random => {
                                secret_key = SecretKey::random(&mut rng);

                                break;
                            }

                            SelectedButton::Exit => return Ok(None)
                        }
                    }

                    _ => ()
                }
            }
        }
    }
}
