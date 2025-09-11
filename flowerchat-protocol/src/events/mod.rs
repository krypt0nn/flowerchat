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

pub mod rooms;

pub mod prelude {
    pub use super::rooms::prelude::*;
}

use prelude::*;

pub trait Event {
    type Error: std::error::Error;

    /// Serialize current event into the provided write buffer.
    fn serialize(&self, out_buf: &mut impl Write) -> Result<(), Self::Error>;

    /// Deserialize event from the given bytes buffer.
    fn deserialize(
        bytes: &mut impl Read
    ) -> Result<Self, Self::Error> where Self: Sized;
}

#[derive(Debug, thiserror::Error)]
pub enum EventsError {
    #[error("failed to read or write bytes: {0}")]
    Io(#[from] std::io::Error),

    #[error("unknown event id: {0}")]
    UnknownEventId(u8),

    #[error(transparent)]
    CreatePublicRoom(#[from] CreatePublicRoomEventError),

    #[error(transparent)]
    PublicRoomMessage(#[from] PublicRoomMessageEventError)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Events {
    CreatePublicRoom(CreatePublicRoomEvent),
    PublicRoomMessage(PublicRoomMessageEvent)
}

impl Events {
    pub const V1_CREATE_PUBLIC_ROOM: u8  = 0;
    pub const V1_PUBLIC_ROOM_MESSAGE: u8 = 1;
}

impl Event for Events {
    type Error = EventsError;

    fn serialize(&self, out_buf: &mut impl Write) -> Result<(), Self::Error> {
        match self {
            Self::CreatePublicRoom(event) => {
                out_buf.write_all(&[Self::V1_CREATE_PUBLIC_ROOM])?;

                event.serialize(out_buf)?;
            }

            Self::PublicRoomMessage(event) => {
                out_buf.write_all(&[Self::V1_PUBLIC_ROOM_MESSAGE])?;

                event.serialize(out_buf)?;
            }
        }

        Ok(())
    }

    fn deserialize(
        bytes: &mut impl Read
    ) -> Result<Self, Self::Error> where Self: Sized {
        let mut event_id = [0; 1];

        bytes.read_exact(&mut event_id)?;

        match event_id[0] {
            Self::V1_CREATE_PUBLIC_ROOM => {
                let event = CreatePublicRoomEvent::deserialize(bytes)?;

                Ok(Self::from(event))
            }

            Self::V1_PUBLIC_ROOM_MESSAGE => {
                let event = PublicRoomMessageEvent::deserialize(bytes)?;

                Ok(Self::from(event))
            }

            _ => Err(EventsError::UnknownEventId(event_id[0]))
        }
    }
}

impl From<CreatePublicRoomEvent> for Events {
    #[inline(always)]
    fn from(value: CreatePublicRoomEvent) -> Self {
        Self::CreatePublicRoom(value)
    }
}

impl From<PublicRoomMessageEvent> for Events {
    #[inline(always)]
    fn from(value: PublicRoomMessageEvent) -> Self {
        Self::PublicRoomMessage(value)
    }
}
