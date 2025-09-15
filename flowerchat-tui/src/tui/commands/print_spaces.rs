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

use crate::tui::app::{AppState, Action};
use crate::utils::make_table;

pub async fn run(
    state: AppState,
    output: impl Fn(Action)
) {
    let mut spaces_data = Vec::new();

    for space in state.database.spaces() {
        let title = match space.title() {
            Ok(title) if title.is_empty() => String::from("<unknown>"),
            Ok(title) => title,
            Err(err) => {
                output(Action::TerminalPush(format!("failed to get space title: {err}")));

                return;
            }
        };

        let root_block = match space.root_block() {
            Ok(root_block) => root_block,
            Err(err) => {
                output(Action::TerminalPush(format!("failed to get space root block: {err}")));

                return;
            }
        };

        let author = match space.author() {
            Ok(author) => author,
            Err(err) => {
                output(Action::TerminalPush(format!("failed to get space author: {err}")));

                return;
            }
        };

        let space_id = space.id().to_string();
        let root_block = root_block.to_base64();
        let public_key = author.to_base64();

        spaces_data.push([space_id, title, root_block, public_key]);
    }

    if spaces_data.is_empty() {
        return;
    }

    output(Action::TerminalPush(make_table(
        ["#", "Title", "Root block", "Public key"],
        spaces_data
    )));
}
