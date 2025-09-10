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

use libflowerpot::crypto::*;
use libflowerpot::block::*;

use crate::database::Database;
use crate::database::user::{UserRecord, UserInfo};
use crate::database::room::{RoomRecord, RoomInfo};
use crate::database::message::{MessageRecord, MessageInfo};

/// Handle block with all the inner transactions using provided database handle
/// and internal ID of the space to which the block belongs.
pub fn handle_block(
    database: &Database,
    space_id: i64,
    block: &Block
) -> anyhow::Result<()> {
    let block_hash = block.hash()
        .context("failed to calculate block hash")?;

    if let BlockContent::Transactions(transactions) = block.content() {
        for transaction in transactions {
            let event = Event::from_bytes(transaction.data())
                .context("failed to decode event from transaction body")?;

            let (is_valid, transaction_hash, transaction_author) = transaction.verify()?;

            // Skip invalid transactions.
            if !is_valid {
                continue;
            }

            let is_handled = database.is_handled(space_id, block_hash, transaction_hash)
                .context("failed to check if transaction is handled")?;

            if !is_handled {
                // TODO: transactions for atomic changes!

                database.mark_handled(space_id, block_hash, transaction_hash)
                    .context("failed to mark transaction as handled")?;

                let transaction_author_user = UserRecord::find(database.clone(), space_id, &transaction_author)
                    .context("failed to find user")?;

                let transaction_author_user = match transaction_author_user {
                    Some(user) => user,
                    None => UserRecord::create(database.clone(), &UserInfo {
                        space_id,
                        public_key: transaction_author,
                        nickname: None,
                        balance: 0
                    }).context("failed to create user")?
                };

                match event {
                    Event::PublicRoomCreate(name) => {
                        RoomRecord::create(database.clone(), &RoomInfo {
                            space_id,
                            name,
                            author_id: transaction_author_user.id(),
                            block_hash,
                            transaction_hash
                        }).context("failed to create room")?;
                    }

                    Event::PublicRoomMessage {
                        reply_message_block,
                        reply_message_transaction,
                        room_name,
                        text
                    } => {
                        let room = RoomRecord::find(database.clone(), space_id, &room_name)
                            .context("failed to find room")?;

                        // Ignore messages in non-existing rooms.
                        if let Some(room) = room {
                            MessageRecord::create(database.clone(), &MessageInfo {
                                room_id: room.id(),
                                user_id: transaction_author_user.id(),
                                block_hash,
                                transaction_hash,
                                reply_block_hash: reply_message_block,
                                reply_transaction_hash: reply_message_transaction,
                                timestamp: *block.timestamp(),
                                content: text
                            }).context("failed to create message")?;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Event {
    /// Create new space room with provided name.
    PublicRoomCreate(String),

    /// Send new message to the public room.
    ///
    /// - `reply_message_block` and `reply_message_transaction` are used to
    ///   determine the previous message to the given one. This is needed to
    ///   properly order the messages within a single block.
    /// - `room_name` is the name of the room where the message should be sent.
    /// - `text` is the plain text content of the message.
    PublicRoomMessage {
        /// Block hash where the previous message is stored.
        reply_message_block: Hash,

        /// Transaction hash where the previous hash is stored.
        reply_message_transaction: Hash,

        /// Name of the room.
        room_name: String,

        /// Plain text content of the message.
        text: String
    }
}

impl Event {
    pub const V1_PUBLIC_ROOM_CREATE: u8  = 0;
    pub const V1_PUBLIC_ROOM_MESSAGE: u8 = 1;

    pub fn to_bytes(&self) -> Box<[u8]> {
        match self {
            Self::PublicRoomCreate(name) => {
                let mut bytes = Vec::with_capacity(name.len() + 1);

                bytes.push(Self::V1_PUBLIC_ROOM_CREATE);
                bytes.extend(name.as_bytes());

                bytes.into_boxed_slice()
            }

            Self::PublicRoomMessage {
                reply_message_block,
                reply_message_transaction,
                room_name,
                text
            } => {
                let room_name_len = room_name.len();

                assert!(
                    (1..256).contains(&room_name_len),
                    "room name must be at least 1 byte long and shorter than 256 bytes"
                );

                let mut bytes = Vec::with_capacity(66 + room_name_len + text.len());

                bytes.push(Self::V1_PUBLIC_ROOM_MESSAGE);
                bytes.extend_from_slice(&reply_message_block.0);
                bytes.extend_from_slice(&reply_message_transaction.0);
                bytes.push(room_name_len as u8);
                bytes.extend(room_name.as_bytes());
                bytes.extend(text.as_bytes());

                bytes.into_boxed_slice()
            }
        }
    }

    pub fn from_bytes(bytes: impl AsRef<[u8]>) -> anyhow::Result<Self> {
        let bytes = bytes.as_ref();

        match bytes[0] {
            Self::V1_PUBLIC_ROOM_CREATE => {
                if bytes.len() < 2 {
                    anyhow::bail!("public room name must be at least 1 byte long");
                }

                let name = String::from_utf8_lossy(&bytes[1..])
                    .to_string();

                Ok(Self::PublicRoomCreate(name))
            }

            Self::V1_PUBLIC_ROOM_MESSAGE => {
                if bytes.len() < 66 {
                    anyhow::bail!("public room message event is too short");
                }

                let mut reply_message_block = [0; 32];
                let mut reply_message_transaction = [0; 32];

                reply_message_block.copy_from_slice(&bytes[1..33]);
                reply_message_transaction.copy_from_slice(&bytes[33..65]);

                let room_name_len = bytes[65] as usize;

                let room_name = String::from_utf8_lossy(&bytes[66..66 + room_name_len])
                    .to_string();

                let text = String::from_utf8_lossy(&bytes[66 + room_name_len..])
                    .to_string();

                Ok(Self::PublicRoomMessage {
                    reply_message_block: Hash::from(reply_message_block),
                    reply_message_transaction: Hash::from(reply_message_transaction),
                    room_name,
                    text
                })
            }

            _ => anyhow::bail!("unknown event format: {}", bytes[0])
        }
    }
}
