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

use std::str::FromStr;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Role {
    #[default]
    User,
    Moderator,
    Administrator,
    Owner
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::User          => f.write_str("user"),
            Self::Moderator     => f.write_str("moderator"),
            Self::Administrator => f.write_str("administrator"),
            Self::Owner         => f.write_str("owner")
        }
    }
}

impl FromStr for Role {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "user"          | "member"             => Ok(Self::User),
            "moderator"     | "mod"     | "moder"  => Ok(Self::Moderator),
            "administrator" | "admin"              => Ok(Self::Administrator),
            "owner"         | "creator" | "author" => Ok(Self::Owner),

            _ => Err(s.to_string())
        }
    }
}

#[test]
fn test_roles() {
    const ROLES: &[Role] = &[
        Role::User,
        Role::Moderator,
        Role::Administrator,
        Role::Owner
    ];

    for i in 1..ROLES.len() {
        assert!(ROLES[i] > ROLES[i - 1]);
    }

    for role in ROLES {
        assert_eq!(Role::from_str(&role.to_string()), Ok(*role));
    }
}
