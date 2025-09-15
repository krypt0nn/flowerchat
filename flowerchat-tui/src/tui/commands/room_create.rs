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

use rand_chacha::rand_core::RngCore;

use libflowerpot::transaction::Transaction;

use flowerchat_protocol::types::room_name::RoomName;
use flowerchat_protocol::events::{Event, Events};
use flowerchat_protocol::events::rooms::create_public::CreatePublicRoomEvent;

use crate::database::public_room::PublicRoomRecord;
use crate::tui::app::{AppState, Action};
use crate::utils::get_rng;

pub async fn run(
    state: AppState,
    name: impl ToString,
    output: impl Fn(Action)
) {
    let Some(connection) = &*state.connection.read() else {
        output(Action::TerminalPush(String::from("Not connected")));

        return;
    };

    let Some(name) = RoomName::new(name.to_string()) else {
        output(Action::TerminalPush(String::from("Room name is invalid")));

        return;
    };

    let database = state.database.clone();

    match PublicRoomRecord::find(database, connection.space.id(), &name) {
        Ok(None) => {
            output(Action::TerminalSetCurrentLine(String::from("Building transaction...")));

            let event = Events::from(CreatePublicRoomEvent::from(name));

            let mut data = Vec::new();

            if let Err(err) = event.serialize(&mut data) {
                output(Action::TerminalSetCurrentLine(String::new()));
                output(Action::TerminalPush(format!("Failed to create event: {err}")));

                return;
            };

            let transaction = Transaction::create(
                &connection.identity,
                get_rng().next_u64(),
                data
            );

            output(Action::TerminalSetCurrentLine(String::new()));

            let transaction = match transaction {
                Ok(transaction) => transaction,
                Err(err) => {
                    output(Action::TerminalPush(format!("Failed to create transaction: {err}")));

                    return;
                }
            };

            output(Action::TerminalPush(format!(
                "Building transaction... {}",
                transaction.hash().to_base64()
            )));

            let shards = connection.shards_pool.active()
                .map(String::from)
                .collect::<Vec<String>>();

            output(Action::TerminalSetCurrentLine(format!(
                "Announcing transaction to {} active shards...",
                shards.len()
            )));

            let result = connection.client.put_transaction(
                &shards,
                &transaction
            ).await;

            output(Action::TerminalSetCurrentLine(String::new()));

            if let Err(err) = result {
                output(Action::TerminalPush(format!(
                    "Announcing transaction to {} active shards... Error",
                    shards.len()
                )));

                output(Action::TerminalPush(format!("Failed to announce transaction: {err}")));
            }

            else {
                output(Action::TerminalPush(format!(
                    "Announcing transaction to {} active shards... Done",
                    shards.len()
                )));
            }
        }

        Ok(Some(_)) => output(Action::TerminalPush(String::from("Room with such name already exists"))),
        Err(err) => output(Action::TerminalPush(format!("Failed to verify if such room already exists: {err}")))
    }
}
