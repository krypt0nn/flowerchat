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
use crate::types::room_message::RoomMessage;
use crate::events::Event;

#[derive(Debug, thiserror::Error)]
pub enum PublicRoomMessageEventError {
    #[error("failed to read or write bytes: {0}")]
    Io(#[from] std::io::Error),

    #[error("failed to compress/decompress zstd stream: {0}")]
    Zstd(#[source] std::io::Error),

    #[error("room name is invalid: '{0}'")]
    InvalidRoomName(String),

    #[error("message content is invalid: '{0}'")]
    InvalidContent(String)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PublicRoomMessageEvent {
    room_name: RoomName,
    content: RoomMessage
}

impl PublicRoomMessageEvent {
    /// Create new public room message event from provided room name and content
    /// strings.
    ///
    /// This function will return `None` if provided strings have invalid
    /// format.
    pub fn new(
        room_name: impl AsRef<str>,
        content: impl AsRef<str>
    ) -> Option<Self> {
        Some(Self {
            room_name: RoomName::new(room_name)?,
            content: RoomMessage::new(content)?
        })
    }

    /// Create new public room message event from provided name and content
    /// newtypes.
    #[inline]
    pub const fn new_from(
        room_name: RoomName,
        content: RoomMessage
    ) -> Self {
        Self {
            room_name,
            content
        }
    }

    #[inline]
    pub fn room_name(&self) -> &str {
        &self.room_name
    }

    #[inline]
    pub fn content(&self) -> &str {
        &self.content
    }
}

impl Event for PublicRoomMessageEvent {
    type Error = PublicRoomMessageEventError;

    fn serialize(&self, out_buf: &mut impl Write) -> Result<(), Self::Error> {
        let room_name = zstd::encode_all(self.room_name.as_bytes(), 20)
            .map_err(PublicRoomMessageEventError::Zstd)?;

        out_buf.write_all(&[room_name.len() as u8])
            .map_err(PublicRoomMessageEventError::Io)?;

        out_buf.write_all(&room_name)
            .map_err(PublicRoomMessageEventError::Io)?;

        let content = zstd::encode_all(self.content.as_bytes(), 20)
            .map_err(PublicRoomMessageEventError::Zstd)?;

        out_buf.write_all(&(content.len() as u16).to_le_bytes())
            .map_err(PublicRoomMessageEventError::Io)?;

        out_buf.write_all(&content)
            .map_err(PublicRoomMessageEventError::Io)?;

        Ok(())
    }

    fn deserialize(
        bytes: &mut impl Read
    ) -> Result<Self, Self::Error> where Self: Sized {
        let mut room_name_len = [0; 1];
        let mut content_len = [0; 2];

        bytes.read_exact(&mut room_name_len)
            .map_err(PublicRoomMessageEventError::Io)?;

        let mut room_name = vec![0; room_name_len[0] as usize];

        bytes.read_exact(&mut room_name)
            .map_err(PublicRoomMessageEventError::Io)?;

        let room_name = zstd::decode_all(room_name.as_slice())
            .map_err(PublicRoomMessageEventError::Zstd)?;

        let room_name = String::from_utf8_lossy(&room_name)
            .to_string();

        let Some(room_name) = RoomName::new(&room_name) else {
            return Err(PublicRoomMessageEventError::InvalidRoomName(room_name));
        };

        bytes.read_exact(&mut content_len)
            .map_err(PublicRoomMessageEventError::Io)?;

        let mut content = vec![0; u16::from_le_bytes(content_len) as usize];

        bytes.read_exact(&mut content)
            .map_err(PublicRoomMessageEventError::Io)?;

        let content = zstd::decode_all(content.as_slice())
            .map_err(PublicRoomMessageEventError::Zstd)?;

        let content = String::from_utf8_lossy(&content)
            .to_string();

        let Some(content) = RoomMessage::new(&content) else {
            return Err(PublicRoomMessageEventError::InvalidContent(content));
        };

        Ok(Self::new_from(room_name, content))
    }
}

#[test]
fn test_serialize() -> Result<(), PublicRoomMessageEventError> {
    let event = PublicRoomMessageEvent::new("some-channel", "Hello, World!")
        .expect("failed to create public message event");

    let mut buf = Vec::new();

    event.serialize(&mut buf)?;

    assert_eq!(PublicRoomMessageEvent::deserialize(&mut buf.as_slice())?, event);

    Ok(())
}
