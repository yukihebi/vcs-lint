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
    /// Resolves the rules before reading the message file.
    pub fn resolve(self) -> Result<(CommitLintTarget, CommitLintRules), CommitResolveError> {
        let Self {
            file,
            types,
            scopes,
            scope_mode,
        } = self;
        let rules = CommitLintRules {
            types: resolve_types(types),
            scopes: Self::resolve_scopes(scope_mode, scopes)?,
        };
        let text = fs::read_to_string(&file)
            .map_err(|source| CommitResolveError::Unreadable { file, source })?;
        let target = CommitMessage(&text).resolve();
        Ok((target, rules))
    }

    /// Without a mode, the mode is `auto`: a scope is required when any is
    /// given and forbidden otherwise. A mode that lets headers have a scope
    /// fails when no scope is allowed.
    fn resolve_scopes(
        mode: Option<CommitScopeClapMode>,
        scopes: Option<Vec<String>>,
    ) -> Result<CommitScopeRule, CommitResolveError> {
        let mode = mode.unwrap_or(CommitScopeClapMode::Auto);
        let given = scopes.is_some();
        let allowed: NameSet = scopes.into_iter().flatten().collect();
        match mode {
            CommitScopeClapMode::Auto if !given => Ok(CommitScopeRule::Forbidden),
            CommitScopeClapMode::Forbidden => Ok(CommitScopeRule::Forbidden),
            _ if allowed.is_empty() => Err(CommitResolveError::NoScopes { mode }),
            CommitScopeClapMode::Auto | CommitScopeClapMode::Required => {
                Ok(CommitScopeRule::Required(allowed))
            }
            CommitScopeClapMode::Optional => Ok(CommitScopeRule::Optional(allowed)),
        }
    }
}

#[cfg(test)]
#[path = "commit_tests.rs"]
mod tests;
