//! The environment variables vcs-lint reads.

use std::ffi::OsString;

/// The values of the environment variables vcs-lint reads, as set.
#[derive(Debug, Default)]
pub struct Env {
    /// The minimum version of vcs-lint a run requires.
    pub required_version: Option<OsString>,
}

impl Env {
    pub const REQUIRED_VERSION: &str = "VCS_LINT_REQUIRED_VERSION";

    /// Picks the variables vcs-lint reads out of `vars`, ignoring the others.
    pub fn from_vars(vars: impl IntoIterator<Item = (OsString, OsString)>) -> Self {
        let mut env = Self::default();
        for (name, value) in vars {
            if name == Self::REQUIRED_VERSION {
                env.required_version = Some(value);
            }
        }
        env
    }
}
