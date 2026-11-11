//! The environment variables vcs-lint reads.

use std::ffi::OsString;

/// The values of the environment variables vcs-lint reads, as set.
#[derive(Debug)]
pub struct Env {
    /// The minimum version of vcs-lint a run requires.
    pub required_version: Option<OsString>,
}

impl Env {
    pub const REQUIRED_VERSION: &str = "VCS_LINT_REQUIRED_VERSION";

    /// Reads the variables vcs-lint reads through `var`, which gives the value
    /// of the variable named by its argument, so no other variable is read.
    pub fn read(var: impl Fn(&str) -> Option<OsString>) -> Self {
        Self {
            required_version: var(Self::REQUIRED_VERSION),
        }
    }
}
