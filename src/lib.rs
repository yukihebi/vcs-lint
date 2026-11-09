//! Lints branch names and commit message headers against the types and scopes
//! given on the command line.

use std::{ffi::OsString, process::ExitCode};

use cli::{ClapCommand, Parsed};
use report::RunStatus;

mod cli;
mod lint;
mod name_set;
mod report;
mod resolve;

/// Runs the command line `args`, which exclude the program name.
pub fn run(args: Vec<OsString>) -> ExitCode {
    run_status(args).report()
}

fn run_status(args: Vec<OsString>) -> RunStatus {
    match Parsed::parse(args) {
        Ok(Parsed::Command(command)) => run_command(command),
        Ok(Parsed::Info(info)) => RunStatus::Info(info),
        Err(err) => RunStatus::SyntaxError(err),
    }
}

/// Resolves and lints what `command` refers to.
fn run_command(command: ClapCommand) -> RunStatus {
    match command {
        ClapCommand::Branch(args) => {
            let (target, rules) = args.into_raw_config().resolve();
            match rules.lint(&target) {
                Ok(()) => RunStatus::Success,
                Err(err) => RunStatus::BranchLintError(err),
            }
        }
        ClapCommand::Commit(args) => {
            let (target, rules) = match args.into_raw_config().resolve() {
                Ok(resolved) => resolved,
                Err(err) => return RunStatus::CommitResolveError(err),
            };
            match rules.lint(&target) {
                Ok(()) => RunStatus::Success,
                Err(err) => RunStatus::CommitLintError(err),
            }
        }
    }
}
