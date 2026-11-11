//! Lints branch names and commit message headers against the types and scopes
//! given on the command line.

use std::ffi::OsString;

use cli::{ClapCommand, Parsed};
use env::Env;
use git::Git;
pub use report::RunStatus;
use version::Version;

mod cli;
mod env;
mod git;
mod lint;
mod name_set;
mod report;
mod resolve;
mod version;

/// Runs the command line `args`, which exclude the program name, with `var`
/// giving the value of the environment variable named by its argument, without
/// printing anything; [`RunStatus::report`] prints how it ends.
///
/// The required version is checked before anything else, as an older
/// vcs-lint may not know the options the command line uses.
pub fn run(args: Vec<OsString>, var: impl Fn(&str) -> Option<OsString>) -> RunStatus {
    let env = Env::read(var);
    if let Some(required) = &env.required_version
        && let Err(err) = Version::CURRENT.require(required)
    {
        return RunStatus::RequiredVersionError(err);
    }
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
            let (target, rules) = match args.into_raw_config().resolve(&Git) {
                Ok(resolved) => resolved,
                Err(err) => return RunStatus::BranchResolveError(err),
            };
            match rules.lint(&target) {
                Ok(()) => RunStatus::Success,
                Err(err) => RunStatus::BranchLintError(err),
            }
        }
        ClapCommand::Commit(args) => {
            let (target, rules) = match args.into_raw_config().resolve(&Git) {
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
