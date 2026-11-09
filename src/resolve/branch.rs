//! Interpretation of the `branch` command.

use super::{list::ListRawValues, resolve_types};
use crate::{
    cli::{TypeDefaultsClapMode, branch::BranchDetachedClapRule},
    lint::branch::{BranchDetachedRule, BranchLintRules, BranchLintTarget},
    report::{branch::BranchResolveError, list::ListName},
};

/// The `branch` command line as parsed, before it is interpreted.
#[derive(Debug, PartialEq, Eq)]
pub struct BranchRawConfig {
    pub branch: String,
    pub types: ListRawValues,
    pub type_defaults: Option<TypeDefaultsClapMode>,
    pub protected: ListRawValues,
    pub detached: Option<BranchDetachedClapRule>,
}

impl BranchRawConfig {
    /// Without any protected branch left, no branch is protected; without a
    /// rule for a detached HEAD, a detached HEAD is allowed.
    pub fn resolve(self) -> Result<(BranchLintTarget, BranchLintRules), BranchResolveError> {
        let target = Self::resolve_target(self.branch);
        let rules = BranchLintRules {
            types: resolve_types(self.types, self.type_defaults)?,
            protected: self.protected.resolve(ListName::Protected, &[])?,
            detached: Self::resolve_detached(self.detached),
        };
        Ok((target, rules))
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
