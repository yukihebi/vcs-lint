//! Why the running vcs-lint does not meet the required version, worded for
//! the user.

use std::fmt::{self, Display, Formatter};

use crate::{env::Env, version::Version};

/// Why the required version is not met.
#[derive(Debug, PartialEq, Eq)]
pub enum RequiredVersionError {
    /// The required version, trimmed of surrounding whitespace, is not a
    /// version.
    Invalid(String),
    /// The required version is newer than the running one.
    Unmet { required: Version, current: Version },
}

impl Display for RequiredVersionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(written) => write!(
                f,
                "invalid version {written:?} in `{}`\n  expected `MAJOR.MINOR.PATCH`, optionally followed by `-alpha.N`, `-beta.N` or `-rc.N`",
                Env::REQUIRED_VERSION,
            ),
            Self::Unmet { required, current } => write!(
                f,
                "vcs-lint {required} or later is required, but this is {current}\n  required by `{}`",
                Env::REQUIRED_VERSION,
            ),
        }
    }
}
