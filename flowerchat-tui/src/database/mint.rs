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

// TODO

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MintInfo {
    /// Internal ID of the user.
    pub user_id: i64,

    /// Unique nonce bytes slice.
    pub nonce: Box<[u8]>,

    /// Hash of the block where this mint record is stored.
    pub block_hash: Hash,

    /// Hash of the transaction where this mint record is stored.
    pub transaction_hash: Hash
}
