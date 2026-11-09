//! Interpretation of the command line into what a run lints and the rules it
//! is linted against.

pub mod branch;
pub mod commit;

use crate::name_set::NameSet;

/// Types commonly used with Conventional Commits, allowed when no type is
/// given.
pub const DEFAULT_TYPES: [&str; 11] = [
    "feat", "fix", "docs", "style", "refactor", "perf", "test", "build", "ci", "chore", "revert",
];

/// Interprets the values of `--type`, falling back to [`DEFAULT_TYPES`].
fn resolve_types(values: Option<Vec<String>>) -> NameSet {
    resolve_list(values).unwrap_or_else(|| DEFAULT_TYPES.into_iter().collect())
}

/// Interprets the values of a list option as written, `None` when omitted.
///
/// Values are trimmed of surrounding whitespace and empty values are dropped;
/// a list left without values is `None`, as if the option were omitted.
fn resolve_list(values: Option<Vec<String>>) -> Option<NameSet> {
    let values = values?;
    let names: Vec<&str> = values
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .collect();
    (!names.is_empty()).then(|| names.into_iter().collect())
}

#[cfg(test)]
#[path = "resolve_tests.rs"]
mod tests;
