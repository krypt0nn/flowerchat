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

use std::iter::FusedIterator;

use libflowerpot::crypto::*;

use crate::utils::*;

use super::Database;
use super::public_room::PublicRoomRecord;

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

    /// Open existing space from its root block hash. Return `None` if such
    /// space doesn't exist.
    pub fn find(
        database: Database,
        root_block: &Hash
    ) -> rusqlite::Result<Option<Self>> {
        let id = database.lock()
            .prepare_cached("SELECT id FROM spaces WHERE root_block = ?1")?
            .query_row([root_block.0], |row| row.get("id"));

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

    /// List of current space shards.
    pub fn shards(&self) -> rusqlite::Result<Vec<String>> {
        let lock = self.0.lock();

        let mut query = lock.prepare_cached(
            "SELECT address FROM shards WHERE space_id = ?1"
        )?;

        let mut shards = Vec::new();

        for address in query.query_map([self.1], |row| row.get("address"))? {
            shards.push(address?);
        }

        Ok(shards)
    }

    /// Add shard address to the current space.
    pub fn add_shard(&self, address: impl AsRef<str>) -> rusqlite::Result<()> {
        let lock = self.0.lock();

        let mut query = lock.prepare_cached(
            "INSERT OR IGNORE INTO shards (space_id, address) VALUES (?1, ?2)"
        )?;

        query.execute((self.1, address.as_ref()))?;

        Ok(())
    }

    /// Get iterator of all the public rooms existing in the current space.
    #[inline]
    pub fn public_rooms(&self) -> PublicRoomsIter {
        PublicRoomsIter {
            database: self.0.clone(),
            space_id: self.1,
            current: 0
        }
    }

    fn get_space_slice(&self) -> rusqlite::Result<[u8; 65]> {
        let root_block = self.root_block()?;
        let author = self.author()?.to_bytes();

        let mut slice = [0; 65];

        slice[..32].copy_from_slice(&root_block.0);
        slice[32..].copy_from_slice(&author);

        Ok(slice)
    }

    /// Get emoji representing the current space.
    pub fn emoji(&self) -> rusqlite::Result<&'static str> {
        Ok(bytes_to_emoji(self.get_space_slice()?))
    }

    /// Get shortname representation of the current space.
    pub fn shortname(&self) -> rusqlite::Result<String> {
        Ok(bytes_to_shortname(self.get_space_slice()?))
    }
}

pub struct PublicRoomsIter {
    database: Database,
    space_id: i64,
    current: i64
}

impl Iterator for PublicRoomsIter {
    type Item = PublicRoomRecord;

    fn next(&mut self) -> Option<Self::Item> {
        let lock = self.database.lock();

        let mut query = lock.prepare_cached("
            SELECT id FROM public_rooms
            WHERE space_id = ?1 AND id > ?2
            ORDER BY id ASC
            LIMIT 1
        ").ok()?;

        let id = query.query_row(
            [self.space_id, self.current],
            |row| row.get("id")
        ).ok()?;

        self.current = id;

        let record = PublicRoomRecord::open_raw(
            self.database.clone(),
            id
        );

        Some(record)
    }
}

impl FusedIterator for PublicRoomsIter {}
