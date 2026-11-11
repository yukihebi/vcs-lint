//! Hints: text given for a rule, printed after a report of its violation to
//! say what is expected instead.
//!
//! The text follows the syntax of strfmt, where `{` and `}` open and close a
//! placeholder and `{{` and `}}` stand for themselves. No placeholder is
//! supported yet, so that adding one never changes what a valid hint says.

use strfmt::{FmtError, Formatter, strfmt_map};

use crate::report::hint::HintSyntaxError;

/// The lines of a hint, with `{{` and `}}` unescaped; never empty as written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hint(Vec<String>);

/// The hints given to rules `R` of a command, each rule having one at most.
#[derive(Debug)]
pub struct Hints<R>(Vec<(R, Hint)>);

impl Hint {
    /// Parses what a hint option was given, splitting it into lines as
    /// [`str::lines`] does, so that a line break ending it adds no line.
    pub fn parse(written: &str) -> Result<Self, HintSyntaxError> {
        if written.is_empty() {
            return Err(HintSyntaxError::Empty);
        }
        let text = strfmt_map(written, |placeholder: Formatter| {
            Err(FmtError::KeyError(placeholder.key.to_owned()))
        })
        .map_err(|err| match err {
            FmtError::KeyError(name) => HintSyntaxError::Placeholder { name },
            FmtError::Invalid(_) | FmtError::TypeError(_) => HintSyntaxError::Brace,
        })?;
        Ok(Self(text.lines().map(String::from).collect()))
    }

    pub fn lines(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(String::as_str)
    }
}

impl<R: PartialEq> Hints<R> {
    /// The hints given to `rules`, `None` standing for a rule without one.
    pub fn new(rules: impl IntoIterator<Item = (R, Option<Hint>)>) -> Self {
        Self(
            rules
                .into_iter()
                .filter_map(|(rule, hint)| Some((rule, hint?)))
                .collect(),
        )
    }

    /// The hints of `broken`, in its order, a hint given to several of them
    /// kept once.
    pub fn for_rules(&self, broken: &[R]) -> Vec<Hint> {
        let mut hints: Vec<Hint> = Vec::new();
        for rule in broken {
            let found = self.0.iter().find(|(given, _)| given == rule);
            if let Some((_, hint)) = found
                && !hints.contains(hint)
            {
                hints.push(hint.clone());
            }
        }
        hints
    }
}

#[cfg(test)]
#[path = "hint_tests.rs"]
mod tests;
