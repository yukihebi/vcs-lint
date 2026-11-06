//! Lints branch names and commit message headers against the types and scopes
//! given on the command line.

use std::{ffi::OsString, process::ExitCode};

use clap::Parser;

use cli::{Cli, ReadError};
use rules::{BranchRules, CommitRules};

mod branch;
mod cli;
mod commit;
mod rules;
mod violations;

/// Runs the command line `args`, which exclude the program name.
pub fn run(args: Vec<OsString>) -> ExitCode {
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(err) => {
            let _ = err.print();
            return u8::try_from(err.exit_code()).map_or(ExitCode::FAILURE, ExitCode::from);
        }
    };
    match cli
        .into_lint()
        .map_err(Error::from)
        .and_then(|lint| lint.check())
    {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

/// An input together with the rules to check it against.
#[derive(Debug, PartialEq, Eq)]
enum Lint {
    Branch { name: String, rules: BranchRules },
    Commit { message: String, rules: CommitRules },
}

impl Lint {
    fn check(&self) -> Result<(), Error> {
        match self {
            Self::Branch { name, rules } => Ok(branch::lint(name, rules)?),
            Self::Commit { message, rules } => Ok(commit::lint(message, rules)?),
        }
    }
}

/// Any failure reported as `error: ...`.
#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    Read(#[from] ReadError),
    #[error(transparent)]
    Branch(#[from] branch::Error),
    #[error(transparent)]
    Commit(#[from] commit::Error),
}
