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

/// Cast bytes slice into a unicode emoji.
pub fn bytes_to_emoji(bytes: impl AsRef<[u8]>) -> &'static str {
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

    let hash = crc32fast::hash(bytes.as_ref());

    EMOJIS[(hash % EMOJIS.len() as u32) as usize]
}

/// Cast bytes slice into a short name.
pub fn bytes_to_shortname(bytes: impl AsRef<[u8]>) -> String {
    const CHARS: &[char] = &[
        'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J',
        'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T',
        'U', 'V', 'W', 'X', 'Y', 'Z', '0', '1', '2', '3',
        '4', '5', '6', '7', '8', '9'
    ];

    let hash = crc32fast::hash(bytes.as_ref())
        .to_le_bytes();

    let n = CHARS.len();

    let mut name = String::with_capacity(4);

    name.push(CHARS[hash[0] as usize % n]);
    name.push(CHARS[hash[1] as usize % n]);
    name.push(CHARS[hash[2] as usize % n]);
    name.push(CHARS[hash[3] as usize % n]);

    name
}
