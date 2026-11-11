//! How a run ends: what it prints and the exit status it ends with.

pub mod branch;
pub mod commit;
pub mod config_file;
pub mod git;
pub mod history;
pub mod list;
pub mod version;

use std::{fmt::Display, process::ExitCode};

use branch::{BranchLintError, BranchResolveError};
use commit::{CommitLintError, CommitResolveError};
use config_file::ConfigFileError;
use history::{HistoryLintError, HistoryResolveError};
use version::RequiredVersionError;

use crate::cli::SeverityClapLevel;

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
    BranchLintError(BranchLintError),
    /// The commit message breaks a rule. Exits with `1`.
    CommitLintError(CommitLintError),
    /// The history of the revision breaks a rule. Exits with `1`.
    HistoryLintError(HistoryLintError),
    /// The branch name breaks a rule, reported as a warning. Exits with `0`.
    BranchLintWarning(BranchLintError),
    /// The commit message breaks a rule, reported as a warning. Exits with
    /// `0`.
    CommitLintWarning(CommitLintError),
    /// The history of the revision breaks a rule, reported as a warning.
    /// Exits with `0`.
    HistoryLintWarning(HistoryLintError),
}

/// The statuses a run of one command can end with once its command line is
/// read: why what it refers to cannot be resolved, and a violation reported as
/// an error or as a warning.
pub struct CommandStatuses<R, L> {
    resolve_error: fn(R) -> RunStatus,
    lint_error: fn(L) -> RunStatus,
    lint_warning: fn(L) -> RunStatus,
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

impl<R, L> CommandStatuses<R, L> {
    /// How the run ends with `linted`, the result of linting what the command
    /// refers to or why it cannot be resolved, a violation being reported as
    /// `severity` says.
    pub fn end(&self, linted: Result<Result<(), L>, R>, severity: SeverityClapLevel) -> RunStatus {
        match linted {
            Ok(Ok(())) => RunStatus::Success,
            Ok(Err(err)) => match severity {
                SeverityClapLevel::Error => (self.lint_error)(err),
                SeverityClapLevel::Warning => (self.lint_warning)(err),
            },
            Err(err) => (self.resolve_error)(err),
        }
    }
}

impl CommandStatuses<BranchResolveError, BranchLintError> {
    pub const BRANCH: Self = Self {
        resolve_error: RunStatus::BranchResolveError,
        lint_error: RunStatus::BranchLintError,
        lint_warning: RunStatus::BranchLintWarning,
    };
}

impl CommandStatuses<CommitResolveError, CommitLintError> {
    pub const COMMIT: Self = Self {
        resolve_error: RunStatus::CommitResolveError,
        lint_error: RunStatus::CommitLintError,
        lint_warning: RunStatus::CommitLintWarning,
    };
}

impl CommandStatuses<HistoryResolveError, HistoryLintError> {
    pub const HISTORY: Self = Self {
        resolve_error: RunStatus::HistoryResolveError,
        lint_error: RunStatus::HistoryLintError,
        lint_warning: RunStatus::HistoryLintWarning,
    };
}
