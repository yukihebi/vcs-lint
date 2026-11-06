//! Exit statuses of vcs-lint.

use std::process::ExitCode;

/// The outcome of a run, reported as the process exit status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Status {
    /// No violation was found, or help or the version was printed.
    Success = 0,
    /// The input breaks a rule.
    Violation = 1,
    /// The input could not be read, so it was not linted.
    RuntimeError = 2,
    /// The command line is invalid.
    UsageError = 3,
}

impl Status {
    /// Classifies a command line that clap did not resolve into a command.
    ///
    /// clap reports printing help or the version through an error as well,
    /// which succeeds; only output meant for standard error is a failure.
    pub fn of_clap(err: &clap::Error) -> Self {
        if err.use_stderr() {
            Self::UsageError
        } else {
            Self::Success
        }
    }
}

impl From<Status> for ExitCode {
    fn from(status: Status) -> Self {
        Self::from(status as u8)
    }
}
