//! Temporary directories in which the binary runs as a hook would, with git
//! isolated from the configuration and environment of the machine.

use std::{
    env,
    ffi::OsStr,
    path::Path,
    process::{Command, Output},
};

use tempfile::TempDir;

use super::BinaryOutput;

/// A temporary directory, outside any repository unless one is made in it.
pub struct Workdir(TempDir);

impl Workdir {
    pub fn new() -> Self {
        Self(TempDir::new().unwrap())
    }

    /// A repository whose current branch is `branch`, with no commit yet.
    pub fn repo(branch: &str) -> Self {
        let workdir = Self::new();
        workdir.git(&["init", "-q", "-b", branch]);
        workdir
    }

    pub fn path(&self) -> &Path {
        self.0.path()
    }

    /// Runs git with `args`, failing the test unless it succeeds.
    pub fn git(&self, args: &[&str]) {
        let Output { status, stderr, .. } = self.command("git").args(args).output().unwrap();
        assert!(
            status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&stderr)
        );
    }

    /// Commits nothing on the current branch.
    pub fn commit(&self) {
        self.git(&["commit", "-q", "--allow-empty", "-m", "commit"]);
    }

    /// Runs the binary with `args`.
    pub fn run(&self, args: &[&str]) -> BinaryOutput {
        BinaryOutput::of(&mut self.vcs_lint(args))
    }

    /// A command running the binary with `args`, to be adjusted before it
    /// runs.
    pub fn vcs_lint(&self, args: &[&str]) -> Command {
        let mut command = self.command(BinaryOutput::PROGRAM);
        command.args(args);
        command
    }

    /// A command running `program` in the directory, with only the
    /// environment git needs to run alone: no system or global configuration,
    /// messages in English, no repository found above the directory, and an
    /// identity to commit with.
    fn command(&self, program: impl AsRef<OsStr>) -> Command {
        let mut command = Command::new(program);
        command
            .current_dir(self.path())
            .env_clear()
            .env("PATH", env::var_os("PATH").unwrap_or_default())
            .env("HOME", self.path())
            .env("LC_ALL", "C")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CEILING_DIRECTORIES", self.path().parent().unwrap())
            .env("GIT_AUTHOR_NAME", "vcs-lint")
            .env("GIT_AUTHOR_EMAIL", "vcs-lint@example.com")
            .env("GIT_COMMITTER_NAME", "vcs-lint")
            .env("GIT_COMMITTER_EMAIL", "vcs-lint@example.com");
        command
    }
}
