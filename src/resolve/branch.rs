//! Interpretation of the `branch` command.

use super::{list::ListRawValues, matchable::ListRequirement, resolve_types};
use crate::{
    cli::{TypeDefaultsClapMode, branch::BranchDetachedClapRule},
    git::{Git, head::GitHead},
    lint::branch::{BranchDetachedRule, BranchLintRules, BranchLintTarget},
    report::{branch::BranchResolveError, git::GitError, list::ListName},
};

/// The `branch` command line as parsed, before it is interpreted.
#[derive(Debug, PartialEq, Eq)]
pub struct BranchRawConfig {
    /// The branch name given, or `None` to lint the current branch.
    pub branch: Option<String>,
    pub types: ListRawValues,
    pub type_defaults: Option<TypeDefaultsClapMode>,
    pub protected: ListRawValues,
    pub detached: Option<BranchDetachedClapRule>,
}

impl BranchRawConfig {
    const TYPES: ListRequirement = ListRequirement {
        list: ListName::Types,
        is_matchable: BranchLintRules::is_matchable_type,
        wording: "a type cannot contain `/` or a line break",
    };

    const PROTECTED: ListRequirement = ListRequirement {
        list: ListName::Protected,
        is_matchable: |_| true,
        wording: "a branch name cannot contain a line break",
    };

    /// Without any protected branch left, no branch is protected; without a
    /// rule for a detached HEAD, a detached HEAD is allowed.
    ///
    /// Resolves the rules before reading the current branch from `git`, so
    /// that rules that cannot be resolved are reported without reading it.
    pub fn resolve(
        self,
        git: &Git,
    ) -> Result<(BranchLintTarget, BranchLintRules), BranchResolveError> {
        let rules = BranchLintRules {
            types: resolve_types(self.types, self.type_defaults, git)?,
            protected: self.protected.resolve(ListName::Protected, &[], git)?,
            detached: Self::resolve_detached(self.detached),
        };
        Self::check_matchable(&rules)?;
        let target = match self.branch {
            Some(branch) => Self::resolve_target(branch),
            None => Self::read_target(git)?,
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

    /// The current branch, or a detached HEAD, as `HEAD` points to it.
    fn read_target(git: &Git) -> Result<BranchLintTarget, GitError> {
        Ok(match git.head()? {
            GitHead::Branch(branch) => BranchLintTarget::Branch(branch),
            GitHead::Detached => BranchLintTarget::Detached,
        })
    }

    fn resolve_detached(rule: Option<BranchDetachedClapRule>) -> BranchDetachedRule {
        match rule {
            None | Some(BranchDetachedClapRule::Allow) => BranchDetachedRule::Allowed,
            Some(BranchDetachedClapRule::Deny) => BranchDetachedRule::Forbidden,
        }
    }

    /// Fails when the rules hold values no branch name can match, reporting
    /// every list holding some.
    fn check_matchable(rules: &BranchLintRules) -> Result<(), BranchResolveError> {
        let unmatchable: Vec<_> = [
            Self::TYPES.find_unmatchable(&rules.types),
            Self::PROTECTED.find_unmatchable(&rules.protected),
        ]
        .into_iter()
        .flatten()
        .collect();
        if unmatchable.is_empty() {
            Ok(())
        } else {
            Err(BranchResolveError::Unmatchable(unmatchable))
        }
    }
}

#[cfg(test)]
#[path = "branch_tests.rs"]
mod tests;
