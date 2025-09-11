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

/// Newtype for a valid public room message string.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RoomMessage(String);

impl RoomMessage {
    /// Create new public room message using provided string.
    ///
    /// This function will return `None` if provided message has invalid format.
    pub fn new(content: impl AsRef<str>) -> Option<Self> {
        let content = content.as_ref()
            .trim()
            .to_string();

        if !(1..=1024).contains(&content.len()) {
            return None;
        }

        // TODO: more restrictions
        if content.chars().any(|c| c.is_ascii_control()) {
            return None;
        }

        Some(Self(content))
    }
}

impl AsRef<str> for RoomMessage {
    #[inline(always)]
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for RoomMessage {
    type Target = String;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<RoomMessage> for String {
    #[inline(always)]
    fn from(value: RoomMessage) -> Self {
        value.0
    }
}

#[test]
fn test() {
    assert!(RoomMessage::new("a").is_some());
    assert!(RoomMessage::new("\0").is_none());
    assert!(RoomMessage::new("a".repeat(1025)).is_none());
}
