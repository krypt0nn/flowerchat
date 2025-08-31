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

use std::path::PathBuf;

lazy_static::lazy_static! {
    /// Path to the flowerchat's data folder. Takes one of the following values
    /// in the corresponding priority order.
    ///
    /// - `$FLOWERCHAT_DATA_FOLDER`.
    /// - `$XDG_DATA_HOME/flowerchat`.
    /// - `$HOME/.local/share/flowerchat`.
    /// - `<current directory>/flowerchat`.
    pub static ref DATA_FOLDER: PathBuf = std::env::var("FLOWERCHAT_DATA_FOLDER")
        .map(PathBuf::from)
        .or_else(|_| {
            std::env::var("XDG_DATA_HOME")
                .map(|path| PathBuf::from(path).join("flowerchat"))
        })
        .or_else(|_| {
            std::env::var("HOME")
                .map(|path| {
                    PathBuf::from(path)
                        .join(".local")
                        .join("share")
                        .join("flowerchat")
                })
        })
        .map_err(std::io::Error::other)
        .and_then(|_| {
            std::env::current_dir()
                .map(|path| path.join("flowerchat"))
        })
        .expect("failed to choose the data folder path");

    /// Path to the flowerchat database file: `DATA_FOLDER/flowerchat.db`.
    pub static ref DATABASE_PATH: PathBuf = DATA_FOLDER.join("flowerchat.db");

    /// Path to the flowerchat identities file: `DATA_FOLDER/identities.json`.
    pub static ref IDENTITIES_PATH: PathBuf = DATA_FOLDER.join("identities.json");
}
