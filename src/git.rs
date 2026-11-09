//! Reading git through the `git` executable, run as a child process so that
//! it sees the repository as git itself does, including the environment git
//! sets for hooks, such as `GIT_DIR` and `GIT_INDEX_FILE`.

mod command;
mod head;

use std::ffi::OsString;

use crate::report::git::GitError;
use command::GitCommand;
pub use head::GitHead;

/// The `git` executable, run in the current directory.
#[derive(Debug)]
pub struct Git {
    program: OsString,
}

impl Git {
    /// The `git` found on `PATH`.
    pub fn new() -> Self {
        Self {
            program: OsString::from("git"),
        }
    }

    /// What `HEAD` of the repository points to, including a branch with no
    /// commit yet.
    pub fn head(&self) -> Result<GitHead, GitError> {
        let branch = self
            .command(&["symbolic-ref", "-q", "HEAD"])
            .read_optional(GitHead::parse_branch)?;
        Ok(branch.map_or(GitHead::Detached, GitHead::Branch))
    }

    fn command(&self, args: &[&str]) -> GitCommand {
        GitCommand::new(&self.program, args)
    }
}
