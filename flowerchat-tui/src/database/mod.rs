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
use std::iter::FusedIterator;

use spin::{Mutex, MutexGuard};
use rusqlite::Connection;

use libflowerpot::crypto::Hash;

pub mod space;
pub mod shard;
pub mod user;
pub mod public_room;
pub mod public_message;

#[derive(Debug, Clone)]
pub struct Database(Arc<Mutex<Connection>>);

impl Database {
    pub fn open(path: impl AsRef<Path>) -> rusqlite::Result<Self> {
        let connection = Connection::open(path)?;

        connection.execute_batch(r#"
            CREATE TABLE IF NOT EXISTS spaces (
                id         INTEGER NOT NULL UNIQUE,
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

            CREATE TABLE IF NOT EXISTS handled_transactions (
                space_id         INTEGER NOT NULL,
                block_hash       BLOB    NOT NULL,
                transaction_hash BLOB    NOT NULL,

                PRIMARY KEY (space_id, block_hash, transaction_hash)
            );

            CREATE TABLE IF NOT EXISTS shards (
                space_id INTEGER NOT NULL,
                address  TEXT    NOT NULL,

                UNIQUE (space_id, address)
            );

            CREATE INDEX IF NOT EXISTS shards_idx ON shards (space_id);

            CREATE TABLE IF NOT EXISTS users (
                id         INTEGER NOT NULL UNIQUE,
                space_id   INTEGER NOT NULL,
                public_key BLOB    NOT NULL,
                nickname   TEXT             UNIQUE DEFAULT NULL,

                UNIQUE (space_id, public_key),

                PRIMARY KEY (id),
                FOREIGN KEY (space_id) REFERENCES spaces (id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS userd_idx ON users (
                id,
                space_id,
                public_key,
                nickname
            );

            CREATE TABLE IF NOT EXISTS public_rooms (
                id       INTEGER NOT NULL UNIQUE,
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

            CREATE INDEX IF NOT EXISTS public_rooms_idx ON public_rooms (
                id,
                space_id,
                name
            );

            CREATE TABLE IF NOT EXISTS public_messages (
                id      INTEGER NOT NULL UNIQUE,
                room_id INTEGER NOT NULL,
                user_id INTEGER NOT NULL,

                block_hash       BLOB NOT NULL,
                transaction_hash BLOB NOT NULL,

                timestamp INTEGER NOT NULL,
                content   TEXT    NOT NULL,

                PRIMARY KEY (id),
                FOREIGN KEY (room_id)  REFERENCES public_rooms (id) ON DELETE CASCADE,
                FOREIGN KEY (user_id)  REFERENCES users        (id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS public_messages_idx ON public_messages (
                id,
                room_id,
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

    /// Check if transaction with given values is handled.
    pub fn is_handled(
        &self,
        space_id: i64,
        block_hash: impl Into<Hash>,
        transaction_hash: impl Into<Hash>
    ) -> anyhow::Result<bool> {
        let block_hash: Hash = block_hash.into();
        let transaction_hash: Hash = transaction_hash.into();

        let lock = self.lock();

        let mut query = lock.prepare_cached("
            SELECT 1 FROM handled_transactions
            WHERE
                space_id = ?1 AND
                block_hash = ?2 AND
                transaction_hash = ?3
            LIMIT 1
        ")?;

        let result = query.query_one((
            space_id,
            block_hash.0,
            transaction_hash.0
        ), |_| Ok(true));

        match result {
            Ok(result) => Ok(result),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(false),
            Err(err) => anyhow::bail!(err)
        }
    }

    /// Mark transaction with given values as handled.
    pub fn mark_handled(
        &self,
        space_id: i64,
        block_hash: impl Into<Hash>,
        transaction_hash: impl Into<Hash>
    ) -> anyhow::Result<()> {
        let block_hash: Hash = block_hash.into();
        let transaction_hash: Hash = transaction_hash.into();

        self.lock()
            .prepare_cached("
                INSERT OR IGNORE INTO handled_transactions (
                    space_id,
                    block_hash,
                    transaction_hash
                ) VALUES (?1, ?2, ?3)
            ")?
            .execute((space_id, block_hash.0, transaction_hash.0))?;

        Ok(())
    }

    /// Get iterator over all the stored spaces.
    pub fn spaces(&self) -> SpacesIter {
        SpacesIter {
            database: self.clone(),
            current: 0
        }
    }
}

pub struct SpacesIter {
    database: Database,
    current: i64
}

impl Iterator for SpacesIter {
    type Item = space::SpaceRecord;

    fn next(&mut self) -> Option<Self::Item> {
        let lock = self.database.lock();

        let mut query = lock.prepare_cached("
            SELECT id FROM spaces WHERE id > ?1 ORDER BY id ASC LIMIT 1
        ").ok()?;

        let id = query.query_row([self.current], |row| row.get("id")).ok()?;

        self.current = id;

        let record = space::SpaceRecord::open_raw(
            self.database.clone(),
            id
        );

        Some(record)
    }
}

impl FusedIterator for SpacesIter {}
