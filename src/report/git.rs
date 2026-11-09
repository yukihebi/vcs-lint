//! Why git cannot be read, worded for the user.

use std::{
    fmt::{self, Display, Formatter},
    io,
    process::ExitStatus,
};

/// Why a git command does not give what is asked of it.
#[derive(Debug)]
pub enum GitError {
    /// The `git` executable is not found.
    NotFound,
    /// The `git` executable is found but cannot be run.
    Spawn(io::Error),
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

impl Display for GitError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => write!(f, "git is not found\n  install git or add it to `PATH`"),
            Self::Spawn(err) => write!(f, "cannot run git: {err}"),
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
