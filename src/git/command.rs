//! A git command run with the settings every command shares.

use std::{
    ffi::OsStr,
    io::Write,
    process::{Command, Output, Stdio},
    thread,
};

use crate::report::git::GitError;

/// A git command whose standard output is read once it succeeds.
///
/// It inherits the environment, which carries what git sets for hooks, and
/// adds [`GitCommand::ENV`]. Its standard input holds only what
/// [`GitCommand::input`] gives, so it never waits for input.
pub struct GitCommand {
    command: Command,
    /// The command as shown to the user.
    written: String,
    input: Option<Vec<u8>>,
}

/// How a query exits when what it asks for is absent, printing nothing to
/// standard output.
#[derive(Debug, Clone, Copy)]
pub struct GitAbsentStatus {
    code: i32,
    /// Whether it may print to standard error, as the transport of a remote
    /// may print warnings whatever the query finds.
    with_stderr: bool,
}

impl GitCommand {
    const ENV: [(&str, &str); 6] = [
        // Fails instead of prompting for credentials on the terminal.
        ("GIT_TERMINAL_PROMPT", "0"),
        // Fails instead of prompting for credentials with a program, which an
        // empty value keeps git from taking from `core.askPass` or
        // `SSH_ASKPASS`.
        ("GIT_ASKPASS", ""),
        // Keeps ssh from prompting with a program, such as for a passphrase.
        ("SSH_ASKPASS_REQUIRE", "never"),
        // Keeps a partial clone from fetching the objects it lacks, so that
        // only a command reading a remote accesses the network, and an object
        // missing from the repository is found missing.
        ("GIT_NO_LAZY_FETCH", "1"),
        // Skips optional writes such as refreshing the index, so that reading
        // never takes a lock.
        ("GIT_OPTIONAL_LOCKS", "0"),
        // Takes paths as they are, so a directory whose name holds glob
        // characters matches only itself.
        ("GIT_LITERAL_PATHSPECS", "1"),
    ];

    /// A command running `program` with `args`.
    pub fn new(program: &OsStr, args: &[impl AsRef<OsStr>]) -> Self {
        let mut command = Command::new(program);
        command.args(args).envs(Self::ENV);
        let written: Vec<_> = args
            .iter()
            .map(|arg| arg.as_ref().to_string_lossy())
            .collect();
        Self {
            command,
            written: format!("git {}", written.join(" ")),
            input: None,
        }
    }

    /// Gives `input` to the command as its standard input.
    pub fn input(mut self, input: impl Into<Vec<u8>>) -> Self {
        self.input = Some(input.into());
        self
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

    /// Runs a query, which exits as `absent` says when what it asks for is
    /// absent, giving `None`, and otherwise reads its standard output with
    /// `parse`, which gives `None` for output not in the expected form.
    ///
    /// The same status with output it should not print is a failure, as from
    /// a `git` that is not git itself, such as a shim that cannot find git.
    pub fn read_optional<T>(
        self,
        absent: GitAbsentStatus,
        parse: impl FnOnce(&[u8]) -> Option<T>,
    ) -> Result<Option<T>, GitError> {
        let (written, output) = self.output()?;
        match output.status.code() {
            Some(0) => parse(&output.stdout)
                .map(Some)
                .ok_or_else(|| GitError::unexpected_output(written, &output)),
            Some(_) if absent.matches(&output) => Ok(None),
            _ => Err(GitError::failed(written, &output)),
        }
    }

    /// Runs the command to completion, giving it back as written.
    ///
    /// The input is written while the output is read, so that neither waits
    /// for the other. A command that exits before reading all its input is
    /// judged by how it exits.
    fn output(mut self) -> Result<(String, Output), GitError> {
        let stdin = if self.input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        };
        let mut child = self
            .command
            .stdin(stdin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(GitError::spawn)?;
        let output = thread::scope(|scope| {
            if let (Some(mut stdin), Some(input)) = (child.stdin.take(), &self.input) {
                scope.spawn(move || {
                    let _ = stdin.write_all(input);
                });
            }
            child.wait_with_output()
        })
        .map_err(GitError::spawn)?;
        Ok((self.written, output))
    }
}

impl GitAbsentStatus {
    /// Status `1` with no output, as of a quiet query such as
    /// `git symbolic-ref -q`.
    pub const QUIET: Self = Self {
        code: 1,
        with_stderr: false,
    };

    /// Status `code`, possibly with output to standard error.
    pub const fn with_stderr(code: i32) -> Self {
        Self {
            code,
            with_stderr: true,
        }
    }

    fn matches(self, output: &Output) -> bool {
        output.status.code() == Some(self.code)
            && output.stdout.is_empty()
            && (self.with_stderr || output.stderr.is_empty())
    }
}

#[cfg(test)]
#[path = "command_tests.rs"]
mod tests;
