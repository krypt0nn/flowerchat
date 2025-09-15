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

use tokio::sync::oneshot::channel as oneshot_channel;

use crate::tui::app::Action;

pub async fn run(output: impl Fn(Action)) {
    let (send, recv) = oneshot_channel();

    output(Action::RequestSpaces(send));

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

                max_id_len = max_id_len.max(space_id.len());
                max_title_len = max_title_len.max(title.len());
                max_root_block_len = max_root_block_len.max(root_block.len());
                max_public_key_len = max_public_key_len.max(public_key.len());

                spaces_data.push((space_id, title, root_block, public_key));
            }

            if spaces_data.is_empty() {
                return;
            }

            output(Action::TerminalPush(format!(
                "+-{}-+-{}-+-{}-+-{}-+",
                "-".repeat(max_id_len),
                "-".repeat(max_title_len),
                "-".repeat(max_root_block_len),
                "-".repeat(max_public_key_len)
            )));

            output(Action::TerminalPush(format!(
                "| #{} | Title{} | Root block{} | Public key{} |",
                " ".repeat(max_id_len - 1),
                " ".repeat(max_title_len - 5),
                " ".repeat(max_root_block_len - 10),
                " ".repeat(max_public_key_len - 10)
            )));

            output(Action::TerminalPush(format!(
                "+-{}-+-{}-+-{}-+-{}-+",
                "-".repeat(max_id_len),
                "-".repeat(max_title_len),
                "-".repeat(max_root_block_len),
                "-".repeat(max_public_key_len)
            )));

            for (space_id, title, root_block, public_key) in spaces_data {
                output(Action::TerminalPush(format!(
                    "| {} | {} | {} | {} |",
                    space_id,
                    title,
                    root_block,
                    public_key
                )));
            }

            output(Action::TerminalPush(format!(
                "+-{}-+-{}-+-{}-+-{}-+",
                "-".repeat(max_id_len),
                "-".repeat(max_title_len),
                "-".repeat(max_root_block_len),
                "-".repeat(max_public_key_len)
            )));
        }

        Err(err) => output(Action::TerminalPush(format!("failed to get spaces: {err}")))
    }
}
