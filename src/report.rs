//! How a run ends: what it prints and the exit status it ends with.

pub mod branch;
pub mod commit;

use std::{fmt::Display, process::ExitCode};

use branch::BranchLintError;
use commit::{CommitLintError, CommitResolveError};

/// How a run ends, before anything is printed.
#[derive(Debug)]
pub enum RunStatus {
    /// Nothing breaks a rule. Exits with `0`.
    Success,
    /// Help or the version was asked for. Printed to standard output, exits
    /// with `0`.
    Info(clap::Error),
    /// The command line does not follow the syntax of the commands. Exits
    /// with `3`.
    SyntaxError(clap::Error),
    /// The commit message cannot be read, so it is not linted. Exits with `2`.
    CommitResolveError(CommitResolveError),
    /// The branch name breaks a rule. Exits with `1`.
    BranchLintError(BranchLintError),
    /// The commit message breaks a rule. Exits with `1`.
    CommitLintError(CommitLintError),
}

impl RunStatus {
    /// Prints how the run ends and returns its exit status.
    pub fn report(self) -> ExitCode {
        match self {
            Self::Success => ExitCode::SUCCESS,
            Self::Info(info) => {
                let _ = info.print();
                ExitCode::SUCCESS
            }
            Self::SyntaxError(err) => {
                let _ = err.print();
                ExitCode::from(3)
            }
            Self::CommitResolveError(err) => Self::fail(err, 2),
            Self::BranchLintError(err) => Self::fail(err, 1),
            Self::CommitLintError(err) => Self::fail(err, 1),
        }
    }

    fn fail(err: impl Display, code: u8) -> ExitCode {
        eprintln!("error: {err}");
        ExitCode::from(code)
    }
}
