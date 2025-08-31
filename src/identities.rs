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

use anyhow::Context;
use time::UtcDateTime;
use serde_json::{json, Value as Json};

use libflowerpot::crypto::*;

use crate::consts::IDENTITIES_PATH;
use crate::utils::{bytes_to_emoji, bytes_to_shortname};

/// Read identities list from the data folder.
pub fn read() -> anyhow::Result<Vec<Identity>> {
    if !IDENTITIES_PATH.exists() {
        return Ok(vec![]);
    }

    let identities = std::fs::read(IDENTITIES_PATH.as_path())?;
    let identities = serde_json::from_slice::<Vec<Json>>(&identities)?;

    let mut identities = identities.into_iter()
        .map(|identity| {
            Identity::from_json(&identity)
                .context("failed to read identities list")
        })
        .collect::<Result<Vec<_>, _>>()?;

    identities.dedup_by(|a, b| a.secret_key() == b.secret_key());

    Ok(identities)
}

/// Write identities list to the data folder.
pub fn write(
    identities: impl IntoIterator<Item = Identity>
) -> anyhow::Result<()> {
    let identities = identities.into_iter()
        .map(|identity| identity.to_json())
        .collect::<Vec<_>>();

    std::fs::write(
        IDENTITIES_PATH.as_path(),
        serde_json::to_vec_pretty(&json!(identities))?
    )?;

    Ok(())
}

/// Identity is a cross-space profile which can be used by the user. It has a
/// user-defined title for easier navigation and a secret key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity {
    title: String,
    secret_key: SecretKey,
    created_at: UtcDateTime
}

impl Identity {
    pub fn new(
        title: impl ToString,
        secret_key: impl Into<SecretKey>
    ) -> Self {
        Self {
            title: title.to_string(),
            secret_key: secret_key.into(),
            created_at: UtcDateTime::now()
        }
    }

    #[inline(always)]
    pub const fn title(&self) -> &String {
        &self.title
    }

    #[inline(always)]
    pub const fn secret_key(&self) -> &SecretKey {
        &self.secret_key
    }

    #[inline(always)]
    pub const fn created_at(&self) -> &UtcDateTime {
        &self.created_at
    }

    /// Get emoji representing the current identity.
    #[inline]
    pub fn emoji(&self) -> &'static str {
        bytes_to_emoji(self.secret_key.to_bytes())
    }

    /// Get shortname representation of the current identity.
    #[inline]
    pub fn shortname(&self) -> String {
        bytes_to_shortname(self.secret_key.to_bytes())
    }

    pub fn to_json(&self) -> Json {
        json!({
            "title": self.title.as_str(),
            "secret_key": self.secret_key.to_base64(),
            "created_at": self.created_at.unix_timestamp()
        })
    }

    pub fn from_json(json: &Json) -> anyhow::Result<Self> {
        Ok(Self {
            title: json.get("title")
                .and_then(Json::as_str)
                .map(String::from)
                .ok_or_else(|| anyhow::anyhow!("identity field 'title' is missing"))?,

            secret_key: json.get("secret_key")
                .and_then(Json::as_str)
                .and_then(SecretKey::from_base64)
                .ok_or_else(|| anyhow::anyhow!("identity field 'secret_key' is invalid"))?,

            created_at: json.get("created_at")
                .and_then(Json::as_i64)
                .map(UtcDateTime::from_unix_timestamp)
                .ok_or_else(|| anyhow::anyhow!("identity field 'created_at' is missing"))??
        })
    }
}
