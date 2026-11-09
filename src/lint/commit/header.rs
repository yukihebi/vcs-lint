//! Commit headers of the form `<type>(<scope>): <subject>`.

use std::sync::LazyLock;

use regex::Regex;

use crate::report::commit::{CommitSpacingViolation, CommitSubjectViolation};

/// A header split into its parts.
#[derive(Debug, PartialEq, Eq)]
pub struct CommitHeader<'a> {
    pub ty: &'a str,
    pub scope: Option<&'a str>,
    /// Whitespace between `:` and the subject.
    pub spacing: &'a str,
    pub subject: &'a str,
}

impl<'a> CommitHeader<'a> {
    /// Splits `line`, or returns `None` when it lacks the `:` after the type
    /// and scope.
    pub fn parse(line: &'a str) -> Option<Self> {
        static PATTERN: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(
                r"^(?<type>[^():!\s]*)(?:\((?<scope>[^()]*)\))?:(?<spacing>\s*)(?<subject>.*)$",
            )
            .unwrap()
        });
        let captures = PATTERN.captures(line)?;
        Some(Self {
            ty: captures.name("type")?.as_str(),
            scope: captures.name("scope").map(|scope| scope.as_str()),
            spacing: captures.name("spacing")?.as_str(),
            subject: captures.name("subject")?.as_str(),
        })
    }

    /// Spacing is only judged once there is a subject to separate.
    pub fn lint_spacing(&self) -> Option<CommitSpacingViolation> {
        (!self.subject.is_empty() && self.spacing != " ").then_some(CommitSpacingViolation)
    }

    pub fn lint_subject(&self) -> Option<CommitSubjectViolation> {
        self.subject.is_empty().then_some(CommitSubjectViolation)
    }
}

#[cfg(test)]
#[path = "header_tests.rs"]
mod tests;
