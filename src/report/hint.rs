//! Hints as printed after a report, and why a hint cannot be given, worded
//! for the user.

use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

use derive_more::Display;

use crate::hint::Hint;

/// Why the text given to a hint option is not a hint.
#[derive(Debug, PartialEq, Eq, Display)]
pub enum HintSyntaxError {
    #[display("the hint is empty")]
    Empty,
    #[display("placeholder `{{{name}}}` is not supported; write `{{{{` and `}}}}` for braces")]
    Placeholder { name: String },
    /// A brace opens or closes no placeholder.
    #[display("braces must be written `{{{{` and `}}}}`")]
    Brace,
}

impl Error for HintSyntaxError {}

/// A report followed by the hints of the rules it reports broken.
#[derive(Debug)]
pub struct Hinted<E> {
    pub report: E,
    pub hints: Vec<Hint>,
}

impl<E: Display> Display for Hinted<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.report)?;
        for hint in &self.hints {
            write!(f, "\n{hint}")?;
        }
        Ok(())
    }
}

/// Each line prefixed with `hint: `, or with `hint:` alone when blank.
impl Display for Hint {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (i, line) in self.lines().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            if line.is_empty() {
                write!(f, "hint:")?;
            } else {
                write!(f, "hint: {line}")?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "hint_tests.rs"]
mod tests;
