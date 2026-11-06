//! Lint for commit headers of the form `<type>(<scope>): <subject>`.

use std::sync::LazyLock;

use regex::Regex;

use crate::{
    rules::{
        CommitRules,
        scopes::{ScopeError, ScopeRule},
        types::UnknownType,
    },
    violations::Violations,
};

/// Checks a single header line against `rules`.
pub fn lint(header: &str, rules: &CommitRules) -> Result<(), Violations<Violation>> {
    match Header::parse(header) {
        Some(header) => header.check(rules),
        None => Err(Violation::malformed(&rules.scopes).into()),
    }
}

/// A rule broken by a commit header.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Violation {
    #[error("malformed header\n  expected: {expected}")]
    Malformed { expected: &'static str },
    #[error(transparent)]
    Type(#[from] UnknownType),
    #[error(transparent)]
    Scope(#[from] ScopeError),
    #[error("expected exactly one space after `:`")]
    Spacing,
    #[error("empty subject")]
    EmptySubject,
}

impl Violation {
    fn malformed(scopes: &ScopeRule) -> Self {
        let expected = match scopes {
            ScopeRule::Forbidden => "<type>: <subject>",
            ScopeRule::Required(_) => "<type>(<scope>): <subject>",
        };
        Self::Malformed { expected }
    }
}

/// A header split into its parts, none of which is checked yet.
#[derive(Debug, PartialEq, Eq)]
struct Header<'a> {
    ty: &'a str,
    scope: Option<&'a str>,
    /// Whitespace between `:` and the subject.
    spacing: &'a str,
    subject: &'a str,
}

impl<'a> Header<'a> {
    /// Splits `line`, or returns `None` when it lacks the `:` after the type
    /// and scope.
    fn parse(line: &'a str) -> Option<Self> {
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

    fn check(&self, rules: &CommitRules) -> Result<(), Violations<Violation>> {
        Violations::collect([
            rules.types.check(self.ty).map_err(Into::into),
            rules.scopes.check(self.scope).map_err(Into::into),
            self.check_subject(),
        ])
    }

    /// Spacing is only judged once there is a subject to separate.
    fn check_subject(&self) -> Result<(), Violation> {
        if self.subject.is_empty() {
            Err(Violation::EmptySubject)
        } else if self.spacing != " " {
            Err(Violation::Spacing)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
#[path = "header_tests.rs"]
mod tests;
