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
pub struct PublicRoomInfo {
    /// Internal ID of the space this room belongs to.
    pub space_id: i64,

    /// Name of the room.
    pub name: String,

    /// Internal ID of the user who created the room.
    pub author_id: i64,

    /// Hash of the block where this record is stored.
    pub block_hash: Hash,

    /// Hash of the transaction where this record is stored.
    pub transaction_hash: Hash
}

#[derive(Debug, Clone)]
pub struct PublicRoomRecord(Database, i64);

impl PublicRoomRecord {
    /// Create new room record.
    pub fn create(
        database: Database,
        info: &PublicRoomInfo
    ) -> rusqlite::Result<Self> {
        let lock = database.lock();

        let mut query = lock.prepare_cached("
            INSERT INTO public_rooms (
                space_id,
                name,
                author_id,
                block_hash,
                transaction_hash
            ) VALUES (?1, ?2, ?3, ?4, ?5)
        ")?;

        let id = query.insert((
            info.space_id,
            info.name.as_str(),
            info.author_id,
            info.block_hash.0,
            info.transaction_hash.0
        ))?;

        drop(query);
        drop(lock);

        Ok(Self(database, id))
    }

    /// Open room without verifying its existance.
    #[inline(always)]
    pub fn open_raw(database: Database, id: i64) -> Self {
        Self(database, id)
    }

    /// Open existing room from its ID.
    pub fn open(
        database: Database,
        id: i64
    ) -> rusqlite::Result<Self> {
        database.lock()
            .prepare_cached("SELECT 1 FROM public_rooms WHERE id = ?1")?
            .query_row([id], |_| Ok(()))?;

        Ok(Self(database, id))
    }

    /// Open existing room from its space ID and name. Return `None` if such
    /// room doesn't exist.
    pub fn find(
        database: Database,
        space_id: i64,
        name: impl AsRef<str>
    ) -> rusqlite::Result<Option<Self>> {
        let lock = database.lock();

        let mut query = lock.prepare_cached("
            SELECT id FROM public_rooms WHERE space_id = ?1 AND name = ?2
        ")?;

        let id = query.query_row((
            space_id, name.as_ref()
        ), |row| row.get("id"));

        drop(query);
        drop(lock);

        match id {
            Ok(id) => Ok(Some(Self(database, id))),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(err) => Err(err)
        }
    }

    #[inline(always)]
    pub const fn database(&self) -> &Database {
        &self.0
    }

    /// Internal ID of the room.
    #[inline(always)]
    pub const fn id(&self) -> i64 {
        self.1
    }

    #[inline]
    pub fn into_inner(self) -> (Database, i64) {
        (self.0, self.1)
    }

    /// Internal ID of the space this room belongs to.
    pub fn space_id(&self) -> rusqlite::Result<i64> {
        self.0.lock()
            .prepare_cached("SELECT space_id FROM public_rooms WHERE id = ?1")?
            .query_row([self.1], |row| row.get("space_id"))
    }

    /// Name of the room.
    pub fn name(&self) -> rusqlite::Result<String> {
        self.0.lock()
            .prepare_cached("SELECT name FROM public_rooms WHERE id = ?1")?
            .query_row([self.1], |row| row.get("name"))
    }

    /// Internal ID of the user who created the room.
    pub fn author_id(&self) -> rusqlite::Result<i64> {
        self.0.lock()
            .prepare_cached("SELECT author_id FROM public_rooms WHERE id = ?1")?
            .query_row([self.1], |row| row.get("author_id"))
    }

    /// Hash of the block where this record is stored.
    pub fn block_hash(&self) -> rusqlite::Result<Hash> {
        self.0.lock()
            .prepare_cached("SELECT block_hash FROM public_rooms WHERE id = ?1")?
            .query_row([self.1], |row| row.get::<_, [u8; 32]>("block_hash"))
            .map(Hash::from)
    }

    /// Hash of the transaction where this record is stored.
    pub fn transaction_hash(&self) -> rusqlite::Result<Hash> {
        self.0.lock()
            .prepare_cached("SELECT transaction_hash FROM public_rooms WHERE id = ?1")?
            .query_row([self.1], |row| row.get::<_, [u8; 32]>("transaction_hash"))
            .map(Hash::from)
    }

    /// Update name of the current room.
    pub fn update_name(
        &mut self,
        name: impl AsRef<str>
    ) -> rusqlite::Result<&mut Self> {
        self.0.lock()
            .prepare_cached("UPDATE public_rooms SET name = ?2 WHERE id = ?1")?
            .execute((self.1, name.as_ref()))?;

        Ok(self)
    }
}
