//! Command line syntax: what was written, before any of it is interpreted.
//!
//! List options accept both repeated flags and comma-separated values, so a
//! list can be written by hand or assembled by a shell. Each flag must give a
//! value, so a substitution that comes out empty is reported rather than
//! dropped. A list option comes with an option excluding values from it, named
//! after the list so that help shows them together.

pub mod branch;
pub mod commit;

use std::ffi::OsString;

use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::resolve::{DEFAULT_TYPES, list::ListRawValues};
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
        value_parser = ListClapValues::parse,
        help = TypeClapArgs::help(),
    )]
    types: Option<Vec<ListClapValues>>,
    /// Type excluded from the allowed types, given as repeated flags or
    /// comma-separated values
    #[arg(
        long = "type-exclude",
        value_name = "TYPE",
        value_parser = ListClapValues::parse
    )]
    excluded: Option<Vec<ListClapValues>>,
    #[arg(
        long = "type-defaults",
        value_name = "MODE",
        help = "When the default types are allowed\n[default: auto]"
    )]
    defaults: Option<TypeDefaultsClapMode>,
}

impl TypeClapArgs {
    fn help() -> String {
        format!(
            "Allowed type, given as repeated flags or comma-separated values\n[defaults: {}]",
            DEFAULT_TYPES.join(", ")
        )
    }

    fn into_raw_values(self) -> (ListRawValues, Option<TypeDefaultsClapMode>) {
        (
            ListClapValues::into_raw_values(self.types, self.excluded),
            self.defaults,
        )
    }
}

/// When the default types are allowed, as written.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum TypeDefaultsClapMode {
    /// Allowed only when no type is given
    Auto,
    /// Allowed besides the given types
    Always,
    /// Never allowed
    Never,
}

/// The values one flag of a list option gives: its comma-separated values,
/// trimmed of surrounding whitespace, without empty values; at least one is
/// left.
#[derive(Debug, Clone)]
struct ListClapValues(Vec<String>);

impl ListClapValues {
    /// Parses what one flag was given, failing when every value is empty.
    fn parse(written: &str) -> Result<Self, &'static str> {
        let values: Vec<String> = written
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(String::from)
            .collect();
        if values.is_empty() {
            Err("every value is empty")
        } else {
            Ok(Self(values))
        }
    }

    /// The values of every flag of a list option and of its exclusion option,
    /// in the order written.
    fn into_raw_values(given: Option<Vec<Self>>, excluded: Option<Vec<Self>>) -> ListRawValues {
        ListRawValues {
            given: Self::concat(given),
            excluded: Self::concat(excluded).unwrap_or_default(),
        }
    }

    /// The values of every flag of an option in the order written, `None`
    /// when the option is omitted.
    fn concat(flags: Option<Vec<Self>>) -> Option<Vec<String>> {
        flags.map(|flags| flags.into_iter().flat_map(|flag| flag.0).collect())
    }
}

#[cfg(test)]
#[path = "cli_tests.rs"]
mod tests;
