//! Command line syntax: what was written, before any of it is interpreted.
//!
//! List options accept both repeated flags and comma-separated values, so a
//! list can be written by hand or assembled by a shell.

pub mod branch;
pub mod commit;

use std::ffi::OsString;

use clap::{Args, Parser, Subcommand};

use crate::resolve::DEFAULT_TYPES;
use branch::BranchClapArgs;
use commit::CommitClapArgs;

/// A command line that follows the syntax of the commands.
#[derive(Debug)]
pub enum Parsed {
    Command(ClapCommand),
    /// Help or the version, which clap gives as an error to be printed.
    Info(clap::Error),
}

impl Parsed {
    /// Parses `args`, which exclude the program name, failing on a syntax
    /// error.
    pub fn parse(args: Vec<OsString>) -> Result<Self, clap::Error> {
        match Cli::try_parse_from(args) {
            Ok(cli) => Ok(Self::Command(cli.command)),
            Err(err) if err.use_stderr() => Err(err),
            Err(info) => Ok(Self::Info(info)),
        }
    }
}

#[derive(Debug, Parser)]
#[command(
    name = "vcs-lint",
    bin_name = "vcs-lint",
    version,
    about,
    no_binary_name = true
)]
struct Cli {
    #[command(subcommand)]
    command: ClapCommand,
}

#[derive(Debug, Subcommand)]
pub enum ClapCommand {
    /// Lint a branch name of the form `<type>/<name>`
    Branch(BranchClapArgs),
    /// Lint the header of a commit message file
    Commit(CommitClapArgs),
}

#[derive(Debug, Args)]
struct TypeClapArgs {
    #[arg(
        long = "type",
        value_name = "TYPE",
        value_delimiter = ',',
        help = TypeClapArgs::help(),
    )]
    types: Option<Vec<String>>,
}

impl TypeClapArgs {
    fn help() -> String {
        format!(
            "Allowed type, given as repeated flags or comma-separated values\n[default: {}]",
            DEFAULT_TYPES.join(", ")
        )
    }
}

#[cfg(test)]
#[path = "cli_tests.rs"]
mod tests;
