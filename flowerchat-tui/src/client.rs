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
use time::UtcDateTime;

use libflowerpot::crypto::*;
use libflowerpot::block::BlockContent;
use libflowerpot::viewer::Viewer;

use flowerchat_protocol::events::{Event, Events};

use crate::database::space::SpaceRecord;
use crate::database::user::{UserRecord, UserInfo};
use crate::database::public_room::{PublicRoomRecord, PublicRoomInfo};
use crate::database::public_message::{
    PublicRoomMessageRecord, PublicRoomMessageInfo
};
use crate::database::Database;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandlerEvent {
    pub block_hash: Hash,
    pub block_public_key: PublicKey,
    pub block_timestamp: time::UtcDateTime,

    pub transaction_hash: Hash,
    pub transaction_public_key: PublicKey,

    pub event: Events
}

/// Read blocks using the provided blockchain viewer, decode transactions into
/// flowerchat events and send them to the `handler` callback.
///
/// If `handler` returns `Err(E)` then this function will be terminated and
/// `Ok(Some(E))` will be returned.
pub async fn read_events<E>(
    mut viewer: Viewer,
    mut handler: impl FnMut(HandlerEvent) -> Result<(), E>
) -> anyhow::Result<Option<E>> {
    loop {
        if let Some(block) = viewer.forward().await &&
            let BlockContent::Transactions(transactions) = block.block.content()
        {
            for transaction in transactions {
                let (
                    is_valid,
                    transaction_hash,
                    transaction_public_key
                ) = transaction.verify().context("failed to verify transaction")?;

                if is_valid {
                    let result = handler(HandlerEvent {
                        block_hash: block.hash,
                        block_public_key: block.public_key.clone(),
                        block_timestamp: *block.block.timestamp(),

                        transaction_hash,
                        transaction_public_key,

                        event: Events::deserialize(&mut transaction.data())
                            .context("failed to deserialize transaction into flowerchat event")?
                    });

                    if let Err(err) = result {
                        return Ok(Some(err));
                    }
                }
            }
        }
    }
}

pub enum Update {
    /// Iterating over the network blocks to verify the blockchain integrity
    /// until we find not yet processed blocks.
    Verification {
        /// Hash of the currently processing block.
        block_hash: Hash,

        /// Hash of the currently processing transaction within this block.
        transaction_hash: Hash,

        /// Timestamp of when the block was made.
        block_timestamp: UtcDateTime,

        /// Estimated blocks verification progress. It's calculated as
        /// `block_timestamp / curr_timestamp` where `curr_timestamp` is when
        /// we started the verification progress.
        estimated_progress: f32
    },

    /// Network blocks viewer reached the end of the known blockchain.
    VerificationDone,

    /// New event was processed.
    NewEvent {
        /// Hash of the currently processing block.
        block_hash: Hash,

        /// Hash of the currently processing transaction within this block.
        transaction_hash: Hash,

        /// Timestamp of when the block was made.
        block_timestamp: UtcDateTime
    }
}

/// Read blocks using the provided blockchain viewer, decode transactions into
/// flowerchat events and process them using the database entry.
pub async fn run(
    database: Database,
    viewer: Viewer,
    mut updater: impl FnMut(Update)
) -> anyhow::Result<()> {
    let space = SpaceRecord::find(database.clone(), viewer.root_block())
        .context("failed to find space in the database with the viewer's root block")?;

    let Some(space) = space else {
        anyhow::bail!("space with requested hash is not stored in the database");
    };

    let curr_timestamp = UtcDateTime::now().unix_timestamp() as f32;

    let mut verification_done = false;

    if viewer.blocks_pool().is_empty() {
        updater(Update::VerificationDone);

        verification_done = true;
    }

    let result = read_events(viewer, move |event| -> anyhow::Result<()> {
        let is_handled = database.is_handled(
            space.id(),
            event.block_hash,
            event.transaction_hash
        ).context("failed to verify if transaction is handled")?;

        if !is_handled {
            if !verification_done {
                updater(Update::VerificationDone);

                verification_done = true;
            }

            fn find_or_create_user(
                database: Database,
                space_id: i64,
                public_key: PublicKey
            ) -> anyhow::Result<UserRecord> {
                let user = UserRecord::find(
                    database.clone(),
                    space_id,
                    &public_key
                ).context("failed to find user")?;

                match user {
                    Some(user) => Ok(user),
                    None => UserRecord::create(database, &UserInfo {
                        space_id,
                        public_key,
                        nickname: None
                    }).context("failed to create user")
                }
            }

            match event.event {
                Events::CreatePublicRoom(info) => {
                    let author = find_or_create_user(
                        database.clone(),
                        space.id(),
                        event.transaction_public_key
                    )?;

                    PublicRoomRecord::create(database.clone(), &PublicRoomInfo {
                        space_id: space.id(),
                        name: info.name().to_string(),
                        author_id: author.id(),
                        block_hash: event.block_hash,
                        transaction_hash: event.transaction_hash
                    }).context("failed to create public room")?;
                }

                Events::PublicRoomMessage(info) => {
                    let user = find_or_create_user(
                        database.clone(),
                        space.id(),
                        event.transaction_public_key
                    )?;

                    let room = PublicRoomRecord::find(
                        database.clone(),
                        space.id(),
                        info.room_name()
                    ).context("failed to find public room")?;

                    // Skip event handling if room doesn't exist.
                    let Some(room) = room else {
                        return Ok(());
                    };

                    PublicRoomMessageRecord::create(database.clone(), &PublicRoomMessageInfo {
                        room_id: room.id(),
                        user_id: user.id(),
                        block_hash: event.block_hash,
                        transaction_hash: event.transaction_hash,
                        timestamp: event.block_timestamp,
                        content: info.content().to_string()
                    }).context("failed to create public room message")?;
                }
            }

            database.mark_handled(
                space.id(),
                event.block_hash,
                event.transaction_hash
            ).context("failed to mark transaction as handled")?;

            updater(Update::NewEvent {
                block_hash: event.block_hash,
                transaction_hash: event.transaction_hash,
                block_timestamp: event.block_timestamp
            });
        }

        else if !verification_done {
            updater(Update::Verification {
                block_hash: event.block_hash,
                transaction_hash: event.transaction_hash,
                block_timestamp: event.block_timestamp,
                estimated_progress: event.block_timestamp.unix_timestamp() as f32 / curr_timestamp
            });
        }

        Ok(())
    }).await?;

    if let Some(err) = result {
        anyhow::bail!(err);
    }

    Ok(())
}
