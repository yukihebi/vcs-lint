//! Syntax of the `branch` command.

use clap::{Args, ValueEnum};

use super::{ConfigClapArgs, ListClapValues, SeverityClapArgs, TypeClapArgs};
use crate::{
    hint::{Hint, Hints},
    lint::branch::BranchRule,
    resolve::branch::BranchRawConfig,
};

#[derive(Debug, Args)]
pub struct BranchClapArgs {
    /// Branch name to lint, or `HEAD` for a detached HEAD; the current branch
    /// when omitted
    #[arg(value_name = "NAME")]
    branch: Option<String>,
    #[command(flatten)]
    types: TypeClapArgs,
    /// Hint printed when the name after the type is invalid
    #[arg(long = "format-hint", value_name = "TEXT", value_parser = Hint::parse)]
    format_hint: Option<Hint>,
    /// Branch that fails whatever its format, given as repeated flags or
    /// comma-separated values
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
    /// Hint printed when the branch is protected
    #[arg(long = "protected-hint", value_name = "TEXT", value_parser = Hint::parse)]
    protected_hint: Option<Hint>,
    #[arg(
        long = "detached",
        value_name = "RULE",
        help = "Whether a detached HEAD passes\n[default: allow]"
    )]
    detached: Option<BranchDetachedClapRule>,
    /// Hint printed when a detached HEAD fails
    #[arg(long = "detached-hint", value_name = "TEXT", value_parser = Hint::parse)]
    detached_hint: Option<Hint>,
    #[command(flatten)]
    pub(super) severity: SeverityClapArgs,
    #[command(flatten)]
    pub(super) config: ConfigClapArgs,
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
    /// The hints given to the rules of the command.
    pub fn hints(&self) -> Hints<BranchRule> {
        Hints::new([
            (BranchRule::Protected, self.protected_hint.clone()),
            (BranchRule::Detached, self.detached_hint.clone()),
            (BranchRule::Type, self.types.hint.clone()),
            (BranchRule::Format, self.format_hint.clone()),
        ])
    }

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
