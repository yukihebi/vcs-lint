//! Lint for branch names of the form `<type>/<name>`.

use std::sync::LazyLock;

use regex::Regex;

use crate::{
    rules::{BranchRules, list::List, protected::Protected, types::UnknownType},
    violations::Violations,
};

/// Checks `branch` against `rules`.
///
/// A protected branch fails on its own, without its format being checked.
pub fn lint(branch: &str, rules: &BranchRules) -> Result<(), Error> {
    rules.protected.check(branch)?;
    Violation::find(branch, rules).map_err(|violations| Error::Invalid {
        branch: branch.to_owned(),
        violations,
    })
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Protected(#[from] Protected),
    #[error("invalid branch name `{branch}`\n{violations}")]
    Invalid {
        branch: String,
        violations: Violations<Violation>,
    },
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Violation {
    #[error("missing type prefix\n  expected: <type>/<name>\n  allowed types: {allowed}")]
    MissingType { allowed: List },
    #[error(transparent)]
    Type(#[from] UnknownType),
    #[error(
        "invalid name `{name}`\n  expected: lowercase kebab-case starting with a letter, e.g. `add-matcher`"
    )]
    Name { name: String },
}

impl Violation {
    fn find(branch: &str, rules: &BranchRules) -> Result<(), Violations<Self>> {
        let Some((ty, name)) = branch.split_once('/') else {
            return Err(Self::MissingType {
                allowed: rules.types.allowed().clone(),
            }
            .into());
        };
        Violations::collect([rules.types.check(ty).map_err(Into::into), Self::check_name(name)])
    }

    fn check_name(name: &str) -> Result<(), Self> {
        static KEBAB_CASE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"^[a-z][a-z0-9]*(-[a-z0-9]+)*$").unwrap());
        if KEBAB_CASE.is_match(name) {
            Ok(())
        } else {
            Err(Self::Name {
                name: name.to_owned(),
            })
        }
    }
}

#[cfg(test)]
#[path = "branch_tests.rs"]
mod tests;
