//! Interpretation of the `branch` command.

use super::{resolve_list, resolve_types};
use crate::{
    cli::branch::BranchDetachedClapRule,
    lint::branch::{BranchDetachedRule, BranchLintRules, BranchLintTarget},
};

/// The `branch` command line, with its values as written.
#[derive(Debug, PartialEq, Eq)]
pub struct BranchRawConfig {
    pub branch: String,
    pub types: Option<Vec<String>>,
    pub protected: Option<Vec<String>>,
    pub detached: Option<BranchDetachedClapRule>,
}

impl BranchRawConfig {
    /// Without any protected branch, no branch is protected; without a rule
    /// for a detached HEAD, a detached HEAD is allowed.
    pub fn resolve(self) -> (BranchLintTarget, BranchLintRules) {
        let target = Self::resolve_target(self.branch);
        let rules = BranchLintRules {
            types: resolve_types(self.types),
            protected: resolve_list(self.protected).unwrap_or_default(),
            detached: Self::resolve_detached(self.detached),
        };
        (target, rules)
    }

    /// Git refuses `HEAD` as a branch name, so the name `HEAD` can only stand
    /// for a detached HEAD.
    fn resolve_target(branch: String) -> BranchLintTarget {
        if branch == "HEAD" {
            BranchLintTarget::Detached
        } else {
            BranchLintTarget::Branch(branch)
        }
    }

    fn resolve_detached(rule: Option<BranchDetachedClapRule>) -> BranchDetachedRule {
        match rule {
            None | Some(BranchDetachedClapRule::Allow) => BranchDetachedRule::Allowed,
            Some(BranchDetachedClapRule::Deny) => BranchDetachedRule::Forbidden,
        }
    }
}

#[cfg(test)]
#[path = "branch_tests.rs"]
mod tests;
