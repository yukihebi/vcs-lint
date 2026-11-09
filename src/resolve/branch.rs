//! Interpretation of the `branch` command.

use super::{resolve_list, resolve_types};
use crate::lint::branch::{BranchLintRules, BranchLintTarget};

/// The `branch` command line, with its values as written.
#[derive(Debug, PartialEq, Eq)]
pub struct BranchRawConfig {
    pub branch: String,
    pub types: Option<Vec<String>>,
    pub protected: Option<Vec<String>>,
}

impl BranchRawConfig {
    /// Without any protected branch, no branch is protected.
    pub fn resolve(self) -> (BranchLintTarget, BranchLintRules) {
        let target = BranchLintTarget {
            branch: self.branch,
        };
        let rules = BranchLintRules {
            types: resolve_types(self.types),
            protected: resolve_list(self.protected).unwrap_or_default(),
        };
        (target, rules)
    }
}

#[cfg(test)]
#[path = "branch_tests.rs"]
mod tests;
