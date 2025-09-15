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

use crate::tui::app::Action;

pub async fn run_command(
    command: impl IntoIterator<Item = String>,
    output: impl Fn(Action)
) {
    let mut command = command.into_iter();

    match command.next().as_deref() {
        Some("help") => print_help::run(output),
        Some("spaces") => print_spaces::run(output).await,

        Some("connect") => {
            let Some(space) = command.next() else {
                output(Action::TerminalPush(String::from("space id or root block hash is not specified")));

                return;
            };

            let Some(identity) = command.next() else {
                output(Action::TerminalPush(String::from("identity (secret key) is not specified")));

                return;
            };

            connect_space::run(space, identity, output).await
        }

        Some(_) | None => print_help::run(output)
    }
}
