//! A git command run with the settings every command shares.

use std::{
    ffi::OsStr,
    io::ErrorKind,
    process::{Command, Stdio},
};

use crate::report::git::GitError;

/// A git command whose standard output is read once it succeeds.
///
/// It inherits the environment, which carries what git sets for hooks, and
/// adds [`GitCommand::ENV`]. Its standard input is closed, so it never waits
/// for input.
pub struct GitCommand {
    command: Command,
    /// The command as shown to the user.
    written: String,
}

impl GitCommand {
    const ENV: [(&str, &str); 3] = [
        // Fails instead of prompting for credentials.
        ("GIT_TERMINAL_PROMPT", "0"),
        // Skips optional writes such as refreshing the index, which would
        // contend for its lock with the git process running a hook.
        ("GIT_OPTIONAL_LOCKS", "0"),
        // Takes paths as they are, so a directory whose name holds glob
        // characters matches only itself.
        ("GIT_LITERAL_PATHSPECS", "1"),
    ];

    /// A command running `program` with `args`.
    pub fn new(program: &OsStr, args: &[&str]) -> Self {
        let mut command = Command::new(program);
        command.args(args).envs(Self::ENV).stdin(Stdio::null());
        Self {
            command,
            written: format!("git {}", args.join(" ")),
        }
    }

    /// Runs the command to completion and reads its standard output with
    /// `parse`, which gives `None` for output not in the expected form.
    pub fn read<T>(mut self, parse: impl FnOnce(&[u8]) -> Option<T>) -> Result<T, GitError> {
        let output = self.command.output().map_err(|err| match err.kind() {
            ErrorKind::NotFound => GitError::NotFound,
            _ => GitError::Spawn(err),
        })?;
        if !output.status.success() {
            return Err(GitError::Failed {
                command: self.written,
                status: output.status,
                stderr: String::from_utf8_lossy(&output.stderr)
                    .trim_end()
                    .to_owned(),
            });
        }
        parse(&output.stdout).ok_or_else(|| GitError::UnexpectedOutput {
            command: self.written,
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        })
    }
}

#[cfg(test)]
#[path = "command_tests.rs"]
mod tests;
