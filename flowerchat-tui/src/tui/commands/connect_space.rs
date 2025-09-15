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

use libflowerpot::crypto::*;
use libflowerpot::client::Client;
use libflowerpot::pool::ShardsPool;
use libflowerpot::viewer::Viewer;

use crate::tui::app::Action;

pub async fn run(
    space: impl ToString,
    identity: impl AsRef<[u8]>,
    output: impl Fn(Action)
) {
    let Some(identity) = SecretKey::from_base64(identity) else {
        output(Action::TerminalPush(String::from("invalid identity format: base64 secret key expected")));

        return;
    };

    let (send, recv) = oneshot_channel();

    output(Action::RequestSpaceRecord(space.to_string(), send));

    match recv.await {
        Ok(Ok(space)) => {
            let shards = match space.shards() {
                Ok(shards) => shards,
                Err(err) => {
                    output(Action::TerminalPush(format!("failed to get space shards: {err}")));

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

            output(Action::TerminalSetCurrentLine(String::from("bootstrapping shards pool...")));

            let client = Client::default();
            let mut pool = ShardsPool::new(shards);

            pool.update(&client).await;

            output(Action::TerminalSetCurrentLine(String::new()));
            output(Action::TerminalPush(format!(
                "bootstrapping shards pool... {} active, {} inactive\n",
                pool.active().count(),
                pool.inactive().count()
            )));

            output(Action::TerminalPush(String::from("opening blockchain viewer...")));

            let viewer = match Viewer::open(client, pool.active(), Some(root_block)).await {
                Ok(Some(viewer)) => viewer,

                Ok(None) => {
                    output(Action::TerminalPush(String::from("none of shards provides space blockchain")));

                    return;
                }

                Err(err) => {
                    output(Action::TerminalPush(format!("failed to open blockchain viewer: {err}")));

                    return;
                }
            };

            output(Action::TerminalPush(String::from("connecting to the space...")));
            output(Action::Connect(space, identity, viewer));
        }

        Ok(Err(err)) => output(Action::TerminalPush(format!("failed to obtain space record: {err}"))),
        Err(err) => output(Action::TerminalPush(format!("failed to obtain space record: {err}")))
    }
}
