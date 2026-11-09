//! Lints branch names and commit message headers against the types and scopes
//! given on the command line.

use std::ffi::OsString;

use cli::{ClapCommand, Parsed};
pub use report::RunStatus;

mod cli;
#[allow(dead_code, reason = "no command reads from git yet")]
mod git;
mod lint;
mod name_set;
mod report;
mod resolve;

/// Runs the command line `args`, which exclude the program name, without
/// printing anything; [`RunStatus::report`] prints how it ends.
pub fn run(args: Vec<OsString>) -> RunStatus {
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
            let (target, rules) = match args.into_raw_config().resolve() {
                Ok(resolved) => resolved,
                Err(err) => return RunStatus::BranchResolveError(err),
            };
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
