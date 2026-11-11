//! How a run ends: what it prints and the exit status it ends with.

pub mod branch;
pub mod commit;
pub mod config_file;
pub mod git;
pub mod hint;
pub mod history;
pub mod list;
pub mod version;

use std::{fmt::Display, process::ExitCode};

use branch::{BranchLintError, BranchResolveError};
use commit::{CommitLintError, CommitResolveError};
use config_file::ConfigFileError;
use hint::Hinted;
use history::{HistoryLintError, HistoryResolveError};
use version::RequiredVersionError;

use crate::{
    cli::SeverityClapLevel,
    hint::Hints,
    lint::{branch::BranchRule, commit::CommitRule, history::HistoryRule},
};

/// How a run ends, before anything is printed.
#[derive(Debug)]
#[must_use = "nothing is printed until the status is reported"]
pub enum RunStatus {
    /// Nothing breaks a rule. Exits with `0`.
    Success,
    /// Help or the version was asked for. Printed to standard output, exits
    /// with `0`.
    Info(clap::Error),
    /// The running vcs-lint does not meet the required version, so the
    /// command line is not read. Exits with `3`.
    RequiredVersionError(RequiredVersionError),
    /// The command line does not follow the syntax of the commands. Exits
    /// with `2`.
    SyntaxError(clap::Error),
    /// The config file cannot be read, so nothing is linted. Exits with `2`.
    ConfigFileError(ConfigFileError),
    /// The rules or the branch to lint cannot be resolved, so nothing is
    /// linted. Exits with `2`.
    BranchResolveError(BranchResolveError),
    /// The commit message or the rules cannot be resolved, so nothing is
    /// linted. Exits with `2`.
    CommitResolveError(CommitResolveError),
    /// The revision or the tip of its base cannot be resolved, so nothing is
    /// linted. Exits with `2`.
    HistoryResolveError(HistoryResolveError),
    /// The branch name breaks a rule. Exits with `1`.
    BranchLintError(Hinted<BranchLintError>),
    /// The commit message breaks a rule. Exits with `1`.
    CommitLintError(Hinted<CommitLintError>),
    /// The history of the revision breaks a rule. Exits with `1`.
    HistoryLintError(Hinted<HistoryLintError>),
    /// The branch name breaks a rule, reported as a warning. Exits with `0`.
    BranchLintWarning(Hinted<BranchLintError>),
    /// The commit message breaks a rule, reported as a warning. Exits with
    /// `0`.
    CommitLintWarning(Hinted<CommitLintError>),
    /// The history of the revision breaks a rule, reported as a warning.
    /// Exits with `0`.
    HistoryLintWarning(Hinted<HistoryLintError>),
}

/// The statuses a run of one command can end with once its command line is
/// read: why what it refers to cannot be resolved, and a violation reported as
/// an error or as a warning, followed by the hints of the rules `B` it breaks.
pub struct CommandStatuses<R, L, B> {
    resolve_error: fn(R) -> RunStatus,
    lint_error: fn(Hinted<L>) -> RunStatus,
    lint_warning: fn(Hinted<L>) -> RunStatus,
    broken_rules: fn(&L) -> Vec<B>,
}

impl RunStatus {
    /// Prints how the run ends and returns its exit status.
    pub fn report(self) -> ExitCode {
        match self {
            Self::Success => ExitCode::SUCCESS,
            Self::Info(info) => {
                let _ = info.print();
                ExitCode::SUCCESS
            }
            Self::RequiredVersionError(err) => Self::fail(err, 3),
            Self::SyntaxError(err) => {
                let _ = err.print();
                ExitCode::from(2)
            }
            Self::ConfigFileError(err) => Self::fail(err, 2),
            Self::BranchResolveError(err) => Self::fail(err, 2),
            Self::CommitResolveError(err) => Self::fail(err, 2),
            Self::HistoryResolveError(err) => Self::fail(err, 2),
            Self::BranchLintError(err) => Self::fail(err, 1),
            Self::CommitLintError(err) => Self::fail(err, 1),
            Self::HistoryLintError(err) => Self::fail(err, 1),
            Self::BranchLintWarning(err) => Self::warn(err),
            Self::CommitLintWarning(err) => Self::warn(err),
            Self::HistoryLintWarning(err) => Self::warn(err),
        }
    }

    fn fail(err: impl Display, code: u8) -> ExitCode {
        eprintln!("error: {err}");
        ExitCode::from(code)
    }

    fn warn(err: impl Display) -> ExitCode {
        eprintln!("warning: {err}");
        ExitCode::SUCCESS
    }
}

impl<R, L, B: PartialEq> CommandStatuses<R, L, B> {
    /// How the run ends with `linted`, the result of linting what the command
    /// refers to or why it cannot be resolved, a violation being reported as
    /// `severity` says and followed by those of `hints` given to the rules it
    /// breaks.
    pub fn end(
        &self,
        linted: Result<Result<(), L>, R>,
        severity: SeverityClapLevel,
        hints: &Hints<B>,
    ) -> RunStatus {
        match linted {
            Ok(Ok(())) => RunStatus::Success,
            Ok(Err(err)) => {
                let hinted = Hinted {
                    hints: hints.for_rules(&(self.broken_rules)(&err)),
                    report: err,
                };
                match severity {
                    SeverityClapLevel::Error => (self.lint_error)(hinted),
                    SeverityClapLevel::Warning => (self.lint_warning)(hinted),
                }
            }
            Err(err) => (self.resolve_error)(err),
        }
    }
}

impl CommandStatuses<BranchResolveError, BranchLintError, BranchRule> {
    pub const BRANCH: Self = Self {
        resolve_error: RunStatus::BranchResolveError,
        lint_error: RunStatus::BranchLintError,
        lint_warning: RunStatus::BranchLintWarning,
        broken_rules: BranchLintError::broken_rules,
    };
}

impl CommandStatuses<CommitResolveError, CommitLintError, CommitRule> {
    pub const COMMIT: Self = Self {
        resolve_error: RunStatus::CommitResolveError,
        lint_error: RunStatus::CommitLintError,
        lint_warning: RunStatus::CommitLintWarning,
        broken_rules: CommitLintError::broken_rules,
    };
}

impl CommandStatuses<HistoryResolveError, HistoryLintError, HistoryRule> {
    pub const HISTORY: Self = Self {
        resolve_error: RunStatus::HistoryResolveError,
        lint_error: RunStatus::HistoryLintError,
        lint_warning: RunStatus::HistoryLintWarning,
        broken_rules: HistoryLintError::broken_rules,
    };
}
