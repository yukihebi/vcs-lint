//! Reading git through the `git` executable, run as a child process so that
//! it sees the repository as git itself does, including the environment git
//! sets for hooks, such as `GIT_DIR` and `GIT_INDEX_FILE`.

mod command;
mod version;

use std::ffi::OsString;

use crate::report::git::GitError;
use command::GitCommand;
pub use version::GitVersion;

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

    /// The version of git, readable outside a repository too.
    pub fn version(&self) -> Result<GitVersion, GitError> {
        self.command(&["version"]).read(GitVersion::parse)
    }

    fn command(&self, args: &[&str]) -> GitCommand {
        GitCommand::new(&self.program, args)
    }
}

#[cfg(test)]
#[path = "git_tests.rs"]
mod tests;
