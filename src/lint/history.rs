//! Lint for whether the history of a revision contains the tip of its base
//! branch.

use crate::{git::object_id::GitObjectId, report::history::HistoryLintError};

/// What a run lints: a revision, and where the tip of the base is found from
/// its history.
#[derive(Debug, PartialEq, Eq)]
pub struct HistoryLintTarget {
    /// The revision as given.
    pub rev: String,
    pub tip: HistoryTip,
}

/// The tip of the base branch, as found from the history of the revision.
#[derive(Debug, PartialEq, Eq)]
pub enum HistoryTip {
    /// The tip is in the repository; `missing` commits of its history, the
    /// tip included, are not in the history of the revision.
    Local { missing: u64 },
    /// The tip is not in the repository, so the revision cannot contain it.
    NotLocal { id: GitObjectId },
}

#[derive(Debug, PartialEq, Eq)]
pub struct HistoryLintRules {
    /// The branch whose tip the revision must contain.
    pub base: HistoryBase,
}

/// A base branch, and where its tip is read from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryBase {
    /// The branch name, without `refs/heads/`.
    pub branch: String,
    pub source: HistoryBaseSource,
}

/// Where the tip of a base branch is read from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HistoryBaseSource {
    /// The local branch.
    Local,
    /// The branch on `remote`, as the remote lists it.
    Remote { remote: String },
    /// The remote-tracking branch of `remote`, as last fetched.
    RemoteTracking { remote: String },
}

impl HistoryLintTarget {
    /// Whether the tip is the revision or one of its ancestors.
    pub fn contains_tip(&self) -> bool {
        matches!(self.tip, HistoryTip::Local { missing: 0 })
    }
}

impl HistoryLintRules {
    pub fn lint(&self, target: &HistoryLintTarget) -> Result<(), HistoryLintError> {
        if target.contains_tip() {
            return Ok(());
        }
        let rev = target.rev.clone();
        let base = self.base.clone();
        Err(match &target.tip {
            HistoryTip::Local { missing } => HistoryLintError::Behind {
                rev,
                base,
                missing: *missing,
            },
            HistoryTip::NotLocal { id } => HistoryLintError::TipNotLocal {
                rev,
                base,
                id: id.clone(),
            },
        })
    }
}

impl HistoryBase {
    /// The full name of the ref the tip is read from, on the remote for
    /// [`HistoryBaseSource::Remote`].
    pub fn ref_name(&self) -> String {
        let branch = &self.branch;
        match &self.source {
            HistoryBaseSource::Local | HistoryBaseSource::Remote { .. } => {
                format!("refs/heads/{branch}")
            }
            HistoryBaseSource::RemoteTracking { remote } => {
                format!("refs/remotes/{remote}/{branch}")
            }
        }
    }
}

#[cfg(test)]
#[path = "history_tests.rs"]
mod tests;
