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

mod print_help;
mod print_spaces;
mod connect_space;
mod room_list;
mod room_create;

use crate::tui::app::{AppState, Action};

pub async fn run_command(
    command: impl IntoIterator<Item = String>,
    state: AppState,
    output: impl Fn(Action)
) {
    let is_connected = state.connection.read().is_some();

    let mut command = command.into_iter();

    match command.next().as_deref() {
        Some("help") => print_help::run(is_connected, output),

        // Connected

        Some("room") => match command.next().as_deref() {
            Some("list") => room_list::run(state, output),

            Some("create") => {
                let Some(name) = command.next() else {
                    output(Action::TerminalPush(String::from(
                        "public room name is not provided"
                    )));

                    return;
                };

                room_create::run(state, name, output).await;
            }

            Some("open") => {

            }

            Some(_) => output(Action::TerminalPush(String::from("unknown subcommand"))),
            _ => output(Action::TerminalPush(String::from("not subcommand provided")))
        }

        // Not connected

        Some("spaces") if !is_connected => print_spaces::run(state, output).await,

        Some("connect") if !is_connected => {
            let Some(space) = command.next() else {
                output(Action::TerminalPush(String::from(
                    "space id or root block hash is not provided"
                )));

                return;
            };

            let Some(identity) = command.next() else {
                output(Action::TerminalPush(String::from(
                    "identity (secret key) is not provided"
                )));

                return;
            };

            connect_space::run(space, identity, output).await
        }

        Some(_) | None => print_help::run(is_connected, output)
    }
}
