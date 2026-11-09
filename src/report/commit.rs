//! Why a commit message cannot be linted or breaks the rules, worded for the
//! user.
//!
//! A violation is one broken part of the header, rendered as lines indented
//! under the error holding it.

use std::{
    fmt::{self, Display, Formatter},
    io,
    path::PathBuf,
};

use derive_more::Display;

use crate::name_set::NameSet;

#[derive(Debug, Display)]
#[display("cannot read commit message file `{}`\n  {source}", file.display())]
pub struct CommitResolveError {
    pub file: PathBuf,
    pub source: io::Error,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CommitLintError {
    /// No line is left once the lines git strips are set aside.
    Empty,
    /// The header lacks the `:` after its type and scope.
    Malformed {
        header: String,
        /// The form the header should take.
        expected: &'static str,
    },
    /// Parts of the header break the rules; at least one part is present, and
    /// `spacing` and `subject` are never both present.
    InvalidParts {
        header: String,
        ty: Option<CommitTypeViolation>,
        scope: Option<CommitScopeViolation>,
        spacing: Option<CommitSpacingViolation>,
        subject: Option<CommitSubjectViolation>,
    },
}

impl Display for CommitLintError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "commit message is empty"),
            Self::Malformed { header, expected } => write!(
                f,
                "invalid commit header `{header}`\n  \
                 malformed header\n    \
                 expected: {expected}"
            ),
            Self::InvalidParts {
                header,
                ty,
                scope,
                spacing,
                subject,
            } => {
                write!(f, "invalid commit header `{header}`")?;
                if let Some(ty) = ty {
                    write!(f, "\n{ty}")?;
                }
                if let Some(scope) = scope {
                    write!(f, "\n{scope}")?;
                }
                if let Some(spacing) = spacing {
                    write!(f, "\n{spacing}")?;
                }
                if let Some(subject) = subject {
                    write!(f, "\n{subject}")?;
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Display)]
#[display("  unknown type `{found}`\n    allowed types: {allowed}")]
pub struct CommitTypeViolation {
    pub found: String,
    pub allowed: NameSet,
}

#[derive(Debug, PartialEq, Eq, Display)]
pub enum CommitScopeViolation {
    /// A scope is given where scopes are not used.
    #[display("  unexpected scope `{found}`\n    scopes are not used; remove `({found})`")]
    Unexpected { found: String },
    #[display("  missing scope\n    allowed scopes: {allowed}")]
    Missing { allowed: NameSet },
    #[display("  unknown scope `{found}`\n    allowed scopes: {allowed}")]
    Unknown { found: String, allowed: NameSet },
}

/// The subject is not separated from the `:` by exactly one space.
#[derive(Debug, PartialEq, Eq, Display)]
#[display("  expected exactly one space after `:`")]
pub struct CommitSpacingViolation;

/// Nothing but whitespace follows the `:`.
#[derive(Debug, PartialEq, Eq, Display)]
#[display("  empty subject")]
pub struct CommitSubjectViolation;
