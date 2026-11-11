//! Command line syntax: what was written, before any of it is interpreted.
//!
//! List options accept both repeated flags and comma-separated values, so a
//! list can be written by hand or assembled by a shell. Each flag must give a
//! value, so a substitution that comes out empty is reported rather than
//! dropped. A list option comes with an option adding the names of the
//! subdirectories of directories to it and an option excluding values from it,
//! named after the list so that help shows them together.
//!
//! The options of a config file are parsed as part of the command line, given
//! right after the command name.

pub mod branch;
pub mod commit;

use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use clap::{ArgAction, Args, CommandFactory, Parser, Subcommand, ValueEnum};

use crate::resolve::{DEFAULT_TYPES, list::ListRawValues};
use branch::BranchClapArgs;
use commit::CommitClapArgs;

/// A command line that follows the syntax of the commands.
#[derive(Debug)]
pub enum Parsed {
    Command(Box<ClapCommand>),
    /// Help or the version, which clap gives as an error to be printed.
    Info(clap::Error),
}

impl Parsed {
    /// Parses `args`, which exclude the program name, failing on a syntax
    /// error.
    pub fn parse(args: Vec<OsString>) -> Result<Self, clap::Error> {
        match Cli::try_parse_from(args) {
            Ok(cli) => Ok(Self::Command(Box::new(cli.command))),
            Err(err) if err.use_stderr() => Err(err),
            Err(info) => Ok(Self::Info(info)),
        }
    }

    /// Parses `args` as [`Parsed::parse`] does, with `leading` given right
    /// after their first argument, the command name of a command line that
    /// parses into a [`Parsed::Command`].
    pub fn parse_with_leading(
        args: Vec<OsString>,
        leading: Vec<OsString>,
    ) -> Result<Self, clap::Error> {
        let mut args = args.into_iter();
        let name = args.next();
        Self::parse(name.into_iter().chain(leading).chain(args).collect())
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

impl ClapCommand {
    pub fn name(&self) -> CommandName {
        match self {
            Self::Branch(_) => CommandName::Branch,
            Self::Commit(_) => CommandName::Commit,
        }
    }

    /// The config file given with `--config`.
    pub fn config(&self) -> Option<&Path> {
        let config = match self {
            Self::Branch(args) => &args.config,
            Self::Commit(args) => &args.config,
        };
        config.path.as_deref()
    }
}

/// A command, named as on the command line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandName {
    Branch,
    Commit,
}

impl CommandName {
    pub const ALL: [Self; 2] = [Self::Branch, Self::Commit];

    /// The command named `name`, if any.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|command| command.as_str() == name)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Branch => "branch",
            Self::Commit => "commit",
        }
    }

    /// The options of the command that have a long name, `--help` aside.
    pub fn options(self) -> Vec<CommandOption> {
        let cli = Cli::command();
        let command = cli
            .find_subcommand(self.as_str())
            .expect("every command is a subcommand");
        command
            .get_arguments()
            .filter_map(|arg| {
                Some(CommandOption {
                    name: arg.get_long()?.to_owned(),
                    is_list: matches!(arg.get_action(), ArgAction::Append),
                })
            })
            .collect()
    }
}

/// An option of a command, by its long name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOption {
    pub name: String,
    /// Whether each flag of the option adds to its values, rather than the
    /// option being given once.
    pub is_list: bool,
}

#[derive(Debug, Args)]
struct ConfigClapArgs {
    /// TOML file whose options are read before the command line;
    /// `VCS_LINT_CONFIG` when omitted
    #[arg(long = "config", value_name = "PATH")]
    path: Option<PathBuf>,
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
    /// Directory whose subdirectories in the git index are allowed types,
    /// given as repeated flags or comma-separated values
    #[arg(
        long = "type-dir",
        value_name = "DIR",
        value_parser = ListClapValues::parse
    )]
    dirs: Option<Vec<ListClapValues>>,
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
            ListClapValues::into_raw_values(self.types, self.dirs, self.excluded),
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

    /// The values of every flag of a list option, of its option giving
    /// directories and of its exclusion option, in the order written.
    fn into_raw_values(
        given: Option<Vec<Self>>,
        dirs: Option<Vec<Self>>,
        excluded: Option<Vec<Self>>,
    ) -> ListRawValues {
        ListRawValues {
            given: Self::concat(given),
            dirs: Self::concat(dirs).map(|dirs| dirs.into_iter().map(PathBuf::from).collect()),
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
