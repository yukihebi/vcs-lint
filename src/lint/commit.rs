//! Lint for commit messages, of which only the header is checked.

mod header;

use crate::{
    name_set::NameSet,
    report::commit::{CommitLintError, CommitScopeViolation, CommitTypeViolation},
};
use header::CommitHeader;

/// What a commit message amounts to once the lines git strips are set aside.
#[derive(Debug, PartialEq, Eq)]
pub enum CommitLintTarget {
    /// The first line of the message.
    Header(String),
    /// A message git creates for merges or autosquash, accepted as is.
    Exempt,
    /// No line is left.
    Empty,
}

#[derive(Debug, PartialEq, Eq)]
pub struct CommitLintRules {
    /// Types a header may start with.
    pub types: NameSet,
    pub scopes: CommitScopeRule,
}

/// Whether headers carry a scope, and which scopes they may use.
#[derive(Debug, PartialEq, Eq)]
pub enum CommitScopeRule {
    /// Headers must not have a scope.
    Forbidden,
    /// Headers must have one of the scopes.
    Required(NameSet),
}

impl CommitLintRules {
    pub fn lint(&self, target: &CommitLintTarget) -> Result<(), CommitLintError> {
        match target {
            CommitLintTarget::Header(header) => self.lint_header(header),
            CommitLintTarget::Exempt => Ok(()),
            CommitLintTarget::Empty => Err(CommitLintError::Empty),
        }
    }

    fn lint_header(&self, line: &str) -> Result<(), CommitLintError> {
        let Some(header) = CommitHeader::parse(line) else {
            return Err(CommitLintError::Malformed {
                header: line.to_owned(),
                expected: self.scopes.expected_form(),
            });
        };
        match (
            self.lint_type(header.ty),
            self.scopes.lint(header.scope),
            header.lint_spacing(),
            header.lint_subject(),
        ) {
            (None, None, None, None) => Ok(()),
            (ty, scope, spacing, subject) => Err(CommitLintError::InvalidParts {
                header: line.to_owned(),
                ty,
                scope,
                spacing,
                subject,
            }),
        }
    }

    fn lint_type(&self, ty: &str) -> Option<CommitTypeViolation> {
        (!self.types.contains(ty)).then(|| CommitTypeViolation {
            found: ty.to_owned(),
            allowed: self.types.clone(),
        })
    }
}

impl CommitScopeRule {
    /// The form a header takes under this rule.
    fn expected_form(&self) -> &'static str {
        match self {
            Self::Forbidden => "<type>: <subject>",
            Self::Required(_) => "<type>(<scope>): <subject>",
        }
    }

    fn lint(&self, found: Option<&str>) -> Option<CommitScopeViolation> {
        match (self, found) {
            (Self::Forbidden, None) => None,
            (Self::Forbidden, Some(found)) => Some(CommitScopeViolation::Unexpected {
                found: found.to_owned(),
            }),
            (Self::Required(allowed), None) => Some(CommitScopeViolation::Missing {
                allowed: allowed.clone(),
            }),
            (Self::Required(allowed), Some(found)) if allowed.contains(found) => None,
            (Self::Required(allowed), Some(found)) => Some(CommitScopeViolation::Unknown {
                found: found.to_owned(),
                allowed: allowed.clone(),
            }),
        }
    }
}

#[cfg(test)]
#[path = "commit_tests.rs"]
mod tests;
