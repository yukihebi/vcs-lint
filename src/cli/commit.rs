//! Syntax of the `commit` command.

use std::{
    fmt::{self, Display, Formatter},
    path::PathBuf,
};

use clap::{Args, ValueEnum};

use super::{ConfigClapArgs, ListClapValues, SeverityClapArgs, TypeClapArgs};
use crate::resolve::commit::CommitRawConfig;

#[derive(Debug, Args)]
pub struct CommitClapArgs {
    /// Commit message file, such as the one given to the commit-msg hook
    file: PathBuf,
    #[command(flatten)]
    types: TypeClapArgs,
    /// Allowed scope, given as repeated flags or comma-separated values
    #[arg(
        long = "scope",
        value_name = "SCOPE",
        value_parser = ListClapValues::parse
    )]
    scopes: Option<Vec<ListClapValues>>,
    /// Directory whose subdirectories in the git index are allowed scopes,
    /// given as repeated flags or comma-separated values
    #[arg(
        long = "scope-dir",
        value_name = "DIR",
        value_parser = ListClapValues::parse
    )]
    scope_dirs: Option<Vec<ListClapValues>>,
    /// Scope excluded from the allowed scopes, given as repeated flags or
    /// comma-separated values
    #[arg(
        long = "scope-exclude",
        value_name = "SCOPE",
        value_parser = ListClapValues::parse
    )]
    scopes_excluded: Option<Vec<ListClapValues>>,
    #[arg(
        long = "scope-mode",
        value_name = "MODE",
        help = "Whether commit headers have a scope\n[default: auto]"
    )]
    scope_mode: Option<CommitScopeClapMode>,
    #[command(flatten)]
    pub(super) severity: SeverityClapArgs,
    #[command(flatten)]
    pub(super) config: ConfigClapArgs,
}

/// Whether commit headers have a scope, as written.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum CommitScopeClapMode {
    /// Required when any scope is given, forbidden otherwise
    Auto,
    /// Headers must have an allowed scope
    Required,
    /// Headers may have an allowed scope, or none
    Optional,
    /// Headers must not have a scope; `--scope` is ignored
    Forbidden,
}

/// Written as the value it is given with.
impl Display for CommitScopeClapMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let value = self.to_possible_value().expect("no mode is skipped");
        f.write_str(value.get_name())
    }
}

impl CommitClapArgs {
    pub fn into_raw_config(self) -> CommitRawConfig {
        let (types, type_defaults) = self.types.into_raw_values();
        CommitRawConfig {
            file: self.file,
            types,
            type_defaults,
            scopes: ListClapValues::into_raw_values(
                self.scopes,
                self.scope_dirs,
                self.scopes_excluded,
            ),
            scope_mode: self.scope_mode,
        }
    }
}
