//! The options adding to a list the names of the directories that the index
//! holds entries in under a directory, such as `--scope-dir`.

use std::{fs, os::unix::fs::PermissionsExt};

use insta::assert_snapshot;

use super::{BinaryOutput, workdir::Workdir};

/// A gitlink needs no commit of the submodule to be in the index.
const SUBMODULE: &str = "160000,4b825dc642cb6eb9a060e54bf8d69288fbee4904,crates/sub";

/// A repository whose `crates` holds a committed package `a`, a staged package
/// `b`, a staged hidden directory, a staged file, a submodule and an untracked
/// package, with a commit message file `MSG` naming the scope `x`.
fn packages() -> Workdir {
    let repo = Workdir::repo("main");
    repo.write("crates/a/Cargo.toml", "");
    repo.write("crates/a/src/lib.rs", "");
    repo.git(&["add", "crates"]);
    repo.commit();
    repo.write("crates/b/Cargo.toml", "");
    repo.write("crates/.hidden/x", "");
    repo.write("crates/README.md", "");
    repo.git(&["add", "crates"]);
    repo.git(&["update-index", "--add", "--cacheinfo", SUBMODULE]);
    repo.write("crates/untracked/Cargo.toml", "");
    repo.write("MSG", "feat(x): add matcher\n");
    repo
}

#[test]
fn adds_directories_in_index() {
    let repo = packages();
    assert_snapshot!(repo.run(&["commit", "MSG", "--scope-dir", "crates"]), @"
    exit: 1
    --- stdout
    --- stderr
    error: invalid commit header `feat(x): add matcher`
      unknown scope `x`
        allowed scopes: .hidden, a, b, sub
    ");
}

#[test]
fn adds_directory_names_after_given_values() {
    let repo = packages();
    let args = [
        "commit",
        "MSG",
        "--scope-dir",
        "crates",
        "--scope",
        "root",
        "--scope-exclude",
        "b,.hidden",
    ];
    assert_snapshot!(repo.run(&args), @"
    exit: 1
    --- stdout
    --- stderr
    error: invalid commit header `feat(x): add matcher`
      unknown scope `x`
        allowed scopes: root, a, sub
    ");
}

#[test]
fn counts_directory_without_subdirectories_as_given() {
    let repo = packages();
    assert_snapshot!(repo.run(&["commit", "MSG", "--type-dir", "crates/a/src"]), @"
    exit: 2
    --- stdout
    --- stderr
    error: there are no allowed types
      give one with `--type`
    ");
}

#[test]
fn reads_directory_relative_to_current_directory() {
    let repo = packages();
    let output = BinaryOutput::of(
        repo.vcs_lint(&["commit", "../../MSG", "--scope-dir", "../.././crates//"])
            .current_dir(repo.path().join("crates/a")),
    );
    assert_snapshot!(output, @"
    exit: 1
    --- stdout
    --- stderr
    error: invalid commit header `feat(x): add matcher`
      unknown scope `x`
        allowed scopes: .hidden, a, b, sub
    ");
}

#[test]
fn adds_directories_to_protected_branches() {
    let repo = packages();
    assert_snapshot!(repo.run(&["branch", "b", "--protected-dir", "crates"]), @"
    exit: 1
    --- stdout
    --- stderr
    error: direct commits to `b` are not allowed
      create a branch and open a pull request instead
    ");
}

#[test]
fn reports_directory_names_nothing_can_match() {
    let repo = packages();
    repo.write("kinds/feat/x", "");
    repo.write("kinds/my kind/x", "");
    repo.git(&["add", "kinds"]);
    assert_snapshot!(repo.run(&["commit", "MSG", "--type-dir", "kinds"]), @r#"
    exit: 2
    --- stdout
    --- stderr
    error: no commit header can match these values
      allowed types: "my kind"
        a type cannot contain `(`, `)`, `:`, `!` or whitespace, nor start with `#`
    "#);
}

#[test]
fn rejects_missing_directory() {
    let repo = packages();
    assert_snapshot!(repo.run(&["commit", "MSG", "--scope-dir", "packages"]), @"
    exit: 2
    --- stdout
    --- stderr
    error: cannot read the directories under `packages` for `--scope-dir`
      no such directory
    ");
}

#[test]
fn rejects_file() {
    let repo = packages();
    assert_snapshot!(repo.run(&["commit", "MSG", "--scope-dir", "crates/README.md"]), @"
    exit: 2
    --- stdout
    --- stderr
    error: cannot read the directories under `crates/README.md` for `--scope-dir`
      not a directory
    ");
}

#[test]
fn rejects_directory_outside_work_tree() {
    let repo = packages();
    assert_snapshot!(repo.run(&["commit", "MSG", "--scope-dir", ".."]), @"
    exit: 2
    --- stdout
    --- stderr
    error: cannot read the directories under `..` for `--scope-dir`
      outside the work tree
    ");
}

#[test]
fn rejects_directory_outside_repository() {
    let workdir = Workdir::new();
    workdir.write("MSG", "feat: add matcher\n");
    assert_snapshot!(workdir.run(&["commit", "MSG", "--type-dir", "."]), @"
    exit: 2
    --- stdout
    --- stderr
    error: cannot read the directories under `.` for `--type-dir`
      `git rev-parse --show-toplevel` failed with exit status: 128
        fatal: not a git repository (or any of the parent directories): .git
    ");
}

/// A linked worktree runs hooks with `GIT_DIR` set to its git directory, which
/// git takes the current directory as the top of the work tree with.
#[test]
fn reads_index_of_commit_in_linked_worktree_hook() {
    let repo = packages();
    repo.commit();
    repo.git(&["worktree", "add", "-q", "wt"]);
    repo.write("wt/crates/c/Cargo.toml", "");
    repo.git_in("wt", &["add", "crates/c"]);
    let hook = repo.path().join(".git/hooks/commit-msg");
    fs::write(
        &hook,
        format!(
            "#!/bin/sh\nexec '{}' commit \"$1\" --scope-dir crates\n",
            BinaryOutput::PROGRAM
        ),
    )
    .unwrap();
    fs::set_permissions(&hook, fs::Permissions::from_mode(0o755)).unwrap();
    assert_snapshot!(repo.git_in("wt", &["commit", "-q", "-m", "feat(x): add matcher"]), @"
    exit: 1
    --- stdout
    --- stderr
    error: invalid commit header `feat(x): add matcher`
      unknown scope `x`
        allowed scopes: .hidden, a, b, c, sub
    ");
    assert_snapshot!(repo.git_in("wt", &["commit", "-q", "-m", "feat(c): add c"]), @"
    exit: 0
    --- stdout
    --- stderr
    ");
}
