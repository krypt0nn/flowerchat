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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpaceInfo {
    /// Title of the space.
    pub title: String,

    /// Hash of the root block of the space's blockchain.
    pub root_block: Hash,

    /// Public key of the root block's author - creator of the space.
    pub author: PublicKey
}

#[derive(Debug, Clone)]
pub struct SpaceRecord(Database, i64);

impl SpaceRecord {
    /// Create new space record.
    pub fn create(
        database: Database,
        info: &SpaceInfo
    ) -> rusqlite::Result<Self> {
        let lock = database.lock();

        let mut query = lock.prepare_cached("
            INSERT INTO spaces (
                title,
                root_block,
                author
            ) VALUES (?1, ?2, ?3)
        ")?;

        let id = query.insert((
            info.title.as_str(),
            info.root_block.0,
            info.author.to_bytes()
        ))?;

        drop(query);
        drop(lock);

        Ok(Self(database, id))
    }

    /// Open space without verifying its existance.
    #[inline(always)]
    pub fn open_raw(database: Database, id: i64) -> Self {
        Self(database, id)
    }

    /// Open existing space from its ID.
    pub fn open(
        database: Database,
        id: i64
    ) -> rusqlite::Result<Self> {
        database.lock()
            .prepare_cached("SELECT 1 FROM spaces WHERE id = ?1")?
            .query_row([id], |_| Ok(()))?;

        Ok(Self(database, id))
    }

    /// Open existing space from its root block hash.
    pub fn find(
        database: Database,
        root_block: &Hash
    ) -> rusqlite::Result<Self> {
        let id = database.lock()
            .prepare_cached("SELECT id FROM spaces WHERE root_block = ?1")?
            .query_row([root_block.0], |row| row.get("id"))?;

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

    /// Title of the space.
    pub fn title(&self) -> rusqlite::Result<String> {
        self.0.lock()
            .prepare_cached("SELECT title FROM spaces WHERE id = ?1")?
            .query_row([self.1], |row| row.get("title"))
    }

    /// Hash of the root block of the space's blockchain.
    pub fn root_block(&self) -> rusqlite::Result<Hash> {
        self.0.lock()
            .prepare_cached("SELECT root_block FROM spaces WHERE id = ?1")?
            .query_row([self.1], |row| row.get::<_, [u8; 32]>("root_block"))
            .map(Hash::from)
    }

    /// Public key of the root block's author - creator of the space.
    pub fn author(&self) -> rusqlite::Result<PublicKey> {
        self.0.lock()
            .prepare_cached("SELECT author FROM spaces WHERE id = ?1")?
            .query_row([self.1], |row| row.get::<_, [u8; 33]>("author"))
            .and_then(|author| {
                // TODO: better error handling?
                PublicKey::from_bytes(author)
                    .ok_or_else(|| rusqlite::Error::InvalidQuery)
            })
    }

    /// Update title of the current space.
    pub fn update_title(
        &mut self,
        title: impl AsRef<str>
    ) -> rusqlite::Result<&mut Self> {
        self.0.lock()
            .prepare_cached("UPDATE spaces SET title = ?2 WHERE id = ?1")?
            .execute((self.1, title.as_ref()))?;

        Ok(self)
    }
}
