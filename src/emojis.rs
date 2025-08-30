// SPDX-License-Identifier: GPL-3.0-only
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

// TODO: review these emojis

const EMOJIS: &[&str] = &[
    // Food & Drink
    "🍇", "🍈", "🍉", "🍊", "🍋", "🍌", "🍍", "🥭", "🍎", "🍏",
    "🍐", "🍑", "🍒", "🍓", "🥝", "🍅", "🥥", "🥑", "🍆", "🥔",
    "🥕", "🌽", "🌶️", "🥒", "🥬", "🥦", "🧄", "🧅", "🥜", "🌰",
    "🍞", "🥐", "🥖", "🥨", "🥯", "🥞", "🧇", "🧀", "🍖", "🍗",
    "🥩", "🥓", "🍔", "🍟", "🍕", "🌭", "🥪", "🌮", "🌯", "🥙",
    "🧆", "🥚", "🍳", "🥘", "🍲", "🥣", "🥗", "🍿", "🧈", "🧂",
    "🥫", "🍱", "🍘", "🍙", "🍚", "🍛", "🍜", "🍝", "🍠", "🍢",
    "🍣", "🍤", "🍥", "🥮", "🍡", "🥟", "🥠", "🥡", "🍦", "🍧",
    "🍨", "🍩", "🍪", "🎂", "🍰", "🧁", "🥧", "🍫", "🍬", "🍭",
    "🍮", "🍯", "🍺", "🍷", "🍸", "🍹", "🧉",

    // Plants & Flowers
    "🌸", "🏵️", "🌼", "🌷", "🌹", "🥀", "🌺", "🌻", "🌵", "🌲",
    "🌳", "🌴", "🌿", "🍀", "🍁", "🍂", "🌾", "💐", "🌰", "🎋",
    "🌱", "🍄",

    // Animals
    "🐶", "🐹", "🐰", "🦊", "🐻", "🐼", "🐨", "🐯", "🦁", "🐸",
    "🦝", "🐺", "🐧", "🐤", "🦆", "🦅", "🦉", "🦇", "🐴", "🦄",
    "🐝", "🐛", "🦋", "🐌", "🦂", "🐢", "🐍", "🦎", "🦖", "🦕",
    "🐙", "🦐", "🦞", "🦀", "🐡", "🐠", "🐟", "🐬", "🐋", "🦈",
    "🐊", "🦓", "🦍", "🐘", "🦛", "🦏", "🐫", "🦒", "🦘", "🦬",
    "🐃", "🐄", "🐎", "🐑", "🦙", "🐐", "🦜", "🦢", "🦩", "🐇",
    "🦨", "🦫", "🦦"
];

/// Cast bytes slice into a unicode emoji.
pub fn bytes_to_emoji(bytes: impl AsRef<[u8]>) -> &'static str {
    // FNV-1a hash
    // Source: https://en.wikipedia.org/wiki/Fowler%E2%80%93Noll%E2%80%93Vo_hash_function
    let mut hash = 0xcbf29ce484222325;

    for byte in bytes.as_ref() {
        hash = (hash ^ (*byte as u64)).wrapping_mul(0x100000001b3);
    }

    EMOJIS[(hash % EMOJIS.len() as u64) as usize]
}
