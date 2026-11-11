//! The names git gives objects.

use std::str;

use derive_more::Display;

/// The full name of an object, in lowercase hexadecimal, as git prints it.
#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub struct GitObjectId(String);

impl GitObjectId {
    /// The numbers of hexadecimal digits of a SHA-1 and of a SHA-256 name.
    const LENGTHS: [usize; 2] = [40, 64];

    /// The words `git cat-file --batch-check` prints after a name that names
    /// no single object.
    const NOT_FOUND: [&str; 2] = ["missing", "ambiguous"];

    /// Reads what `git cat-file --batch-check=%(objectname)` prints for the
    /// one line `name` given to it: the object `name` names, or `name`
    /// followed by why it names no single object, giving `Some(None)`.
    ///
    /// Gives `None` for output not in the expected form.
    pub fn parse_lookup(stdout: &[u8], name: &str) -> Option<Option<Self>> {
        let line = str::from_utf8(stdout.strip_suffix(b"\n")?).ok()?;
        if let Some(id) = Self::parse(line.as_bytes()) {
            return Some(Some(id));
        }
        let (named, word) = line.rsplit_once(' ')?;
        (named == name && Self::NOT_FOUND.contains(&word)).then_some(None)
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
