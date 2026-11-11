//! Lints branch names and commit message headers against the types and scopes
//! given on the command line or in a config file, and whether the history of a
//! revision contains the tip of its base branch.

use std::ffi::OsString;

use cli::{ClapCommand, Parsed};
use config_file::ConfigFile;
use env::Env;
use git::Git;
use report::CommandStatuses;
pub use report::RunStatus;
use version::Version;

mod cli;
mod config_file;
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
/// vcs-lint may not know the options the command line uses. When a config
/// file is given, the command line is parsed again with the flags the command
/// reads from the file given right after the command name.
pub fn run(args: Vec<OsString>, var: impl Fn(&str) -> Option<OsString>) -> RunStatus {
    let env = Env::read(var);
    if let Some(required) = &env.required_version
        && let Err(err) = Version::CURRENT.require(required)
    {
        return RunStatus::RequiredVersionError(err);
    }
    let parsed = Parsed::parse(args.clone());
    let Ok(Parsed::Command { command, written }) = &parsed else {
        return run_parsed(parsed);
    };
    let Some(path) = ConfigFile::path(command.config(), &env) else {
        return run_parsed(parsed);
    };
    match ConfigFile::read(path).and_then(|file| file.flags(command.name(), written)) {
        Ok(flags) => run_parsed(Parsed::parse_with_leading(args, flags)),
        Err(err) => RunStatus::ConfigFileError(err),
    }
}

/// Runs what the command line was parsed into.
fn run_parsed(parsed: Result<Parsed, clap::Error>) -> RunStatus {
    match parsed {
        Ok(Parsed::Command { command, .. }) => run_command(*command),
        Ok(Parsed::Info(info)) => RunStatus::Info(info),
        Err(err) => RunStatus::SyntaxError(err),
    }
}

/// Resolves and lints what `command` refers to.
fn run_command(command: ClapCommand) -> RunStatus {
    let severity = command.severity();
    match command {
        ClapCommand::Branch(args) => CommandStatuses::BRANCH.end(
            args.into_raw_config()
                .resolve(&Git)
                .map(|(target, rules)| rules.lint(&target)),
            severity,
        ),
        ClapCommand::Commit(args) => CommandStatuses::COMMIT.end(
            args.into_raw_config()
                .resolve(&Git)
                .map(|(target, rules)| rules.lint(&target)),
            severity,
        ),
        ClapCommand::History(args) => CommandStatuses::HISTORY.end(
            args.into_raw_config()
                .resolve(&Git)
                .map(|(target, rules)| rules.lint(&target)),
            severity,
        ),
    }
}
