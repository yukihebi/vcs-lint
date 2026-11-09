//! Lint for branch names of the form `<type>/<name>`.

use std::sync::LazyLock;

use regex::Regex;

use crate::{
    name_set::NameSet,
    report::branch::{BranchLintError, BranchNameViolation, BranchTypeViolation},
};

/// The branch a run lints.
#[derive(Debug, PartialEq, Eq)]
pub struct BranchLintTarget {
    pub branch: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct BranchLintRules {
    /// Types a branch name may start with.
    pub types: NameSet,
    /// Branches that must not receive direct commits.
    pub protected: NameSet,
}

impl BranchLintRules {
    /// A protected branch fails on its own, without its format being checked.
    pub fn lint(&self, target: &BranchLintTarget) -> Result<(), BranchLintError> {
        let branch = &target.branch;
        if self.protected.contains(branch) {
            return Err(BranchLintError::Protected {
                branch: branch.clone(),
            });
        }
        let Some((ty, name)) = branch.split_once('/') else {
            return Err(BranchLintError::Malformed {
                branch: branch.clone(),
                allowed: self.types.clone(),
            });
        };
        match (self.lint_type(ty), Self::lint_name(name)) {
            (None, None) => Ok(()),
            (ty, name) => Err(BranchLintError::InvalidParts {
                branch: branch.clone(),
                ty,
                name,
            }),
        }
    }

    fn lint_type(&self, ty: &str) -> Option<BranchTypeViolation> {
        (!self.types.contains(ty)).then(|| BranchTypeViolation {
            found: ty.to_owned(),
            allowed: self.types.clone(),
        })
    }

    /// Names are lowercase kebab-case starting with a letter.
    fn lint_name(name: &str) -> Option<BranchNameViolation> {
        static KEBAB_CASE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"^[a-z][a-z0-9]*(-[a-z0-9]+)*$").unwrap());
        (!KEBAB_CASE.is_match(name)).then(|| BranchNameViolation {
            found: name.to_owned(),
        })
    }
}

#[cfg(test)]
#[path = "branch_tests.rs"]
mod tests;
