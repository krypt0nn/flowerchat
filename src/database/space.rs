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

// use std::io::{Cursor, Read};

// use varint_rs::{VarintReader, VarintWriter};

use libflowerpot::crypto::*;

use crate::utils::*;

use super::Database;

// const SPACE_SHARE_LINK_COMPRESSION_LEVEL: i32 = 20;

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub struct SpaceShareLink {
//     pub title: String,
//     pub root_block: Hash,
//     pub author: PublicKey,
//     pub shards: Vec<String>
// }

// impl SpaceShareLink {
//     pub fn to_base64(&self) -> anyhow::Result<String> {
//         let mut bytes = Vec::new();

//         bytes.push(0); // Format version
//         bytes.write_u32_varint(self.title.len() as u32)?;
//         bytes.extend_from_slice(self.title.as_bytes());
//         bytes.extend_from_slice(&self.root_block.0);
//         bytes.extend_from_slice(&self.author.to_bytes());

//         for address in &self.shards {
//             bytes.write_u32_varint(address.len() as u32)?;
//             bytes.extend_from_slice(address.as_bytes());
//         }

//         let bytes = zstd::encode_all(
//             Cursor::new(bytes),
//             SPACE_SHARE_LINK_COMPRESSION_LEVEL
//         )?;

//         Ok(base64_encode(bytes))
//     }

//     pub fn from_base64(link: impl AsRef<[u8]>) -> anyhow::Result<Self> {
//         let bytes = base64_decode(link)?;
//         let bytes = zstd::decode_all(bytes.as_slice())?;

//         if bytes.len() < 66 {
//             anyhow::bail!("space share link length must be at least 66 bytes long");
//         }

//         if bytes[0] != 0 {
//             anyhow::bail!("unknown space share link format");
//         }

//         let mut bytes = Cursor::new(bytes);

//         let title_len = bytes.read_u32_varint()? as usize;

//         let mut title = vec![0; title_len];
//         let mut root_block = [0; 32];
//         let mut author = [0; 33];

//         bytes.read_exact(&mut title)?;
//         bytes.read_exact(&mut root_block)?;
//         bytes.read_exact(&mut author)?;

//         // TODO

//         Ok(Self {
//             title: String::from_utf8_lossy(&title)
//                 .to_string(),
//             root_block: Hash::from(root_block),
//             author: PublicKey::from_bytes(author)?,
//             shards: vec![]
//         })
//     }
// }

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
