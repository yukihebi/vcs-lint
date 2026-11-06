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
    let checked = match BranchName::parse(branch) {
        Some(name) => name.check(rules),
        None => Err(Violation::missing_type(rules).into()),
    };
    checked.map_err(|violations| Error::Invalid {
        branch: branch.to_owned(),
        violations,
    })
}

/// Why a branch name was rejected.
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

/// A rule broken by a branch name.
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
    fn missing_type(rules: &BranchRules) -> Self {
        Self::MissingType {
            allowed: rules.types.allowed().clone(),
        }
    }
}

/// A branch name split into its parts, none of which is checked yet.
#[derive(Debug, PartialEq, Eq)]
struct BranchName<'a> {
    ty: &'a str,
    name: &'a str,
}

impl<'a> BranchName<'a> {
    /// Splits `branch` at its first `/`, or returns `None` when it has none.
    fn parse(branch: &'a str) -> Option<Self> {
        let (ty, name) = branch.split_once('/')?;
        Some(Self { ty, name })
    }

    fn check(&self, rules: &BranchRules) -> Result<(), Violations<Violation>> {
        Violations::collect([
            rules.types.check(self.ty).map_err(Into::into),
            self.check_name(),
        ])
    }

    fn check_name(&self) -> Result<(), Violation> {
        static KEBAB_CASE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"^[a-z][a-z0-9]*(-[a-z0-9]+)*$").unwrap());
        if KEBAB_CASE.is_match(self.name) {
            Ok(())
        } else {
            Err(Violation::Name {
                name: self.name.to_owned(),
            })
        }
    }
}

#[cfg(test)]
#[path = "branch_tests.rs"]
mod tests;
