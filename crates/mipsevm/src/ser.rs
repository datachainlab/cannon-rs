//! Serialization utilities for the `cannon-mipsevm` crate.

use std::io::{Error, Read, Write};
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;

/// Generates a hex string serialization module for a fixed-size byte array.
macro_rules! fixed_hex_ser {
    ($module_name:ident, $size:expr) => {
        pub mod $module_name {
            use alloy_primitives::hex;
            use serde::{self, Deserialize, Deserializer, Serializer};

            pub fn serialize<S>(bytes: &[u8; $size], serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(&format!("0x{}", hex::encode(bytes)))
            }

            pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; $size], D::Error>
            where
                D: Deserializer<'de>,
            {
                let s = String::deserialize(deserializer)?;
                hex::decode(s)
                    .map_err(serde::de::Error::custom)
                    .map(|bytes| {
                        let mut array = [0u8; $size];
                        array.copy_from_slice(&bytes);
                        array
                    })
            }
        }
    };
}

fixed_hex_ser!(fixed_32_hex, 32);
fixed_hex_ser!(page_hex, crate::page::PAGE_SIZE);
fixed_hex_ser!(state_witness_hex, crate::witness::STATE_WITNESS_SIZE);

pub mod vec_u8_hex {
    use alloy_primitives::hex;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("0x{}", hex::encode(bytes)))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        hex::decode(s).map_err(serde::de::Error::custom)
    }
}

macro_rules! fixed_base64_ser {
    ($module_name:ident, $size:expr) => {
        pub mod $module_name {
            use serde::{self, Deserialize, Deserializer, Serializer};
            use base64::prelude::BASE64_STANDARD;
            use base64::Engine;

            use crate::ser::{compress_bytes, decompress_bytes};

            pub fn serialize<S>(bytes: &[u8; $size], serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                 let encoded = BASE64_STANDARD.encode(bytes);
                 let encoded = compress_bytes(encoded.as_bytes()).map_err(serde::ser::Error::custom)?;
                 serializer.serialize_bytes(&encoded)
            }

            pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; $size], D::Error>
            where
                D: Deserializer<'de>,
            {
                let s = String::deserialize(deserializer)?;
                let decoded = BASE64_STANDARD.decode(s).map_err(serde::de::Error::custom)?;
                decompress_bytes(&decoded)
                    .map_err(serde::de::Error::custom)
                    .map(|bytes| {
                        let mut array = [0u8; $size];
                        array.copy_from_slice(&bytes);
                        array
                    })
            }
        }
    };
}

fixed_base64_ser!(fixed_32_base64, 32);
fixed_base64_ser!(page_base64, crate::page::PAGE_SIZE);
fixed_base64_ser!(state_witness_base64, crate::witness::STATE_WITNESS_SIZE);

pub fn decompress_bytes(compressed_bytes: &[u8]) -> Result<Vec<u8>, Error> {
    let mut decoder = ZlibDecoder::new(compressed_bytes);
    let mut decompressed_bytes = Vec::with_capacity(compressed_bytes.len());
    decoder.read_to_end(&mut decompressed_bytes)?;

    Ok(decompressed_bytes)
}

pub fn compress_bytes(decompressed_bytes: &[u8]) -> Result<Vec<u8>, Error> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(decompressed_bytes)?;
    Ok(encoder.finish()?)
}