//! Versions of vcs-lint and the minimum version a run requires.

use std::{
    ffi::OsStr,
    fmt::{self, Display, Formatter},
};

use crate::report::version::RequiredVersionError;

/// A version of the form `MAJOR.MINOR.PATCH`, optionally followed by
/// `-alpha.N`, `-beta.N` or `-rc.N`, ordered by SemVer precedence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    major: u64,
    minor: u64,
    patch: u64,
    stage: Stage,
}

impl Version {
    /// The version of the running vcs-lint. A package version not of this
    /// form fails to compile.
    pub const CURRENT: Self = match Self::parse_package(env!("CARGO_PKG_VERSION")) {
        Some(version) => version,
        None => panic!("the package version is not of the form vcs-lint takes"),
    };

    /// Fails when `required`, trimmed of surrounding whitespace, is not a
    /// version or is newer than this one. A blank `required` requires nothing.
    pub fn require(self, required: &OsStr) -> Result<(), RequiredVersionError> {
        let written = required.to_string_lossy();
        let written = written.trim();
        if written.is_empty() {
            return Ok(());
        }
        let Some(required) = Self::parse(written) else {
            return Err(RequiredVersionError::Invalid(written.to_owned()));
        };
        if self < required {
            Err(RequiredVersionError::Unmet {
                required,
                current: self,
            })
        } else {
            Ok(())
        }
    }

    /// Parses `written` as a whole, without build metadata.
    const fn parse(written: &str) -> Option<Self> {
        VersionParser::new(written.as_bytes()).parse()
    }

    /// Parses a package version, whose build metadata, already validated by
    /// Cargo, is ignored as it does not take part in precedence.
    const fn parse_package(written: &str) -> Option<Self> {
        let bytes = written.as_bytes();
        let mut end = 0;
        while end < bytes.len() && bytes[end] != b'+' {
            end += 1;
        }
        VersionParser::new(bytes.split_at(end).0).parse()
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        match self.stage {
            Stage::PreRelease(label, number) => write!(f, "-{}.{number}", label.as_str()),
            Stage::Release => Ok(()),
        }
    }
}

/// How far a version is toward its release, declared in order of precedence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Stage {
    PreRelease(PreReleaseLabel, u64),
    Release,
}

/// The kind of a pre-release, declared in order of precedence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum PreReleaseLabel {
    Alpha,
    Beta,
    Rc,
}

impl PreReleaseLabel {
    const ALL: [Self; 3] = [Self::Alpha, Self::Beta, Self::Rc];

    /// The label as written in a version.
    const fn as_str(self) -> &'static str {
        match self {
            Self::Alpha => "alpha",
            Self::Beta => "beta",
            Self::Rc => "rc",
        }
    }
}

/// Reads a version from bytes, in a form usable in constants.
struct VersionParser<'a> {
    /// What is left to read.
    rest: &'a [u8],
}

impl<'a> VersionParser<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { rest: bytes }
    }

    /// Reads a version spanning every byte left.
    const fn parse(mut self) -> Option<Version> {
        let Some(major) = self.number() else {
            return None;
        };
        if !self.eat(b".") {
            return None;
        }
        let Some(minor) = self.number() else {
            return None;
        };
        if !self.eat(b".") {
            return None;
        }
        let Some(patch) = self.number() else {
            return None;
        };
        let Some(stage) = self.stage() else {
            return None;
        };
        if !self.rest.is_empty() {
            return None;
        }
        Some(Version {
            major,
            minor,
            patch,
            stage,
        })
    }

    /// Reads a pre-release preceded by `-`, or nothing for a release.
    const fn stage(&mut self) -> Option<Stage> {
        if !self.eat(b"-") {
            return Some(Stage::Release);
        }
        let Some(label) = self.pre_release_label() else {
            return None;
        };
        if !self.eat(b".") {
            return None;
        }
        match self.number() {
            Some(number) => Some(Stage::PreRelease(label, number)),
            None => None,
        }
    }

    const fn pre_release_label(&mut self) -> Option<PreReleaseLabel> {
        let mut i = 0;
        while i < PreReleaseLabel::ALL.len() {
            let label = PreReleaseLabel::ALL[i];
            if self.eat(label.as_str().as_bytes()) {
                return Some(label);
            }
            i += 1;
        }
        None
    }

    /// Reads decimal digits without leading zeros that fit in `u64`.
    const fn number(&mut self) -> Option<u64> {
        let mut value: u64 = 0;
        let mut digits = 0;
        while let [digit @ b'0'..=b'9', rest @ ..] = self.rest {
            if digits > 0 && value == 0 {
                return None;
            }
            value = match value.checked_mul(10) {
                Some(value) => match value.checked_add((*digit - b'0') as u64) {
                    Some(value) => value,
                    None => return None,
                },
                None => return None,
            };
            digits += 1;
            self.rest = rest;
        }
        if digits == 0 { None } else { Some(value) }
    }

    /// Reads `expected` if the bytes left start with it.
    const fn eat(&mut self, expected: &[u8]) -> bool {
        if self.rest.len() < expected.len() {
            return false;
        }
        let (head, rest) = self.rest.split_at(expected.len());
        let mut i = 0;
        while i < head.len() {
            if head[i] != expected[i] {
                return false;
            }
            i += 1;
        }
        self.rest = rest;
        true
    }
}

#[cfg(test)]
#[path = "version_tests.rs"]
mod tests;
