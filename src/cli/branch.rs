//! Syntax of the `branch` command.

use clap::Args;

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
}

impl BranchClapArgs {
    pub fn into_raw_config(self) -> BranchRawConfig {
        BranchRawConfig {
            branch: self.branch,
            types: self.types.types,
            protected: self.protected,
        }
    }
}
