//! Why a branch name breaks the rules, worded for the user.
//!
//! A violation is one broken part, rendered as lines indented under the
//! error holding it.

use std::fmt::{self, Display, Formatter};

use derive_more::Display;

use super::{
    git::GitError,
    list::{ListResolveError, UnmatchableList},
};
use crate::name_set::NameSet;

/// Why the rules or the branch to lint cannot be resolved.
#[derive(Debug, PartialEq, Eq, Display)]
pub enum BranchResolveError {
    /// The current branch cannot be read from git.
    #[display("{_0}")]
    Git(GitError),
    #[display("{_0}")]
    List(ListResolveError),
    /// Lists of the rules hold values no branch name can match; at least one
    /// list is present.
    #[display(
        "no branch name can match these values{}",
        UnmatchableList::render_all(_0)
    )]
    Unmatchable(Vec<UnmatchableList>),
}

impl From<GitError> for BranchResolveError {
    fn from(err: GitError) -> Self {
        Self::Git(err)
    }
}

impl From<ListResolveError> for BranchResolveError {
    fn from(err: ListResolveError) -> Self {
        Self::List(err)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum BranchLintError {
    /// Commits on a detached HEAD are not allowed.
    Detached,
    /// The branch must not receive direct commits, whatever its format.
    Protected { branch: String },
    /// The branch has no `/` separating a type from a name.
    Malformed { branch: String, allowed: NameSet },
    /// Parts of the branch break the rules; at least one part is present.
    InvalidParts {
        branch: String,
        ty: Option<BranchTypeViolation>,
        name: Option<BranchNameViolation>,
    },
}

impl Display for BranchLintError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Detached => write!(
                f,
                "commits on a detached HEAD are not allowed\n  \
                 switch to a branch or create one before committing"
            ),
            Self::Protected { branch } => write!(
                f,
                "direct commits to `{branch}` are not allowed\n  \
                 create a branch and open a pull request instead"
            ),
            Self::Malformed { branch, allowed } => write!(
                f,
                "invalid branch name `{branch}`\n  \
                 missing type prefix\n    \
                 expected: <type>/<name>\n    \
                 allowed types: {allowed}"
            ),
            Self::InvalidParts { branch, ty, name } => {
                write!(f, "invalid branch name `{branch}`")?;
                if let Some(ty) = ty {
                    write!(f, "\n{ty}")?;
                }
                if let Some(name) = name {
                    write!(f, "\n{name}")?;
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Display)]
#[display("  unknown type `{found}`\n    allowed types: {allowed}")]
pub struct BranchTypeViolation {
    pub found: String,
    pub allowed: NameSet,
}

#[derive(Debug, PartialEq, Eq, Display)]
#[display(
    "  invalid name `{found}`\n    expected: lowercase kebab-case starting with a letter, e.g. `add-matcher`"
)]
pub struct BranchNameViolation {
    pub found: String,
}
