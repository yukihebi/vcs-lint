//! Syntax of the `commit` command.

use std::path::PathBuf;

use clap::{Args, ValueEnum};

use super::{ListClapValues, TypeClapArgs};
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
        value_parser = ListClapValues::parse,
        required_if_eq_any = [("scope_mode", "required"), ("scope_mode", "optional")],
    )]
    scopes: Option<Vec<ListClapValues>>,
    #[arg(
        long = "scope-mode",
        value_name = "MODE",
        help = "Whether commit headers have a scope\n[default: auto]"
    )]
    scope_mode: Option<CommitScopeClapMode>,
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

impl CommitClapArgs {
    pub fn into_raw_config(self) -> CommitRawConfig {
        CommitRawConfig {
            file: self.file,
            types: ListClapValues::concat(self.types.types),
            scopes: ListClapValues::concat(self.scopes),
            scope_mode: self.scope_mode,
        }
    }
}
