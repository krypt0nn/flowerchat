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

use libflowerpot::crypto::*;

use super::Database;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MessageInfo {
    /// Internal ID of the chat.
    pub chat_id: i64,

    /// Internal ID of the message sender.
    pub user_id: i64,

    /// Hash of the block where this record is stored.
    pub block_hash: Hash,

    /// Hash of the transaction where this record is stored.
    pub transaction_hash: Hash,

    /// Hash of the block where the previous message's record is stored.
    ///
    /// This information is needed to reconstruct proper messages order.
    pub reply_block_hash: Hash,

    /// Hash of the transaction where the previous message's record is stored.
    ///
    /// This information is needed to reconstruct proper messages order.
    pub reply_transaction_hash: Hash,

    /// Timestamp of when the message was approved by a validator.
    pub timestamp: time::UtcDateTime,

    /// Content of the message.
    pub content: String
}

#[derive(Debug, Clone)]
pub struct MessageRecord(Database, i64);

impl MessageRecord {
    /// Create new message record.
    pub fn create(
        database: Database,
        info: &MessageInfo
    ) -> rusqlite::Result<Self> {
        let lock = database.lock();

        let mut query = lock.prepare_cached("
            INSERT INTO messages (
                chat_id,
                user_id,
                block_hash,
                transaction_hash,
                reply_block_hash,
                reply_transaction_hash,
                timestamp,
                content
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        ")?;

        let id = query.insert((
            info.chat_id,
            info.user_id,
            info.block_hash.0,
            info.transaction_hash.0,
            info.reply_block_hash.0,
            info.reply_transaction_hash.0,
            info.timestamp.unix_timestamp(),
            info.content.as_str()
        ))?;

        drop(query);
        drop(lock);

        Ok(Self(database, id))
    }

    /// Open existing message from its ID.
    pub fn open(
        database: Database,
        id: i64
    ) -> rusqlite::Result<Self> {
        database.lock()
            .prepare_cached("SELECT 1 FROM messages WHERE id = ?1")?
            .query_row([id], |_| Ok(()))?;

        Ok(Self(database, id))
    }

    #[inline(always)]
    pub const fn database(&self) -> &Database {
        &self.0
    }

    /// Internal ID of the space.
    #[inline(always)]
    pub const fn id(&self) -> i64 {
        self.1
    }

    /// Internal ID of the chat.
    pub fn chat_id(&self) -> rusqlite::Result<i64> {
        self.0.lock()
            .prepare_cached("SELECT chat_id FROM chats WHERE id = ?1")?
            .query_row([self.1], |row| row.get("chat_id"))
    }

    /// Internal ID of the message sender.
    pub fn user_id(&self) -> rusqlite::Result<i64> {
        self.0.lock()
            .prepare_cached("SELECT user_id FROM chats WHERE id = ?1")?
            .query_row([self.1], |row| row.get("user_id"))
    }

    /// Hash of the block where this record is stored.
    pub fn block_hash(&self) -> rusqlite::Result<Hash> {
        self.0.lock()
            .prepare_cached("SELECT block_hash FROM chats WHERE id = ?1")?
            .query_row([self.1], |row| row.get::<_, [u8; 32]>("block_hash"))
            .map(Hash::from)
    }

    /// Hash of the transaction where this record is stored.
    pub fn transaction_hash(&self) -> rusqlite::Result<Hash> {
        self.0.lock()
            .prepare_cached("SELECT transaction_hash FROM chats WHERE id = ?1")?
            .query_row([self.1], |row| row.get::<_, [u8; 32]>("transaction_hash"))
            .map(Hash::from)
    }

    /// Hash of the block where the previous message's record is stored.
    ///
    /// This information is needed to reconstruct proper messages order.
    pub fn reply_block_hash(&self) -> rusqlite::Result<Hash> {
        self.0.lock()
            .prepare_cached("SELECT reply_block_hash FROM chats WHERE id = ?1")?
            .query_row([self.1], |row| row.get::<_, [u8; 32]>("reply_block_hash"))
            .map(Hash::from)
    }

    /// Hash of the transaction where the previous message's record is stored.
    ///
    /// This information is needed to reconstruct proper messages order.
    pub fn reply_transaction_hash(&self) -> rusqlite::Result<Hash> {
        self.0.lock()
            .prepare_cached("SELECT reply_transaction_hash FROM chats WHERE id = ?1")?
            .query_row([self.1], |row| row.get::<_, [u8; 32]>("reply_transaction_hash"))
            .map(Hash::from)
    }
}
