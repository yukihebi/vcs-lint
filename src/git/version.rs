//! The version of git.

use std::str;

/// The release numbers of git, ordered as releases are.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct GitVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl GitVersion {
    /// Reads the output of `git version`, such as `git version 2.39.5` or a
    /// vendor build like `git version 2.39.5 (Apple Git-154)` and
    /// `git version 2.45.1.windows.1`, from its first three release numbers.
    pub fn parse(output: &[u8]) -> Option<Self> {
        let version = str::from_utf8(output)
            .ok()?
            .strip_prefix("git version ")?
            .split_whitespace()
            .next()?;
        let mut numbers = version.split('.').map(str::parse);
        Some(Self {
            major: numbers.next()?.ok()?,
            minor: numbers.next()?.ok()?,
            patch: numbers.next()?.ok()?,
        })
    }
}

#[cfg(test)]
#[path = "version_tests.rs"]
mod tests;
