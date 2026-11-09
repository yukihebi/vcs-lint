//! A git command run with the settings every command shares.

use std::{
    ffi::OsStr,
    process::{Command, Output, Stdio},
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
        // Skips optional writes such as refreshing the index, so that reading
        // never takes a lock.
        ("GIT_OPTIONAL_LOCKS", "0"),
        // Takes paths as they are, so a directory whose name holds glob
        // characters matches only itself.
        ("GIT_LITERAL_PATHSPECS", "1"),
    ];

    /// The exit status of a quiet query, such as `git symbolic-ref -q`, when
    /// what it asks for is absent.
    const ABSENT: i32 = 1;

    /// A command running `program` with `args`.
    pub fn new(program: &OsStr, args: &[impl AsRef<OsStr>]) -> Self {
        let mut command = Command::new(program);
        command.args(args).envs(Self::ENV).stdin(Stdio::null());
        let written: Vec<_> = args
            .iter()
            .map(|arg| arg.as_ref().to_string_lossy())
            .collect();
        Self {
            command,
            written: format!("git {}", written.join(" ")),
        }
    }

    /// Runs a command that must succeed, reading its standard output with
    /// `parse`, which gives `None` for output not in the expected form.
    pub fn read<T>(self, parse: impl FnOnce(&[u8]) -> Option<T>) -> Result<T, GitError> {
        let (written, output) = self.output()?;
        if output.status.success() {
            parse(&output.stdout).ok_or_else(|| GitError::unexpected_output(written, &output))
        } else {
            Err(GitError::failed(written, &output))
        }
    }

    /// Runs a quiet query, which exits with [`GitCommand::ABSENT`] and prints
    /// nothing when what it asks for is absent, giving `None`, and otherwise
    /// reads its standard output with `parse`, which gives `None` for output
    /// not in the expected form.
    ///
    /// The same status with output is a failure, as from a `git` that is not
    /// git itself, such as a shim that cannot find git.
    pub fn read_optional<T>(
        self,
        parse: impl FnOnce(&[u8]) -> Option<T>,
    ) -> Result<Option<T>, GitError> {
        let (written, output) = self.output()?;
        match output.status.code() {
            Some(0) => parse(&output.stdout)
                .map(Some)
                .ok_or_else(|| GitError::unexpected_output(written, &output)),
            Some(Self::ABSENT) if output.stdout.is_empty() && output.stderr.is_empty() => Ok(None),
            _ => Err(GitError::failed(written, &output)),
        }
    }

    /// Runs the command to completion, giving it back as written.
    fn output(mut self) -> Result<(String, Output), GitError> {
        let output = self.command.output().map_err(GitError::spawn)?;
        Ok((self.written, output))
    }
}

#[cfg(test)]
#[path = "command_tests.rs"]
mod tests;
