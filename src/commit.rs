//! Lint for commit messages, of which only the header is checked.

pub mod header;
mod message;

use crate::{rules::CommitRules, violations::Violations};
use message::Message;

/// Checks the header of `message`, the content of a commit message file.
pub fn lint(message: &str, rules: &CommitRules) -> Result<(), Error> {
    match Message::parse(message) {
        Message::Exempt => Ok(()),
        Message::Empty => Err(Error::Empty),
        Message::Header(header) => {
            header::lint(header, rules).map_err(|violations| Error::Invalid {
                header: header.to_owned(),
                violations,
            })
        }
    }
}

/// Why a commit message was rejected.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("commit message is empty")]
    Empty,
    #[error("invalid commit header `{header}`\n{violations}")]
    Invalid {
        header: String,
        violations: Violations<header::Violation>,
    },
}

#[cfg(test)]
#[path = "commit_tests.rs"]
mod tests;
