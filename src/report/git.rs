//! Why git cannot be read, worded for the user.

use std::{
    fmt::{self, Display, Formatter},
    io::{self, ErrorKind},
    process::{ExitStatus, Output},
};

/// Why a git command does not give what is asked of it.
#[derive(Debug, PartialEq, Eq)]
pub enum GitError {
    /// The `git` executable is not found.
    NotFound,
    /// The `git` executable is found but cannot be run.
    Spawn(ErrorKind),
    /// The command exits with a failure.
    Failed {
        command: String,
        status: ExitStatus,
        /// What the command printed to standard error, without trailing
        /// whitespace.
        stderr: String,
    },
    /// The command succeeds, but its output is not in the expected form.
    UnexpectedOutput { command: String, stdout: String },
}

impl GitError {
    /// Why the `git` executable cannot be run.
    pub fn spawn(err: io::Error) -> Self {
        match err.kind() {
            ErrorKind::NotFound => Self::NotFound,
            kind => Self::Spawn(kind),
        }
    }

    pub fn failed(command: String, output: &Output) -> Self {
        Self::Failed {
            command,
            status: output.status,
            stderr: String::from_utf8_lossy(&output.stderr)
                .trim_end()
                .to_owned(),
        }
    }

    pub fn unexpected_output(command: String, output: &Output) -> Self {
        Self::UnexpectedOutput {
            command,
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        }
    }
}

impl Display for GitError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => write!(f, "git is not found\n  install git or add it to `PATH`"),
            Self::Spawn(kind) => write!(f, "cannot run git: {kind}"),
            Self::Failed {
                command,
                status,
                stderr,
            } => {
                write!(f, "`{command}` failed with {status}")?;
                for line in stderr.lines() {
                    write!(f, "\n  {line}")?;
                }
                Ok(())
            }
            Self::UnexpectedOutput { command, stdout } => {
                write!(f, "unexpected output from `{command}`: {stdout:?}")
            }
        }
    }
}
