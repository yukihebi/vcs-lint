//! Syntax of the `branch` command.

use clap::{Args, ValueEnum};

use super::{ListClapValues, TypeClapArgs};
use crate::resolve::branch::BranchRawConfig;

#[derive(Debug, Args)]
pub struct BranchClapArgs {
    /// Branch name to lint, or `HEAD` for a detached HEAD; the current branch
    /// when omitted
    #[arg(value_name = "NAME")]
    branch: Option<String>,
    #[command(flatten)]
    types: TypeClapArgs,
    /// Branch that must not be committed to directly, given as repeated flags
    /// or comma-separated values
    #[arg(
        long = "protected",
        value_name = "BRANCH",
        value_parser = ListClapValues::parse
    )]
    protected: Option<Vec<ListClapValues>>,
    /// Directory whose subdirectories in the git index are protected branches,
    /// given as repeated flags or comma-separated values
    #[arg(
        long = "protected-dir",
        value_name = "DIR",
        value_parser = ListClapValues::parse
    )]
    protected_dirs: Option<Vec<ListClapValues>>,
    /// Branch excluded from the protected branches, given as repeated flags or
    /// comma-separated values
    #[arg(
        long = "protected-exclude",
        value_name = "BRANCH",
        value_parser = ListClapValues::parse
    )]
    protected_excluded: Option<Vec<ListClapValues>>,
    #[arg(
        long = "detached",
        value_name = "RULE",
        help = "Whether a detached HEAD, given as `HEAD`, passes\n[default: allow]"
    )]
    detached: Option<BranchDetachedClapRule>,
}

/// Whether a detached HEAD passes, as written.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum BranchDetachedClapRule {
    /// A detached HEAD passes
    Allow,
    /// A detached HEAD fails
    Deny,
}

impl BranchClapArgs {
    pub fn into_raw_config(self) -> BranchRawConfig {
        let (types, type_defaults) = self.types.into_raw_values();
        BranchRawConfig {
            branch: self.branch,
            types,
            type_defaults,
            protected: ListClapValues::into_raw_values(
                self.protected,
                self.protected_dirs,
                self.protected_excluded,
            ),
            detached: self.detached,
        }
    }
}
