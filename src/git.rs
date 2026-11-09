//! Reading git through the `git` executable, run as a child process so that
//! it sees the repository as git itself does, including the environment git
//! sets for hooks, such as `GIT_DIR` and `GIT_INDEX_FILE`.
//!
//! Git always runs in the current directory: with `GIT_DIR` set alone, as in
//! the hooks of a linked worktree, git takes the directory it runs in as the
//! top of the work tree.

mod command;
pub mod head;
mod index;

use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    str,
};

use crate::report::git::{GitError, GitSubdirsError};
use command::GitCommand;
use head::GitHead;
use index::GitIndexDir;

/// The `git` executable found on `PATH`, run in the current directory.
#[derive(Debug)]
pub struct Git;

impl Git {
    const PROGRAM: &str = "git";

    /// What `HEAD` of the repository points to, including a branch with no
    /// commit yet.
    pub fn head(&self) -> Result<GitHead, GitError> {
        let branch = self
            .command(&["symbolic-ref", "-q", "HEAD"])
            .read_optional(GitHead::parse_branch)?;
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

    /// Reads a path followed by a line break.
    fn parse_path(stdout: &[u8]) -> Option<PathBuf> {
        let path = str::from_utf8(stdout).ok()?.strip_suffix('\n')?;
        (!path.is_empty() && !path.contains('\n')).then(|| PathBuf::from(path))
    }

    fn command(&self, args: &[impl AsRef<OsStr>]) -> GitCommand {
        GitCommand::new(OsStr::new(Self::PROGRAM), args)
    }
}
