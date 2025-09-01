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

use anyhow::Context;
use time::UtcOffset;
use time::format_description::OwnedFormatItem;

use crate::consts::*;
use crate::database::Database;
use crate::database::space::SpaceRecord;
use crate::identities::Identity;

use super::*;

lazy_static::lazy_static! {
    static ref FORMAT: OwnedFormatItem = time::format_description::parse_owned::<2>(
        "[year]-[month]-[day] [hour]:[minute]:[second]"
    ).expect("failed to parse time format");

    static ref OFFSET: UtcOffset = UtcOffset::current_local_offset()
        .expect("failed to obtain local time offset");
}

#[derive(Debug, Clone)]
struct SpaceView {
    pub title: String,
    pub emoji: &'static str,
    pub shortname: String,
    pub record: SpaceRecord
}

#[derive(Debug, Clone)]
struct IdentityView {
    pub emoji: &'static str,
    pub shortname: String,
    pub created_at: String,
    pub identity: Identity
}

impl IdentityView {
    pub fn new(identity: Identity) -> anyhow::Result<Self> {
        let created_at = identity.created_at()
            .to_offset(*OFFSET)
            .format(&FORMAT)?;

        Ok(IdentityView {
            emoji: identity.emoji(),
            shortname: identity.shortname(),
            created_at,
            identity
        })
    }
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
    let spaces = database.spaces()
        .map(|record| {
            Ok::<_, anyhow::Error>(SpaceView {
                title: record.title()?,
                emoji: record.emoji()?,
                shortname: record.shortname()?,
                record
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut identities = crate::identities::read()?
        .into_iter()
        .map(IdentityView::new)
        .collect::<Result<Vec<_>, _>>()?;

    let mut selection = Selection::Space;
    let mut selected_space = 0;
    let mut selected_identity = 0;
    let mut selected_button = SelectedButton::Continue;

    loop {
        terminal.draw(|frame| {
            // Calculate areas for all the widgets.

            let [_, area, _] = Layout::horizontal([
                Constraint::Percentage(20),
                Constraint::Fill(1),
                Constraint::Percentage(20)
            ]).areas(frame.area());

            let [_, space_title_area, space_area, _, identity_title_area, identity_area, _, buttons_area, _] = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(2),
                Constraint::Length(5),
                Constraint::Length(2),
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

            // Prepare styles and components.

            let primary_block = Block::bordered()
                .border_style(Style::new().fg(TUI_PRIMARY_COLOR));

            let arrow_left = Text::from_iter([
                Line::from(""),
                Line::from(""),
                Line::from("←"),
                Line::from(""),
                Line::from("")
            ]);

            let arrow_right = Text::from_iter([
                Line::from(""),
                Line::from(""),
                Line::from("→"),
                Line::from(""),
                Line::from("")
            ]);

            // Draw space selection widget.

            frame.render_widget(
                Span::from("space").underlined(),
                space_title_area
            );

            if selected_space > 0 {
                frame.render_widget(arrow_left.clone(), space_left_area);
            }

            if selected_space < spaces.len() {
                frame.render_widget(arrow_right.clone(), space_right_area);
            }

            // Join space.
            if selected_space >= spaces.len() {
                let text = Text::from_iter([
                    Line::from(""),
                    Line::from("join space").centered(),
                    Line::from("")
                ]);

                let block = if selection == Selection::Space {
                    primary_block.clone()
                } else {
                    Block::bordered()
                };

                frame.render_widget(
                    Paragraph::new(text).block(block),
                    space_area
                );
            }

            // Choose existing space.
            else {
                let space = &spaces[selected_space];

                let text = Text::from_iter([
                    Line::from(space.title.as_str()),
                    Line::from(format!("{} {}", space.emoji, &space.shortname)),
                    Line::from("")
                ]);

                let block = if selection == Selection::Identity {
                    primary_block.clone()
                } else {
                    Block::bordered()
                };

                frame.render_widget(
                    Paragraph::new(text).block(block),
                    identity_area
                );
            }

            // Draw identity selection widgets.

            frame.render_widget(
                Span::from("identity").underlined(),
                identity_title_area
            );

            if selected_identity > 0 {
                frame.render_widget(arrow_left, identity_left_area);
            }

            if selected_identity < identities.len() {
                frame.render_widget(arrow_right, identity_right_area);
            }

            // Create new identity.
            if selected_identity >= identities.len() {
                let text = Text::from_iter([
                    Line::from(""),
                    Line::from("create new identity").centered(),
                    Line::from("")
                ]);

                let block = if selection == Selection::Identity {
                    primary_block.clone()
                } else {
                    Block::bordered()
                };

                frame.render_widget(
                    Paragraph::new(text).block(block),
                    identity_area
                );
            }

            // Choose existing identity.
            else {
                let identity = &identities[selected_identity];

                let text = Text::from_iter([
                    Line::from(identity.identity.title().as_str()),
                    Line::from(format!("{} {}", identity.emoji, &identity.shortname)),
                    Line::from(format!("created at {}", &identity.created_at))
                ]);

                let block = if selection == Selection::Identity {
                    primary_block.clone()
                } else {
                    Block::bordered()
                };

                frame.render_widget(
                    Paragraph::new(text).block(block),
                    identity_area
                );
            }

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
                        Selection::Space if selected_space < spaces.len() => {
                            selected_space += 1;

                            break;
                        }

                        Selection::Identity if selected_identity < identities.len() => {
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
                        // Create new identity.
                        Selection::Identity if selected_identity == identities.len() => {
                            terminal.clear()?;

                            if let Some(identity) = super::create_identity::render(terminal).await? {
                                let identity = IdentityView::new(identity)
                                    .context("failed to format newly created identity")?;

                                identities.push(identity);

                                let identities = identities.iter()
                                    .map(|view| view.identity.clone());

                                crate::identities::write(identities)
                                    .context("failed to write identities")?;
                            }

                            terminal.clear()?;

                            break;
                        }

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
