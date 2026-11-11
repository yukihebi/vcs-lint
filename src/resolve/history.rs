//! Interpretation of the `history` command.

use crate::{
    cli::{NetworkClapAccess, history::HistoryBaseFromClapSource},
    git::{Git, object_id::GitObjectId},
    lint::history::{
        HistoryBase, HistoryBaseSource, HistoryLintRules, HistoryLintTarget, HistoryTip,
    },
    report::{git::GitError, history::HistoryResolveError},
};

/// The `history` command line as parsed, before it is interpreted.
#[derive(Debug, PartialEq, Eq)]
pub struct HistoryRawConfig {
    /// The revision given, or `None` to lint `HEAD`.
    pub rev: Option<String>,
    pub base: Option<String>,
    pub base_from: Option<HistoryBaseFromClapSource>,
    pub remote: Option<String>,
    pub network: Option<NetworkClapAccess>,
}

impl HistoryRawConfig {
    const DEFAULT_REV: &str = "HEAD";
    const DEFAULT_REMOTE: &str = "origin";

    /// Resolves the base branch before reading anything from `git`, then the
    /// revision, then the tip of the base.
    ///
    /// A tip not found in the history of the revision cannot be told from one
    /// in history that a shallow repository is missing, so it fails there.
    pub fn resolve(
        self,
        git: &Git,
    ) -> Result<(HistoryLintTarget, HistoryLintRules), HistoryResolveError> {
        let base = Self::resolve_base(self.base, self.base_from, self.remote, self.network)?;
        let ref_name = base.ref_name();
        if !git.is_ref_name(&ref_name)? {
            return Err(HistoryResolveError::InvalidBase { base, ref_name });
        }
        let rev = self.rev.unwrap_or_else(|| Self::DEFAULT_REV.to_owned());
        let Some(commit) = git.commit(&rev)? else {
            return Err(HistoryResolveError::UnresolvableRev { rev });
        };
        let Some(tip) = Self::read_tip(&base, &ref_name, git)? else {
            return Err(HistoryResolveError::BaseNotFound { base });
        };
        let target = HistoryLintTarget {
            tip: Self::locate_tip(tip, &commit, git)?,
            rev,
        };
        if !target.contains_tip() && git.is_shallow()? {
            return Err(HistoryResolveError::Shallow {
                rev: target.rev,
                base,
            });
        }
        Ok((target, HistoryLintRules { base }))
    }

    /// Without `--base-from`, the base is read from the remote, `origin`
    /// without `--remote`; without `--network`, the network may be accessed
    /// to read it.
    fn resolve_base(
        branch: Option<String>,
        base_from: Option<HistoryBaseFromClapSource>,
        remote: Option<String>,
        network: Option<NetworkClapAccess>,
    ) -> Result<HistoryBase, HistoryResolveError> {
        let branch = branch.ok_or(HistoryResolveError::MissingBase)?;
        let remote = || remote.unwrap_or_else(|| Self::DEFAULT_REMOTE.to_owned());
        let source = match (
            base_from.unwrap_or(HistoryBaseFromClapSource::Remote),
            network.unwrap_or(NetworkClapAccess::Allow),
        ) {
            (HistoryBaseFromClapSource::Local, _) => HistoryBaseSource::Local,
            (HistoryBaseFromClapSource::Remote, NetworkClapAccess::Allow) => {
                HistoryBaseSource::Remote { remote: remote() }
            }
            (HistoryBaseFromClapSource::Remote, NetworkClapAccess::Deny) => {
                HistoryBaseSource::RemoteTracking { remote: remote() }
            }
        };
        Ok(HistoryBase { branch, source })
    }

    /// The object the ref `ref_name` of `base` points to, `None` when there
    /// is no such ref where it is read from.
    fn read_tip(
        base: &HistoryBase,
        ref_name: &str,
        git: &Git,
    ) -> Result<Option<GitObjectId>, GitError> {
        match &base.source {
            HistoryBaseSource::Local | HistoryBaseSource::RemoteTracking { .. } => {
                git.read_ref(ref_name)
            }
            HistoryBaseSource::Remote { remote } => git.read_remote_ref(remote, ref_name),
        }
    }

    /// Finds `tip` from the history of `commit`, counting the commits it is
    /// missing when the tip is in the repository.
    fn locate_tip(
        tip: GitObjectId,
        commit: &GitObjectId,
        git: &Git,
    ) -> Result<HistoryTip, GitError> {
        Ok(match git.commit(tip.as_str())? {
            Some(tip) => HistoryTip::Local {
                missing: git.count_missing(&tip, commit)?,
            },
            None => HistoryTip::NotLocal { id: tip },
        })
    }
}

#[cfg(test)]
#[path = "history_tests.rs"]
mod tests;
