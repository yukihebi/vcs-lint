//! git's reading of a commit message file.

use crate::lint::commit::CommitLintTarget;

/// The content of a commit message file.
pub struct CommitMessage<'a>(pub &'a str);

impl CommitMessage<'_> {
    /// Marks the start of the diff added by `git commit --verbose`.
    const SCISSORS: &str = "# ------------------------ >8 ------------------------";

    const EXEMPT_PREFIXES: [&str; 4] = ["Merge ", "fixup! ", "squash! ", "amend! "];

    /// Sets aside the lines git strips before committing, takes the first
    /// line left as the header, and exempts messages git creates itself.
    pub fn resolve(&self) -> CommitLintTarget {
        let header = self
            .0
            .lines()
            .take_while(|line| *line != Self::SCISSORS)
            .find(|line| !Self::is_comment(line) && !line.trim().is_empty());
        match header {
            None => CommitLintTarget::Empty,
            Some(header) if Self::is_exempt(header) => CommitLintTarget::Exempt,
            Some(header) => CommitLintTarget::Header(header.to_owned()),
        }
    }

    /// Whether git strips `line` as a comment.
    pub fn is_comment(line: &str) -> bool {
        line.starts_with('#')
    }

    fn is_exempt(header: &str) -> bool {
        Self::EXEMPT_PREFIXES
            .iter()
            .any(|prefix| header.starts_with(prefix))
    }
}

#[cfg(test)]
#[path = "message_tests.rs"]
mod tests;
