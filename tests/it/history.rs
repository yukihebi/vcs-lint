//! `vcs-lint history`, run in a clone of a bare remote that another clone
//! pushes to.

use insta::assert_snapshot;
use regex::Regex;

use super::{BinaryOutput, workdir::Workdir};

/// A bare remote whose `main` starts with one commit, a clone of it on
/// `feat/x` one commit ahead of `main`, and another clone that pushes to it.
struct Clones {
    remote: Workdir,
    /// The clone vcs-lint runs in.
    repo: Workdir,
    other: Workdir,
}

impl Clones {
    fn new() -> Self {
        let remote = Workdir::new();
        remote.git(&["init", "-q", "--bare", "-b", "main"]);
        let seed = Workdir::repo("main");
        Self::commit(&seed, "init");
        seed.git(&["push", "-q", &Self::url(&remote), "main"]);
        let clones = Self {
            repo: Self::clone(&remote, &[]),
            other: Self::clone(&remote, &[]),
            remote,
        };
        clones.repo.git(&["switch", "-q", "-c", "feat/x"]);
        Self::commit(&clones.repo, "feat");
        clones
    }

    /// A clone of `remote` with `options` given to `git clone`.
    fn clone(remote: &Workdir, options: &[&str]) -> Workdir {
        let clone = Workdir::new();
        let url = Self::url(remote);
        clone.git(&[&["clone", "-q"], options, &[&url, "."]].concat());
        clone
    }

    /// The URL of `remote`, through which a clone can be shallow.
    fn url(remote: &Workdir) -> String {
        format!("file://{}", remote.path().display())
    }

    /// Commits nothing on the current branch with `message`, which tells the
    /// commit apart from those made in other clones at the same time.
    fn commit(workdir: &Workdir, message: &str) {
        workdir.git(&["commit", "-q", "--allow-empty", "-m", message]);
    }

    /// Pushes `count` new commits on `main` of the remote from the other
    /// clone.
    fn advance_remote(&self, count: usize) {
        for i in 0..count {
            Self::commit(&self.other, &format!("main {i}"));
        }
        self.other.git(&["push", "-q", "origin", "main"]);
    }

    fn run(&self, args: &[&str]) -> BinaryOutput {
        self.repo.run(args)
    }
}

/// Renders `output` with the object names in it replaced, as they change with
/// the time commits are made.
fn redact(output: &BinaryOutput) -> String {
    Regex::new(r"\b[0-9a-f]{40}\b")
        .unwrap()
        .replace_all(&output.to_string(), "<id>")
        .into_owned()
}

