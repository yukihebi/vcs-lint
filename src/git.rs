//! Reading git through the `git` executable, run as a child process so that
//! it sees the repository as git itself does, including the environment git
//! sets for hooks, such as `GIT_DIR` and `GIT_INDEX_FILE`.
//!
//! Git always runs in the current directory: with `GIT_DIR` set alone, as in
//! the hooks of a linked worktree, git takes the directory it runs in as the
//! top of the work tree.
//!
//! What the user writes is never given where git could read it as an option:
//! a revision is given on standard input, and a remote after
//! `--end-of-options`.

mod command;
pub mod head;
mod index;
pub mod object_id;
mod refs;

use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    str,
};

use crate::report::git::{GitError, GitSubdirsError};
use command::{GitAbsentStatus, GitCommand};
use head::GitHead;
use index::GitIndexDir;
use object_id::GitObjectId;
use refs::GitListedRef;

/// The `git` executable found on `PATH`, run in the current directory.
#[derive(Debug)]
pub struct Git;

impl Git {
    const PROGRAM: &str = "git";

    /// How `git ls-remote --exit-code` exits when no ref matches, after the
    /// transport may have printed warnings.
    const LS_REMOTE_ABSENT: GitAbsentStatus = GitAbsentStatus::with_stderr(2);

    /// What `HEAD` of the repository points to, including a branch with no
    /// commit yet.
    pub fn head(&self) -> Result<GitHead, GitError> {
        let branch = self
            .command(&["symbolic-ref", "-q", "HEAD"])
            .read_optional(GitAbsentStatus::QUIET, GitHead::parse_branch)?;
        Ok(branch.map_or(GitHead::Detached, GitHead::Branch))
    }

    /// The names of the directories directly under `dir` that the index holds
    /// entries in, submodules included, each once and ordered by their bytes.
    ///
    /// `dir` is resolved to its real path, which git matches the entries to,
    /// and which is compared with the top of the work tree to find how deep
    /// the directory is, so that however `dir` is written, its path is never
    /// normalized by hand.
    pub fn index_subdirs(&self, dir: &Path) -> Result<Vec<String>, GitSubdirsError> {
        let real = fs::canonicalize(dir).map_err(|err| GitSubdirsError::unresolvable(&err))?;
        if !real.is_dir() {
            return Err(GitSubdirsError::NotDirectory);
        }
        let top = self
            .command(&["rev-parse", "--show-toplevel"])
            .read(Self::parse_path)?;
        let relative = real
            .strip_prefix(&top)
            .map_err(|_| GitSubdirsError::OutsideWorkTree)?;
        let index_dir = GitIndexDir::new(relative);
        let args = ["ls-files", "-z", "-s", "--full-name", "--"].map(OsStr::new);
        let names = self
            .command(&[&args[..], &[real.as_os_str()]].concat())
            .read(|stdout| {
                let names = index_dir.parse_subdirs(stdout)?;
                Some(names.into_iter().map(<[u8]>::to_vec).collect::<Vec<_>>())
            })?;
        names
            .into_iter()
            .map(|name| {
                String::from_utf8(name).map_err(|err| {
                    GitSubdirsError::NonUtf8Name(String::from_utf8_lossy(err.as_bytes()).into())
                })
            })
            .collect()
    }

    /// The commit `rev` names, through the tag it names if any, or `None`
    /// when it names no commit in the repository.
    pub fn commit(&self, rev: &str) -> Result<Option<GitObjectId>, GitError> {
        match self.object(rev)? {
            Some(object) => self.peel_commit(&object),
            None => Ok(None),
        }
    }

    /// The commit `object` is or points to as a tag, or `None` when there is
    /// none in the repository.
    pub fn peel_commit(&self, object: &GitObjectId) -> Result<Option<GitObjectId>, GitError> {
        self.object(&format!("{object}^{{commit}}"))
    }

    /// The object `name` names in any form git reads, such as `HEAD~1` or
    /// `:/fix`, or `None` when it names no single object in the repository.
    ///
    /// A name holding a line break, which would be read as several names,
    /// names none.
    fn object(&self, name: &str) -> Result<Option<GitObjectId>, GitError> {
        if name.contains('\n') {
            return Ok(None);
        }
        self.command(&["cat-file", "--batch-check=%(objectname)"])
            .input(format!("{name}\n"))
            .read(|stdout| GitObjectId::parse_lookup(stdout, name))
    }

    /// Whether `name` is valid as the full name of a ref, such as
    /// `refs/heads/main`, which holds no character that git reads as revision
    /// syntax or as a pattern.
    pub fn is_ref_name(&self, name: &str) -> Result<bool, GitError> {
        let valid = self
            .command(&["check-ref-format", name])
            .read_optional(GitAbsentStatus::QUIET, |stdout| {
                stdout.is_empty().then_some(())
            })?;
        Ok(valid.is_some())
    }

    /// The object the ref named exactly `name` points to in the repository,
    /// or `None` when there is no such ref.
    pub fn read_ref(&self, name: &str) -> Result<Option<GitObjectId>, GitError> {
        let format = "--format=%(objectname)%09%(refname)";
        self.command(&["for-each-ref", format, name])
            .read(|stdout| GitListedRef::find(stdout, name))
    }

    /// The object the ref named exactly `name` points to on `remote`, a remote
    /// name or URL, or `None` when there is no such ref, read with
    /// `git ls-remote`, which accesses the network unless the remote is local.
    pub fn read_remote_ref(
        &self,
        remote: &str,
        name: &str,
    ) -> Result<Option<GitObjectId>, GitError> {
        let found = self
            .command(&["ls-remote", "--exit-code", "--end-of-options", remote, name])
            .read_optional(Self::LS_REMOTE_ABSENT, |stdout| {
                GitListedRef::find(stdout, name)
            })?;
        Ok(found.flatten())
    }

    /// The number of commits in the history of `commit`, itself included,
    /// that are not in the history of `history`.
    pub fn count_missing(
        &self,
        commit: &GitObjectId,
        history: &GitObjectId,
    ) -> Result<u64, GitError> {
        let range = format!("{history}..{commit}");
        self.command(&["rev-list", "--count", &range])
            .read(Self::parse_count)
    }

    /// Whether the repository is shallow, missing the history before some of
    /// its commits.
    pub fn is_shallow(&self) -> Result<bool, GitError> {
        self.command(&["rev-parse", "--is-shallow-repository"])
            .read(Self::parse_bool)
    }

    /// Reads `true` or `false` followed by a line break.
    fn parse_bool(stdout: &[u8]) -> Option<bool> {
        match stdout {
            b"true\n" => Some(true),
            b"false\n" => Some(false),
            _ => None,
        }
    }

    /// Reads a decimal number followed by a line break.
    fn parse_count(stdout: &[u8]) -> Option<u64> {
        let count = str::from_utf8(stdout).ok()?.strip_suffix('\n')?;
        let is_decimal = !count.is_empty() && count.bytes().all(|byte| byte.is_ascii_digit());
        if !is_decimal {
            return None;
        }
        count.parse().ok()
    }

    /// Reads a path followed by a line break.
    fn parse_path(stdout: &[u8]) -> Option<PathBuf> {
        let path = str::from_utf8(stdout).ok()?.strip_suffix('\n')?;
        (!path.is_empty() && !path.contains('\n')).then(|| PathBuf::from(path))
    }

    fn command(&self, args: &[impl AsRef<OsStr>]) -> GitCommand {
        GitCommand::new(OsStr::new(Self::PROGRAM), args)
    }
}

#[cfg(test)]
#[path = "git_tests.rs"]
mod tests;
