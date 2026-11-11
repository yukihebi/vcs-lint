//! Interpretation of the command line into what a run lints and the rules it
//! is linted against.

pub mod branch;
pub mod commit;
pub mod history;
pub mod list;
pub mod matchable;

use crate::{
    cli::TypeDefaultsClapMode,
    git::Git,
    name_set::NameSet,
    report::list::{ListName, ListResolveError},
};
use list::ListRawValues;

/// Types commonly used with Conventional Commits, allowed by default.
pub const DEFAULT_TYPES: [&str; 11] = [
    "feat", "fix", "docs", "style", "refactor", "perf", "test", "build", "ci", "chore", "revert",
];

/// Composes the allowed types, including [`DEFAULT_TYPES`] as `mode` says
/// (`auto` when omitted), failing when no type is left since no type could
/// pass.
fn resolve_types(
    types: ListRawValues,
    mode: Option<TypeDefaultsClapMode>,
    git: &Git,
) -> Result<NameSet, ListResolveError> {
    let with_defaults = match mode.unwrap_or(TypeDefaultsClapMode::Auto) {
        TypeDefaultsClapMode::Auto => !types.is_given(),
        TypeDefaultsClapMode::Always => true,
        TypeDefaultsClapMode::Never => false,
    };
    let defaults: &[&str] = if with_defaults { &DEFAULT_TYPES } else { &[] };
    let allowed = types.resolve(ListName::Types, defaults, git)?;
    if allowed.is_empty() {
        Err(ListResolveError::Empty {
            list: ListName::Types,
        })
    } else {
        Ok(allowed)
    }
}

#[cfg(test)]
#[path = "resolve_tests.rs"]
mod tests;
