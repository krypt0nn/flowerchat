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

use std::path::Path;
use std::sync::Arc;

use spin::{Mutex, MutexGuard};
use rusqlite::Connection;

pub mod space;
pub mod shard;
pub mod user;
pub mod mint;
pub mod chat;
pub mod message;

#[derive(Debug, Clone)]
pub struct Database(Arc<Mutex<Connection>>);

impl Database {
    pub fn open(path: impl AsRef<Path>) -> rusqlite::Result<Self> {
        let connection = Connection::open(path)?;

        connection.execute_batch(r#"
            CREATE TABLE IF NOT EXISTS spaces (
                id         INTEGER NOT NULL UNIQUE AUTOINCREMENT,
                title      TEXT,
                root_block BLOB    NOT NULL,
                author     BLOB    NOT NULL,

                UNIQUE (root_block),

                PRIMARY KEY (id)
            );

            CREATE INDEX IF NOT EXISTS spaces_idx ON spaces (
                id,
                root_block,
                author
            );

            CREATE TABLE IF NOT EXISTS shards (
                space_id INTEGER NOT NULL,
                address  TEXT    NOT NULL,

                UNIQUE (space_id, address)
            );

            CREATE INDEX IF NOT EXISTS shards_idx ON shards (space_id);

            CREATE TABLE IF NOT EXISTS users (
                id         INTEGER NOT NULL UNIQUE AUTOINCREMENT,
                space_id   INTEGER NOT NULL,
                public_key BLOB    NOT NULL,
                nickname   TEXT             UNIQUE DEFAULT NULL,
                balance    INTEGER                 DEFAULT 0,

                UNIQUE (space_id, public_key),
                CHECK (balance >= 0),

                PRIMARY KEY (id),
                FOREIGN KEY (space_id) REFERENCES spaces (id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS userd_idx ON users (
                id,
                space_id,
                public_key,
                nickname
            );

            CREATE TABLE IF NOT EXISTS mints (
                user_id  INTEGER NOT NULL,
                nonce    BLOB    NOT NULL,

                block_hash       BLOB NOT NULL,
                transaction_hash BLOB NOT NULL,

                UNIQUE (user_id, nonce),

                FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS chats (
                id       INTEGER NOT NULL UNIQUE AUTOINCREMENT,
                space_id INTEGER NOT NULL,
                name     TEXT    NOT NULL,

                author_id        INTEGER NOT NULL,
                block_hash       BLOB    NOT NULL,
                transaction_hash BLOB    NOT NULL,

                UNIQUE (space_id, name),

                PRIMARY KEY (id),
                FOREIGN KEY (space_id)  REFERENCES spaces (id) ON DELETE CASCADE,
                FOREIGN KEY (author_id) REFERENCES users  (id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS chats_idx ON chats (
                id,
                space_id,
                name
            );

            CREATE TABLE IF NOT EXISTS messages (
                id      INTEGER NOT NULL UNIQUE AUTOINCREMENT,
                chat_id INTEGER NOT NULL,
                user_id INTEGER NOT NULL,

                block_hash       BLOB NOT NULL,
                transaction_hash BLOB NOT NULL,

                reply_block_hash       BLOB    NOT NULL,
                reply_transaction_hash BLOB    NOT NULL,
                timestamp              INTEGER NOT NULL,
                content                TEXT    NOT NULL,

                PRIMARY KEY (id),
                FOREIGN KEY (chat_id)  REFERENCES chats  (id) ON DELETE CASCADE,
                FOREIGN KEY (user_id)  REFERENCES users  (id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS messages_idx ON messages (
                id,
                chat_id,
                user_id,
                block_hash,
                transaction_hash
            );
        "#)?;

        Ok(Self(Arc::new(Mutex::new(connection))))
    }

    #[inline]
    fn lock(&self) -> MutexGuard<'_, Connection> {
        self.0.lock()
    }
}
