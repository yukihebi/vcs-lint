//! Syntax of the `history` command.

use clap::{Args, ValueEnum, builder::NonEmptyStringValueParser};

use super::{ConfigClapArgs, NetworkClapArgs, SeverityClapArgs};
use crate::{
    hint::{Hint, Hints},
    lint::history::HistoryRule,
    resolve::history::HistoryRawConfig,
};

#[derive(Debug, Args)]
pub struct HistoryClapArgs {
    /// Revision whose history is linted, such as a branch name or a commit
    /// hash; `HEAD` when omitted
    #[arg(value_name = "REV")]
    rev: Option<String>,
    /// Base branch whose tip the history must contain; required
    #[arg(
        long = "base",
        value_name = "BRANCH",
        value_parser = NonEmptyStringValueParser::new()
    )]
    base: Option<String>,
    #[arg(
        long = "base-from",
        value_name = "SOURCE",
        help = "Where the base branch is read from\n[default: remote]"
    )]
    base_from: Option<HistoryBaseClapSource>,
    #[arg(
        long = "remote",
        value_name = "NAME",
        value_parser = NonEmptyStringValueParser::new(),
        help = "Remote the base branch is read from with `--base-from remote`\n[default: origin]"
    )]
    remote: Option<String>,
    #[command(flatten)]
    network: NetworkClapArgs,
    /// Hint printed when the revision is behind the base branch
    #[arg(long = "base-hint", value_name = "TEXT", value_parser = Hint::parse)]
    base_hint: Option<Hint>,
    #[command(flatten)]
    pub(super) severity: SeverityClapArgs,
    #[command(flatten)]
    pub(super) config: ConfigClapArgs,
}

/// Where the base branch is read from, as written.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum HistoryBaseClapSource {
    /// The local branch
    Local,
    /// The branch on the remote given by `--remote`
    Remote,
}

impl HistoryClapArgs {
    /// The hints given to the rules of the command.
    pub fn hints(&self) -> Hints<HistoryRule> {
        Hints::new([(HistoryRule::Base, self.base_hint.clone())])
    }

    pub fn into_raw_config(self) -> HistoryRawConfig {
        HistoryRawConfig {
            rev: self.rev,
            base: self.base,
            base_from: self.base_from,
            remote: self.remote,
            network: self.network.access,
        }
    }
}
