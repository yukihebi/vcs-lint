//! Commit headers of the form `<type>(<scope>): <subject>`.

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
    ///
    /// The type ends at the first `(`, `)`, `:`, `!` or whitespace, and the
    /// scope at the `)` closing the `(` that opens it, so a scope may hold
    /// balanced parentheses.
    pub fn parse(line: &'a str) -> Option<Self> {
        let (ty, rest) = line.split_at(line.find(Self::ends_type).unwrap_or(line.len()));
        let (scope, rest) = match rest.strip_prefix('(') {
            Some(inner) => {
                let end = Self::find_closing_paren(inner)?;
                (Some(&inner[..end]), &inner[end + 1..])
            }
            None => (None, rest),
        };
        let rest = rest.strip_prefix(':')?;
        let (spacing, subject) = rest.split_at(rest.len() - rest.trim_start().len());
        Some(Self {
            ty,
            scope,
            spacing,
            subject,
        })
    }

    /// Whether a header can have `ty` as its type, which holds when nothing
    /// in `ty` ends a type.
    pub fn is_matchable_type(ty: &str) -> bool {
        !ty.contains(Self::ends_type)
    }

    /// Whether a header can have `scope` as its scope, which holds when its
    /// parentheses are balanced.
    pub fn is_matchable_scope(scope: &str) -> bool {
        Self::find_closing_paren(&format!("{scope})")) == Some(scope.len())
    }

    /// Spacing is only judged once there is a subject to separate.
    pub fn lint_spacing(&self) -> Option<CommitSpacingViolation> {
        (!self.subject.is_empty() && self.spacing != " ").then_some(CommitSpacingViolation)
    }

    pub fn lint_subject(&self) -> Option<CommitSubjectViolation> {
        self.subject.is_empty().then_some(CommitSubjectViolation)
    }

    fn ends_type(c: char) -> bool {
        matches!(c, '(' | ')' | ':' | '!') || c.is_whitespace()
    }

    /// The index of the first `)` in `text` that closes a `(` opened before
    /// `text`, passing over the pairs of parentheses in between.
    fn find_closing_paren(text: &str) -> Option<usize> {
        let mut depth = 0_usize;
        for (index, c) in text.char_indices() {
            match c {
                '(' => depth += 1,
                ')' if depth == 0 => return Some(index),
                ')' => depth -= 1,
                _ => {}
            }
        }
        None
    }
}

#[cfg(test)]
#[path = "header_tests.rs"]
mod tests;
