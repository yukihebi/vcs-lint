//! Why the history of a revision cannot be linted or breaks the rules, worded
//! for the user.

use std::fmt::{self, Display, Formatter};

use derive_more::Display;

use super::git::GitError;
use crate::{
    git::object_id::GitObjectId,
    lint::history::{HistoryBase, HistoryBaseSource},
};

/// Why the revision or the tip of its base cannot be resolved.
#[derive(Debug, PartialEq, Eq, Display)]
pub enum HistoryResolveError {
    #[display("no base branch is given\n  give one with `--base`")]
    MissingBase,
    /// The ref the tip would be read from is not a valid ref name.
    #[display("invalid base branch {base}\n  `{ref_name}` is not a valid ref name")]
    InvalidBase { base: HistoryBase, ref_name: String },
    #[display("`{rev}` cannot be resolved to a commit")]
    UnresolvableRev { rev: String },
    #[display("base branch {base} is not found")]
    BaseNotFound { base: HistoryBase },
    /// The tip is not found in the history of the revision, which a shallow
    /// repository may have only part of.
    #[display(
        "cannot tell whether `{rev}` contains the tip of base branch {base}\n  \
         the tip is not found in the history of a shallow repository, which may be missing commits\n  \
         fetch more history, e.g. with `git fetch --unshallow`"
    )]
    Shallow { rev: String, base: HistoryBase },
    #[display("{_0}")]
    Git(GitError),
}

impl From<GitError> for HistoryResolveError {
    fn from(err: GitError) -> Self {
        Self::Git(err)
    }
}

#[derive(Debug, PartialEq, Eq, Display)]
pub enum HistoryLintError {
    /// The tip is in the repository, but not in the history of the revision.
    #[display(
        "`{rev}` is behind base branch {base}\n  missing {} of the base",
        HistoryLintError::commits(*missing)
    )]
    Behind {
        rev: String,
        base: HistoryBase,
        missing: u64,
    },
    /// The tip, read from the remote, is not in the repository.
    #[display(
        "`{rev}` is behind base branch {base}\n  \
         the tip {id} is not in the local repository\n    \
         fetch it to see how many commits are missing"
    )]
    TipNotLocal {
        rev: String,
        base: HistoryBase,
        /// The object name of the tip.
        id: GitObjectId,
    },
}

impl HistoryLintError {
    fn commits(count: u64) -> String {
        match count {
            1 => "1 commit".to_owned(),
            count => format!("{count} commits"),
        }
    }
}

/// A base branch as named in messages, with where its tip is read from.
impl Display for HistoryBase {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let branch = &self.branch;
        match &self.source {
            HistoryBaseSource::Local => write!(f, "`{branch}`"),
            HistoryBaseSource::Remote { remote } => write!(f, "`{branch}` on `{remote}`"),
            HistoryBaseSource::RemoteTracking { remote } => write!(f, "`{remote}/{branch}`"),
        }
    }
}
