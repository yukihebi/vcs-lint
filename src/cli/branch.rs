//! Syntax of the `branch` command.

use clap::{Args, ValueEnum};

use super::TypeClapArgs;
use crate::resolve::branch::BranchRawConfig;

#[derive(Debug, Args)]
pub struct BranchClapArgs {
    /// Branch name to lint
    #[arg(value_name = "NAME")]
    branch: String,
    #[command(flatten)]
    types: TypeClapArgs,
    /// Branch that must not be committed to directly, given as repeated flags
    /// or comma-separated values
    #[arg(long = "protected", value_name = "BRANCH", value_delimiter = ',')]
    protected: Option<Vec<String>>,
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
        BranchRawConfig {
            branch: self.branch,
            types: self.types.types,
            protected: self.protected,
            detached: self.detached,
        }
    }
}
