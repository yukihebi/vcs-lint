//! Why a branch name breaks the rules, worded for the user.
//!
//! A violation is one broken part, rendered as lines indented under the
//! error holding it.

use std::fmt::{self, Display, Formatter};

use derive_more::Display;

use crate::name_set::NameSet;

#[derive(Debug, PartialEq, Eq)]
pub enum BranchLintError {
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
