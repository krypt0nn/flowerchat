// SPDX-License-Identifier: GPL-3.0-or-later
//
// flowerchat-protocol
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

use std::io::{Read, Write};

use crate::types::room_name::RoomName;
use crate::events::Event;

#[derive(Debug, thiserror::Error)]
pub enum CreatePublicRoomEventError {
    #[error("failed to read or write bytes: {0}")]
    Io(#[source] std::io::Error),

    #[error("failed to compress/decompress zstd stream: {0}")]
    Zstd(#[source] std::io::Error),

    #[error("room name is invalid: '{0}'")]
    InvalidName(String)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CreatePublicRoomEvent(RoomName);

impl CreatePublicRoomEvent {
    /// Create new public room event using provided unique name.
    ///
    /// This function will return `None` if provided name has invalid format.
    #[inline]
    pub fn new(name: impl AsRef<str>) -> Option<Self> {
        Some(Self(RoomName::new(name)?))
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.0
    }
}

impl Event for CreatePublicRoomEvent {
    type Error = CreatePublicRoomEventError;

    fn serialize(&self, out_buf: &mut impl Write) -> Result<(), Self::Error> {
        let name = zstd::encode_all(self.0.as_bytes(), 20)
            .map_err(CreatePublicRoomEventError::Zstd)?;

        out_buf.write_all(&[name.len() as u8])
            .map_err(CreatePublicRoomEventError::Io)?;

        out_buf.write_all(&name)
            .map_err(CreatePublicRoomEventError::Io)?;

        Ok(())
    }

    fn deserialize(
        bytes: &mut impl Read
    ) -> Result<Self, Self::Error> where Self: Sized {
        let mut len = [0; 1];

        bytes.read_exact(&mut len)
            .map_err(CreatePublicRoomEventError::Io)?;

        let mut name = vec![0; len[0] as usize];

        bytes.read_exact(&mut name)
            .map_err(CreatePublicRoomEventError::Io)?;

        let name = zstd::decode_all(name.as_slice())
            .map_err(CreatePublicRoomEventError::Zstd)?;

        let name = String::from_utf8_lossy(&name)
            .to_string();

        match Self::new(&name) {
            Some(event) => Ok(event),
            None => Err(CreatePublicRoomEventError::InvalidName(name))
        }
    }
}

impl From<RoomName> for CreatePublicRoomEvent {
    #[inline(always)]
    fn from(value: RoomName) -> Self {
        CreatePublicRoomEvent(value)
    }
}

#[test]
fn test_serialize() -> Result<(), CreatePublicRoomEventError> {
    let event = CreatePublicRoomEvent::new("hello-world")
        .expect("failed to create public room event");

    let mut buf = Vec::new();

    event.serialize(&mut buf)?;

    assert_eq!(CreatePublicRoomEvent::deserialize(&mut buf.as_slice())?, event);

    Ok(())
}
