//! Syntax of the `commit` command.

use std::path::PathBuf;

use clap::Args;

use super::{ListClapValues, TypeClapArgs};
use crate::resolve::commit::CommitRawConfig;

#[derive(Debug, Args)]
pub struct CommitClapArgs {
    /// Commit message file, such as the one given to the commit-msg hook
    file: PathBuf,
    #[command(flatten)]
    types: TypeClapArgs,
    /// Scope that commit headers must have, given as repeated flags or
    /// comma-separated values; without any, headers must not have a scope
    #[arg(
        long = "scope",
        value_name = "SCOPE",
        value_parser = ListClapValues::parse
    )]
    scopes: Option<Vec<ListClapValues>>,
}

impl CommitClapArgs {
    pub fn into_raw_config(self) -> CommitRawConfig {
        CommitRawConfig {
            file: self.file,
            types: ListClapValues::concat(self.types.types),
            scopes: ListClapValues::concat(self.scopes),
        }
    }
}
