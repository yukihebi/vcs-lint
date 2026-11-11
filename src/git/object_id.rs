//! The names git gives objects.

use std::str;

use derive_more::Display;

/// The full name of an object, in lowercase hexadecimal, as git prints it.
#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub struct GitObjectId(String);

impl GitObjectId {
    /// The numbers of hexadecimal digits of a SHA-1 and of a SHA-256 name.
    const LENGTHS: [usize; 2] = [40, 64];

    /// Reads a name followed by a line break.
    pub fn parse_line(stdout: &[u8]) -> Option<Self> {
        Self::parse(stdout.strip_suffix(b"\n")?)
    }

    /// Reads a name, giving `None` for anything but the full name of an
    /// object.
    pub fn parse(name: &[u8]) -> Option<Self> {
        let is_hex = name
            .iter()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'));
        let name = str::from_utf8(name).ok()?;
        (is_hex && Self::LENGTHS.contains(&name.len())).then(|| Self(name.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
#[path = "object_id_tests.rs"]
mod tests;
