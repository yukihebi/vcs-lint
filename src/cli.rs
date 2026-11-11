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
pub mod history;

use std::{
    error::Error,
    ffi::OsString,
    path::{Path, PathBuf},
};

use clap::{
    ArgAction, ArgMatches, Args, CommandFactory, FromArgMatches, Parser, Subcommand, ValueEnum,
    parser::ValueSource,
};

use crate::{
    report::config_file::ValueRejection,
    resolve::{DEFAULT_TYPES, list::ListRawValues},
};
use branch::BranchClapArgs;
use commit::CommitClapArgs;
use history::HistoryClapArgs;

/// A command line that follows the syntax of the commands.
#[derive(Debug)]
pub enum Parsed {
    Command {
        command: Box<ClapCommand>,
        /// The long names of the options written on the command line.
        written: Vec<String>,
    },
    /// Help or the version, which clap gives as an error to be printed.
    Info(clap::Error),
}

impl Parsed {
    /// Parses `args`, which exclude the program name, failing on a syntax
    /// error.
    pub fn parse(args: Vec<OsString>) -> Result<Self, clap::Error> {
        let mut cli = Cli::command();
        let mut matches = match cli.try_get_matches_from_mut(args) {
            Ok(matches) => matches,
            Err(err) if err.use_stderr() => return Err(err),
            Err(info) => return Ok(Self::Info(info)),
        };
        let written = Self::written(&cli, &matches);
        let parsed = Cli::from_arg_matches_mut(&mut matches).map_err(|err| err.format(&mut cli))?;
        Ok(Self::Command {
            command: Box::new(parsed.command),
            written,
        })
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

    /// The long names of the options that `matches`, matched against `cli`,
    /// found written for its command.
    fn written(cli: &clap::Command, matches: &ArgMatches) -> Vec<String> {
        let Some((name, matches)) = matches.subcommand() else {
            return Vec::new();
        };
        cli.find_subcommand(name)
            .into_iter()
            .flat_map(clap::Command::get_arguments)
            .filter(|arg| {
                matches.value_source(arg.get_id().as_str()) == Some(ValueSource::CommandLine)
            })
            .filter_map(|arg| arg.get_long().map(String::from))
            .collect()
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
    /// Lint whether the history of a revision contains the tip of its base
    /// branch
    History(HistoryClapArgs),
}

impl ClapCommand {
    pub fn name(&self) -> CommandName {
        match self {
            Self::Branch(_) => CommandName::Branch,
            Self::Commit(_) => CommandName::Commit,
            Self::History(_) => CommandName::History,
        }
    }

    /// How violations are reported, `error` when `--severity` is omitted.
    pub fn severity(&self) -> SeverityClapLevel {
        let severity = match self {
            Self::Branch(args) => &args.severity,
            Self::Commit(args) => &args.severity,
            Self::History(args) => &args.severity,
        };
        severity.level.unwrap_or_default()
    }

    /// The config file given with `--config`.
    pub fn config(&self) -> Option<&Path> {
        let config = match self {
            Self::Branch(args) => &args.config,
            Self::Commit(args) => &args.config,
            Self::History(args) => &args.config,
        };
        config.path.as_deref()
    }
}

/// A command, named as on the command line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandName {
    Branch,
    Commit,
    History,
}

impl CommandName {
    pub const ALL: [Self; 3] = [Self::Branch, Self::Commit, Self::History];

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
            Self::History => "history",
        }
    }

    /// The syntax of the command.
    pub fn syntax(self) -> CommandSyntax {
        let cli = Cli::command();
        let command = cli
            .find_subcommand(self.as_str())
            .expect("every command is a subcommand");
        CommandSyntax(command.clone())
    }
}

/// The syntax of a command, against which values of its options are checked.
#[derive(Debug)]
pub struct CommandSyntax(clap::Command);

impl CommandSyntax {
    /// The options of the command that have a long name, `--help` aside.
    pub fn options(&self) -> impl Iterator<Item = CommandOption<'_>> {
        self.0.get_arguments().filter_map(|arg| {
            Some(CommandOption {
                arg,
                name: arg.get_long()?,
            })
        })
    }

    /// The option whose long name is `name`, if any.
    pub fn option(&self, name: &str) -> Option<CommandOption<'_>> {
        self.options().find(|option| option.name == name)
    }
}

/// An option of a command that has a long name.
#[derive(Debug, Clone, Copy)]
pub struct CommandOption<'a> {
    arg: &'a clap::Arg,
    pub name: &'a str,
}

impl CommandOption<'_> {
    /// Whether each flag of the option adds to its values, rather than the
    /// option being given once.
    pub fn is_list(&self) -> bool {
        matches!(self.arg.get_action(), ArgAction::Append)
    }

    /// Checks `value` as the command line checks the value of one flag of the
    /// option, parsing the flag on a command that has the option alone.
    pub fn check(&self, value: &str) -> Result<(), ValueRejection> {
        let command = clap::Command::new("vcs-lint")
            .no_binary_name(true)
            .arg(self.arg.clone());
        let Err(err) = command.try_get_matches_from([format!("--{}={value}", self.name)]) else {
            return Ok(());
        };
        let possible: Vec<String> = self
            .arg
            .get_possible_values()
            .iter()
            .map(|value| value.get_name().to_owned())
            .collect();
        if possible.is_empty() {
            let reason = err
                .source()
                .map_or_else(|| err.kind().to_string(), ToString::to_string);
            Err(ValueRejection::Invalid { reason })
        } else {
            Err(ValueRejection::NotPossible { possible })
        }
    }
}

#[derive(Debug, Args)]
struct SeverityClapArgs {
    #[arg(
        long = "severity",
        value_name = "LEVEL",
        help = "How violations are reported\n[default: error]"
    )]
    level: Option<SeverityClapLevel>,
}

/// How violations are reported, as written.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
pub enum SeverityClapLevel {
    /// Violations are reported as errors and fail the run
    #[default]
    Error,
    /// Violations are reported as warnings and do not fail the run
    Warning,
}

#[derive(Debug, Args)]
struct NetworkClapArgs {
    #[arg(
        long = "network",
        value_name = "ACCESS",
        help = "Whether vcs-lint may access the network\n[default: allow]"
    )]
    access: Option<NetworkClapAccess>,
}

/// Whether vcs-lint may access the network, as written.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum NetworkClapAccess {
    /// vcs-lint may access the network
    Allow,
    /// vcs-lint never accesses the network
    Deny,
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
