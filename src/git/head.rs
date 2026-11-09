//! What `HEAD` points to.

use std::str;

/// What `HEAD` points to.
#[derive(Debug, PartialEq, Eq)]
pub enum GitHead {
    /// A branch, by its name without `refs/heads/`.
    Branch(String),
    /// A commit, without a branch.
    Detached,
}

impl GitHead {
    /// Reads the full name of the branch `HEAD` points to, such as
    /// `refs/heads/feat/x` followed by a line break, as the branch name.
    ///
    /// The name is taken from the full name rather than from the short name
    /// git prints, which git lengthens to `heads/<branch>` when a tag of the
    /// same name exists.
    pub fn parse_branch(stdout: &[u8]) -> Option<String> {
        let branch = str::from_utf8(stdout)
            .ok()?
            .strip_suffix('\n')?
            .strip_prefix("refs/heads/")?;
        (!branch.is_empty() && !branch.contains('\n')).then(|| branch.to_owned())
    }
}

#[cfg(test)]
#[path = "head_tests.rs"]
mod tests;
