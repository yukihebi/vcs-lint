//! Lint for branch names of the form `<type>/<name>`.

use std::sync::LazyLock;

use regex::Regex;

use crate::{
    name_set::NameSet,
    report::branch::{BranchLintError, BranchNameViolation, BranchTypeViolation},
};

/// What a run lints.
#[derive(Debug, PartialEq, Eq)]
pub enum BranchLintTarget {
    /// A branch name.
    Branch(String),
    /// A detached HEAD, which has no branch name.
    Detached,
}

#[derive(Debug, PartialEq, Eq)]
pub struct BranchLintRules {
    /// Types a branch name may start with.
    pub types: NameSet,
    /// Branches that fail whatever their format.
    pub protected: NameSet,
    pub detached: BranchDetachedRule,
}

/// Whether a detached HEAD passes.
#[derive(Debug, PartialEq, Eq)]
pub enum BranchDetachedRule {
    Allowed,
    Forbidden,
}

impl BranchLintRules {
    /// Separates the type of a branch name from its name, at its first
    /// occurrence.
    const TYPE_SEPARATOR: char = '/';

    /// A detached HEAD is checked against the detached HEAD rule alone.
    pub fn lint(&self, target: &BranchLintTarget) -> Result<(), BranchLintError> {
        match target {
            BranchLintTarget::Branch(branch) => self.lint_branch(branch),
            BranchLintTarget::Detached => self.detached.lint(),
        }
    }

    /// A protected branch fails on its own, without its format being checked.
    fn lint_branch(&self, branch: &str) -> Result<(), BranchLintError> {
        if self.protected.contains(branch) {
            return Err(BranchLintError::Protected {
                branch: branch.to_owned(),
            });
        }
        let Some((ty, name)) = branch.split_once(Self::TYPE_SEPARATOR) else {
            return Err(BranchLintError::Malformed {
                branch: branch.to_owned(),
                allowed: self.types.clone(),
            });
        };
        match (self.lint_type(ty), Self::lint_name(name)) {
            (None, None) => Ok(()),
            (ty, name) => Err(BranchLintError::InvalidParts {
                branch: branch.to_owned(),
                ty,
                name,
            }),
        }
    }

    /// Whether a branch name can have `ty` as its type, which holds when `ty`
    /// lacks the separator ending a type.
    pub fn is_matchable_type(ty: &str) -> bool {
        !ty.contains(Self::TYPE_SEPARATOR)
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

impl BranchDetachedRule {
    fn lint(&self) -> Result<(), BranchLintError> {
        match self {
            Self::Allowed => Ok(()),
            Self::Forbidden => Err(BranchLintError::Detached),
        }
    }
}

#[cfg(test)]
#[path = "branch_tests.rs"]
mod tests;
