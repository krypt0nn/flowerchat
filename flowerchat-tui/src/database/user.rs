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
pub struct UserInfo {
    /// Internal ID of the space this user belongs to.
    pub space_id: i64,

    /// Public key of the user.
    pub public_key: PublicKey,

    /// Nickname of the user if it's available.
    pub nickname: Option<String>
}

#[derive(Debug, Clone)]
pub struct UserRecord(Database, i64);

impl UserRecord {
    /// Create new user record.
    pub fn create(
        database: Database,
        info: &UserInfo
    ) -> rusqlite::Result<Self> {
        let lock = database.lock();

        let mut query = lock.prepare_cached("
            INSERT INTO users (
                space_id,
                public_key,
                nickname
            ) VALUES (?1, ?2, ?3)
        ")?;

        let id = query.insert((
            info.space_id,
            info.public_key.to_bytes(),
            info.nickname.as_ref()
        ))?;

        drop(query);
        drop(lock);

        Ok(Self(database, id))
    }

    /// Open user without verifying its existance.
    #[inline(always)]
    pub fn open_raw(database: Database, id: i64) -> Self {
        Self(database, id)
    }

    /// Open existing user from its ID.
    pub fn open(
        database: Database,
        id: i64
    ) -> rusqlite::Result<Self> {
        database.lock()
            .prepare_cached("SELECT 1 FROM users WHERE id = ?1")?
            .query_row([id], |_| Ok(()))?;

        Ok(Self(database, id))
    }

    /// Find existing user from its space ID and public key. Return `None` if
    /// such user doesn't exist.
    pub fn find(
        database: Database,
        space_id: i64,
        public_key: &PublicKey
    ) -> rusqlite::Result<Option<Self>> {
        let lock = database.lock();

        let mut query = lock.prepare_cached("
            SELECT id FROM users WHERE space_id = ?1 AND public_key = ?2
        ")?;

        let id = query.query_row((
            space_id, public_key.to_bytes()
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

    /// Internal ID of the user.
    #[inline(always)]
    pub const fn id(&self) -> i64 {
        self.1
    }

    #[inline]
    pub fn into_inner(self) -> (Database, i64) {
        (self.0, self.1)
    }

    /// Internal ID of the space this user belongs to.
    pub fn space_id(&self) -> rusqlite::Result<i64> {
        self.0.lock()
            .prepare_cached("SELECT space_id FROM users WHERE id = ?1")?
            .query_row([self.1], |row| row.get("space_id"))
    }

    /// Public key of the user.
    pub fn public_key(&self) -> rusqlite::Result<PublicKey> {
        self.0.lock()
            .prepare_cached("SELECT public_key FROM users WHERE id = ?1")?
            .query_row([self.1], |row| row.get::<_, [u8; 33]>("public_key"))
            .and_then(|public_key| {
                // TODO: better error handling?
                PublicKey::from_bytes(public_key)
                    .ok_or_else(|| rusqlite::Error::InvalidQuery)
            })
    }

    /// Nickname of the user if it's available.
    pub fn nickname(&self) -> rusqlite::Result<Option<String>> {
        self.0.lock()
            .prepare_cached("SELECT nickname FROM users WHERE id = ?1")?
            .query_row([self.1], |row| row.get("nickname"))
    }

    /// Update nickname of the current space.
    pub fn update_nickname(
        &mut self,
        nickname: impl AsRef<str>
    ) -> rusqlite::Result<&mut Self> {
        self.0.lock()
            .prepare_cached("UPDATE users SET nickname = ?2 WHERE id = ?1")?
            .execute((self.1, nickname.as_ref()))?;

        Ok(self)
    }
}
