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

/// Interprets the values of `--type`, falling back to [`DEFAULT_TYPES`] when
/// it is omitted.
fn resolve_types(values: Option<Vec<String>>) -> NameSet {
    values.map_or_else(|| DEFAULT_TYPES.into_iter().collect(), NameSet::from_iter)
}

#[cfg(test)]
#[path = "resolve_tests.rs"]
mod tests;
