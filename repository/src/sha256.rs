use errors::*;
use hex::FromHex;
use hex_slice::HexSlice;
use ring::digest;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{Error, Visitor};
use std::fmt;
use std::io::Read;
use std::ops::Index;
use std::ops::Range;
use std::result;

pub fn to_sha256<R: Read>(mut reader: R) -> Result<Checksum> {
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 1024];

    loop {
        let len = reader.read(&mut buffer)?;

        if len == 0 {
            break;
        }

        hasher.update(&buffer[0..len]);
    }

    let checksum = hasher.finish()?;
    Ok(checksum)
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct Checksum(Vec<u8>);

impl Checksum {
    pub fn from_str(input: &str) -> Result<Checksum> {
        let bytes: Vec<u8> = FromHex::from_hex(input)?;

        if bytes.len() != 32usize {
            return Err("expected 32 bytes".into());
        }

        Ok(Checksum(bytes))
    }
}

impl AsRef<[u8]> for Checksum {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Index<Range<usize>> for Checksum {
    type Output = [u8];

    #[inline]
    fn index(&self, index: Range<usize>) -> &[u8] {
        Index::index(&self.0[..], index)
    }
}

pub struct Sha256 {
    context: digest::Context,
}

impl Sha256 {
    pub fn new() -> Sha256 {
        Sha256 { context: digest::Context::new(&digest::SHA256) }
    }

    pub fn update(&mut self, bytes: &[u8]) {
        self.context.update(bytes);
    }

    pub fn finish(self) -> Result<Checksum> {
        Ok(Checksum(self.context.finish().as_ref().to_vec()))
    }
}

impl fmt::Display for Checksum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", HexSlice::new(&self.0[..]))
    }
}

impl Serialize for Checksum {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_str(&format!("{}", HexSlice::new(&self.0[..])))
    }
}

struct ChecksumVisitor;

impl<'de> Visitor<'de> for ChecksumVisitor {
    type Value = Checksum;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a hex encoded string of bytes")
    }

    fn visit_str<E>(self, value: &str) -> result::Result<Self::Value, E>
        where E: Error
    {
        Checksum::from_str(value).map_err(Error::custom)
    }

    fn visit_string<E>(self, value: String) -> result::Result<Self::Value, E>
        where E: Error
    {
        self.visit_str(value.as_str())
    }
}

impl<'de> Deserialize<'de> for Checksum {
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        deserializer.deserialize_str(ChecksumVisitor)
    }
}
