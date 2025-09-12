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

use libflowerpot::crypto::*;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("zstd error: {0}")]
    Zstd(#[source] std::io::Error),

    #[error("invalid base64 format")]
    Base64,

    #[error("invalid space sharing link format: {0}")]
    InvalidFormat(u8),

    #[error("invalid public key format")]
    InvalidPublicKey
}

/// Standard format of sharing space with other people. This link contains
/// hash of the root block of the space's blockchain, public key of its creator
/// and list of shards for pool bootstrapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShareLink {
    root_block: Hash,
    public_key: PublicKey,
    shards: Box<[String]>
}

impl ShareLink {
    /// Create new space sharing link.
    pub fn new<T: ToString>(
        root_block: impl Into<Hash>,
        public_key: impl Into<PublicKey>,
        shards: impl IntoIterator<Item = T>
    ) -> Self {
        Self {
            root_block: root_block.into(),
            public_key: public_key.into(),
            shards: shards.into_iter()
                .map(|address| address.to_string())
                .collect()
        }
    }

    /// Get root block's hash for the current space.
    pub const fn root_block(&self) -> &Hash {
        &self.root_block
    }

    /// Get public key of the current space blockchain's creator.
    pub const fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    /// Get list of bootstrap shards for the current space.
    pub const fn shards(&self) -> &[String] {
        &self.shards
    }

    /// Serialize current space sharing link to bytes.
    pub fn to_bytes(&self) -> Result<Box<[u8]>, Error> {
        let mut link = Vec::new();

        link.extend_from_slice(&self.root_block.0);
        link.extend_from_slice(&self.public_key.to_bytes());

        for address in &self.shards {
            let len = address.len();

            if len <= u16::MAX as usize {
                link.extend_from_slice(&(len as u16).to_le_bytes());
                link.extend_from_slice(address.as_bytes());
            }
        }

        let mut compressed_link = vec![0];

        let link = zstd::encode_all(&mut link.as_slice(), 20)
            .map_err(Error::Zstd)?;

        compressed_link.extend(link);

        Ok(compressed_link.into_boxed_slice())
    }

    /// Deserialize space sharing link from bytes.
    pub fn from_bytes(bytes: impl AsRef<[u8]>) -> Result<Self, Error> {
        let bytes = bytes.as_ref();

        if bytes[0] != 0 {
            return Err(Error::InvalidFormat(bytes[0]));
        }

        let bytes = zstd::decode_all(&mut &bytes[1..]).map_err(Error::Zstd)?;

        let mut root_block = [0; 32];
        let mut public_key = [0; 33];

        root_block.copy_from_slice(&bytes[0..32]);
        public_key.copy_from_slice(&bytes[32..65]);

        let len = bytes.len();
        let mut i = 65;

        let mut shards = Vec::new();

        while i < len {
            let mut address_len = [0; 2];

            address_len.copy_from_slice(&bytes[i..i + 2]);

            let address_len = u16::from_le_bytes(address_len) as usize;

            let mut address = vec![0; address_len];

            address.copy_from_slice(&bytes[i + 2..i + 2 + address_len]);

            shards.push(String::from_utf8_lossy(&address).to_string());

            i += address_len + 2;
        }

        Ok(Self {
            root_block: Hash::from(root_block),
            public_key: PublicKey::from_bytes(public_key)
                .ok_or(Error::InvalidPublicKey)?,
            shards: shards.into_boxed_slice()
        })
    }

    /// Serialize current link to base64 string.
    pub fn to_base64(&self) -> Result<String, Error> {
        Ok(base64_encode(self.to_bytes()?))
    }

    /// Deserialize base64 string to a space sharing link.
    pub fn from_base64(link: impl AsRef<[u8]>) -> Result<Self, Error> {
        let link = base64_decode(link).map_err(|_| Error::Base64)?;

        Self::from_bytes(link)
    }
}

#[test]
fn test_serialize() -> Result<(), Error> {
    use rand_chacha::ChaCha20Rng;
    use rand_chacha::rand_core::SeedableRng;

    let mut rng = ChaCha20Rng::seed_from_u64(123);

    let secret_key = SecretKey::random(&mut rng);

    let link = ShareLink::new(
        Hash::default(),
        secret_key.public_key(),
        [
            String::from("test 1"),
            String::from("test 2"),
            String::from("test 3"),
            String::from("test 4"),
            String::from("test 5")
        ]
    );

    assert_eq!(link, ShareLink::from_bytes(link.to_bytes()?)?);
    assert_eq!(link, ShareLink::from_base64(link.to_base64()?)?);

    Ok(())
}
