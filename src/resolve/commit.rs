//! Interpretation of the `commit` command.

mod message;

use std::{fs, path::PathBuf};

use super::{resolve_list, resolve_types};
use crate::{
    lint::commit::{CommitLintRules, CommitLintTarget, CommitScopeRule},
    report::commit::CommitResolveError,
};
use message::CommitMessage;

/// The `commit` command line, with its values as written.
#[derive(Debug, PartialEq, Eq)]
pub struct CommitRawConfig {
    pub file: PathBuf,
    pub types: Option<Vec<String>>,
    pub scopes: Option<Vec<String>>,
}

impl CommitRawConfig {
    /// Reads the message file; without any scope, headers must not have one.
    pub fn resolve(self) -> Result<(CommitLintTarget, CommitLintRules), CommitResolveError> {
        let Self {
            file,
            types,
            scopes,
        } = self;
        let text =
            fs::read_to_string(&file).map_err(|source| CommitResolveError { file, source })?;
        let target = CommitMessage(&text).resolve();
        let rules = CommitLintRules {
            types: resolve_types(types),
            scopes: resolve_list(scopes)
                .map_or(CommitScopeRule::Forbidden, CommitScopeRule::Required),
        };
        Ok((target, rules))
    }
}

#[cfg(test)]
#[path = "commit_tests.rs"]
mod tests;