#[test]
fn accepts_revision_containing_tip_of_base() {
    let clones = Clones::new();
    assert_snapshot!(clones.run(&["history", "--base", "main"]), @"
    exit: 0
    --- stdout
    --- stderr
    ");
}

#[test]
fn accepts_revision_updated_by_merge_or_rebase() {
    let clones = Clones::new();
    clones.advance_remote(1);
    clones.repo.git(&["fetch", "-q"]);
    clones.repo.git(&["switch", "-q", "-c", "feat/merged"]);
    clones
        .repo
        .git(&["merge", "-q", "--no-edit", "origin/main"]);
    clones.repo.git(&["switch", "-q", "feat/x"]);
    clones.repo.git(&["rebase", "-q", "origin/main"]);
    assert_snapshot!(clones.run(&["history", "feat/merged", "--base", "main"]), @"
    exit: 0
    --- stdout
    --- stderr
    ");
    assert_snapshot!(clones.run(&["history", "feat/x", "--base", "main"]), @"
    exit: 0
    --- stdout
    --- stderr
    ");
}

#[test]
fn rejects_revision_behind_fetched_tip_by_missing_commits() {
    let clones = Clones::new();
    clones.advance_remote(2);
    clones.repo.git(&["fetch", "-q"]);
    assert_snapshot!(clones.run(&["history", "--base", "main"]), @"
    exit: 1
    --- stdout
    --- stderr
    error: `HEAD` is behind base branch `main` on `origin`
      missing 2 commits of the base
    ");
    assert_snapshot!(clones.run(&["history", "--base", "main", "--network", "deny"]), @"
    exit: 1
    --- stdout
    --- stderr
    error: `HEAD` is behind base branch `origin/main`
      missing 2 commits of the base
    ");
}

#[test]
fn rejects_tip_on_remote_missing_from_repository() {
    let clones = Clones::new();
    clones.advance_remote(1);
    assert_snapshot!(redact(&clones.run(&["history", "--base", "main"])), @"
    exit: 1
    --- stdout
    --- stderr
    error: `HEAD` is behind base branch `main` on `origin`
      the tip <id> is not in the local repository
        fetch it to see how many commits are missing
    ");
}

#[test]
fn reads_remote_tracking_branch_as_last_fetched_without_network() {
    let clones = Clones::new();
    clones.advance_remote(1);
    assert_snapshot!(clones.run(&["history", "--base", "main", "--network", "deny"]), @"
    exit: 0
    --- stdout
    --- stderr
    ");
}

#[test]
fn reads_local_base_branch() {
    let clones = Clones::new();
    clones.advance_remote(1);
    assert_snapshot!(clones.run(&["history", "--base", "main", "--base-from", "local"]), @"
    exit: 0
    --- stdout
    --- stderr
    ");
    clones.repo.git(&["switch", "-q", "main"]);
    Clones::commit(&clones.repo, "local");
    assert_snapshot!(clones.run(&["history", "feat/x", "--base", "main", "--base-from", "local"]), @"
    exit: 1
    --- stdout
    --- stderr
    error: `feat/x` is behind base branch `main`
      missing 1 commit of the base
    ");
}

#[test]
fn lints_revision_in_any_form_git_reads() {
    let clones = Clones::new();
    assert_snapshot!(clones.run(&["history", ":/feat", "--base", "main"]), @"
    exit: 0
    --- stdout
    --- stderr
    ");
    clones.repo.git(&["tag", "-a", "-m", "tag", "v1"]);
    assert_snapshot!(clones.run(&["history", "v1", "--base", "main"]), @"
    exit: 0
    --- stdout
    --- stderr
    ");
    assert_snapshot!(clones.run(&["history", "HEAD:", "--base", "main"]), @"
    exit: 2
    --- stdout
    --- stderr
    error: `HEAD:` cannot be resolved to a commit
    ");
}

#[test]
fn lints_revision_given_as_commit_whatever_branch() {
    let clones = Clones::new();
    clones.repo.git(&["switch", "-q", "--detach", "HEAD~1"]);
    assert_snapshot!(clones.run(&["history", "--base", "main"]), @"
    exit: 0
    --- stdout
    --- stderr
    ");
    clones.advance_remote(1);
    clones.repo.git(&["fetch", "-q"]);
    assert_snapshot!(clones.run(&["history", "feat/x~1", "--base", "main"]), @"
    exit: 1
    --- stdout
    --- stderr
    error: `feat/x~1` is behind base branch `main` on `origin`
      missing 1 commit of the base
    ");
}

#[test]
fn reports_violation_as_warning() {
    let clones = Clones::new();
    clones.advance_remote(1);
    clones.repo.git(&["fetch", "-q"]);
    assert_snapshot!(clones.run(&["history", "--base", "main", "--severity", "warning"]), @"
    exit: 0
    --- stdout
    --- stderr
    warning: `HEAD` is behind base branch `main` on `origin`
      missing 1 commit of the base
    ");
}

#[test]
fn reads_base_from_config_file() {
    let clones = Clones::new();
    clones.advance_remote(1);
    clones.repo.git(&["fetch", "-q"]);
    clones.repo.write(
        "vcs-lint.toml",
        "[history]\nbase = \"main\"\nnetwork = \"deny\"\n",
    );
    assert_snapshot!(clones.run(&["history", "--config", "vcs-lint.toml"]), @"
    exit: 1
    --- stdout
    --- stderr
    error: `HEAD` is behind base branch `origin/main`
      missing 1 commit of the base
    ");
}

#[test]
fn fails_without_base() {
    let clones = Clones::new();
    assert_snapshot!(clones.run(&["history"]), @"
    exit: 2
    --- stdout
    --- stderr
    error: no base branch is given
      give one with `--base`
    ");
}

#[test]
fn fails_on_base_that_is_not_branch_name() {
    let clones = Clones::new();
    assert_snapshot!(clones.run(&["history", "--base", "main~1"]), @"
    exit: 2
    --- stdout
    --- stderr
    error: invalid base branch `main~1` on `origin`
      `refs/heads/main~1` is not a valid ref name
    ");
    assert_snapshot!(clones.run(&["history", "--base", "main", "--remote", "a b", "--network", "deny"]), @"
    exit: 2
    --- stdout
    --- stderr
    error: invalid base branch `a b/main`
      `refs/remotes/a b/main` is not a valid ref name
    ");
}

#[test]
fn fails_on_revision_naming_no_commit() {
    let clones = Clones::new();
    assert_snapshot!(clones.run(&["history", "feat/y", "--base", "main"]), @"
    exit: 2
    --- stdout
    --- stderr
    error: `feat/y` cannot be resolved to a commit
    ");
    assert_snapshot!(clones.run(&["history", "--base", "main", "--", "--all"]), @"
    exit: 2
    --- stdout
    --- stderr
    error: `--all` cannot be resolved to a commit
    ");
}

#[test]
fn fails_on_base_missing_where_it_is_read_beside_refs_matching_its_name() {
    let clones = Clones::new();
    for branch in ["dev/x", "x/refs/heads/dev"] {
        clones.repo.git(&["branch", "-q", branch]);
        clones.repo.git(&["push", "-q", "origin", branch]);
    }
    clones.repo.git(&["fetch", "-q"]);
    assert_snapshot!(clones.run(&["history", "--base", "dev"]), @"
    exit: 2
    --- stdout
    --- stderr
    error: base branch `dev` on `origin` is not found
    ");
    assert_snapshot!(clones.run(&["history", "--base", "dev", "--network", "deny"]), @"
    exit: 2
    --- stdout
    --- stderr
    error: base branch `origin/dev` is not found
    ");
    assert_snapshot!(clones.run(&["history", "--base", "dev", "--base-from", "local"]), @"
    exit: 2
    --- stdout
    --- stderr
    error: base branch `dev` is not found
    ");
}

#[test]
fn fails_when_remote_cannot_be_read() {
    let clones = Clones::new();
    assert_snapshot!(clones.run(&["history", "--base", "main", "--remote", "upstream"]), @"
    exit: 2
    --- stdout
    --- stderr
    error: `git ls-remote --exit-code --end-of-options upstream refs/heads/main` failed with exit status: 128
      fatal: 'upstream' does not appear to be a git repository
      fatal: Could not read from remote repository.

      Please make sure you have the correct access rights
      and the repository exists.
    ");
}

#[test]
fn fails_outside_repository() {
    let workdir = Workdir::new();
    assert_snapshot!(workdir.run(&["history", "--base", "main"]), @"
    exit: 2
    --- stdout
    --- stderr
    error: `git cat-file --batch-check=%(objectname)` failed with exit status: 128
      fatal: not a git repository (or any of the parent directories): .git
    ");
}

#[test]
fn rejects_tip_missing_from_partial_clone_without_fetching_it() {
    let clones = Clones::new();
    clones
        .remote
        .git(&["config", "uploadpack.allowFilter", "true"]);
    let partial = Clones::clone(&clones.remote, &["--filter=blob:none"]);
    clones.advance_remote(1);
    assert_snapshot!(redact(&partial.run(&["history", "--base", "main"])), @"
    exit: 1
    --- stdout
    --- stderr
    error: `HEAD` is behind base branch `main` on `origin`
      the tip <id> is not in the local repository
        fetch it to see how many commits are missing
    ");
}

#[test]
fn fails_in_shallow_repository_on_tip_not_found_in_history() {
    let clones = Clones::new();
    let shallow = Clones::clone(&clones.remote, &["--depth", "1"]);
    shallow.git(&["switch", "-q", "-c", "feat/x"]);
    Clones::commit(&shallow, "feat");
    assert_snapshot!(shallow.run(&["history", "--base", "main"]), @"
    exit: 0
    --- stdout
    --- stderr
    ");
    clones.advance_remote(1);
    assert_snapshot!(shallow.run(&["history", "--base", "main"]), @"
    exit: 2
    --- stdout
    --- stderr
    error: cannot tell whether `HEAD` contains the tip of base branch `main` on `origin`
      the tip is not found in the history of a shallow repository, which may be missing commits
      fetch more history, e.g. with `git fetch --unshallow`
    ");
    shallow.git(&["fetch", "-q", "--depth", "1"]);
    assert_snapshot!(shallow.run(&["history", "--base", "main"]), @"
    exit: 2
    --- stdout
    --- stderr
    error: cannot tell whether `HEAD` contains the tip of base branch `main` on `origin`
      the tip is not found in the history of a shallow repository, which may be missing commits
      fetch more history, e.g. with `git fetch --unshallow`
    ");
}
