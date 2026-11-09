//! Interpretation of the `commit` command.

mod message;

use std::{fs, path::PathBuf};

use super::resolve_types;
use crate::{
    cli::commit::CommitScopeClapMode,
    lint::commit::{CommitLintRules, CommitLintTarget, CommitScopeRule},
    name_set::NameSet,
    report::commit::CommitResolveError,
};
use message::CommitMessage;

/// The `commit` command line as parsed, before it is interpreted.
#[derive(Debug, PartialEq, Eq)]
pub struct CommitRawConfig {
    pub file: PathBuf,
    pub types: Option<Vec<String>>,
    pub scopes: Option<Vec<String>>,
    pub scope_mode: Option<CommitScopeClapMode>,
}

impl CommitRawConfig {
    /// Reads the message file.
    pub fn resolve(self) -> Result<(CommitLintTarget, CommitLintRules), CommitResolveError> {
        let Self {
            file,
            types,
            scopes,
            scope_mode,
        } = self;
        let text =
            fs::read_to_string(&file).map_err(|source| CommitResolveError { file, source })?;
        let target = CommitMessage(&text).resolve();
        let rules = CommitLintRules {
            types: resolve_types(types),
            scopes: Self::resolve_scopes(scope_mode, scopes),
        };
        Ok((target, rules))
    }

    /// Without a mode, the mode is `auto`: a scope is required when any is
    /// given and forbidden otherwise.
    fn resolve_scopes(
        mode: Option<CommitScopeClapMode>,
        scopes: Option<Vec<String>>,
    ) -> CommitScopeRule {
        let allowed = scopes.map(NameSet::from_iter);
        match mode {
            None | Some(CommitScopeClapMode::Auto) => {
                allowed.map_or(CommitScopeRule::Forbidden, CommitScopeRule::Required)
            }
            Some(CommitScopeClapMode::Required) => {
                CommitScopeRule::Required(allowed.unwrap_or_default())
            }
            Some(CommitScopeClapMode::Optional) => {
                CommitScopeRule::Optional(allowed.unwrap_or_default())
            }
            Some(CommitScopeClapMode::Forbidden) => CommitScopeRule::Forbidden,
        }
    }
}

#[cfg(test)]
#[path = "commit_tests.rs"]
mod tests;
