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

use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::{RngCore, SeedableRng};

/// Get sustainably random number generator.
pub fn get_rng() -> ChaCha20Rng {
    // Seed rng using both system-provided entropy and current time.
    // This is needed because some systems can fallback to zero-filled
    // entropy.
    let mut rng = ChaCha20Rng::from_entropy();
    let mut seed = [0; 32];

    rng.fill_bytes(&mut seed);

    let current_time = time::UtcDateTime::now()
        .unix_timestamp()
        .to_le_bytes();

    for i in 0..32 {
        seed[i] ^= current_time[i % 8];
    }

    ChaCha20Rng::from_seed(seed)
}

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
