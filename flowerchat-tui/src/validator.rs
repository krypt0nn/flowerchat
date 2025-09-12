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

use std::collections::HashSet;

use libflowerpot::crypto::Hash;

use flowerchat_protocol::events::Events;

use crate::client::HandlerEvent;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct ValidatorState {
    pub handled_transactions: HashSet<Hash>,
    pub public_rooms: HashSet<String>
}

/// Try to handle provided event. Return `true` if the event is processed
/// successfully, `false` if there were some problems with it.
pub fn handle_event(
    state: &mut ValidatorState,
    event: &HandlerEvent
) -> bool {
    // Skip already handled transactions.
    if state.handled_transactions.contains(&event.transaction_hash) {
        return false;
    }

    match &event.event {
        Events::CreatePublicRoom(info) => {
            // Forbid transaction if room with this name already exists.
            if !state.public_rooms.insert(info.name().to_string()) {
                return false;
            }

            true
        }

        Events::PublicRoomMessage(_) => true
    }
}
