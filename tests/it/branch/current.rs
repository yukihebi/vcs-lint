//! `vcs-lint branch` without a name, reading the current branch from git.

use insta::assert_snapshot;

use crate::it::{BinaryOutput, workdir::Workdir};

#[test]
fn accepts_current_branch_with_no_commit_yet() {
    let repo = Workdir::repo("feat/add-matcher");
    assert_snapshot!(repo.run(&["branch"]), @"
    exit: 0
    --- stdout
    --- stderr
    ");
}

#[test]
fn reports_current_branch_by_name() {
    let repo = Workdir::repo("feat/Add_Matcher");
    assert_snapshot!(repo.run(&["branch"]), @"
    exit: 1
    --- stdout
    --- stderr
    error: invalid branch name `feat/Add_Matcher`
      invalid name `Add_Matcher`
        expected: lowercase kebab-case starting with a letter, e.g. `add-matcher`
    ");
}

#[test]
fn reads_branch_sharing_its_name_with_tag() {
    let repo = Workdir::repo("main");
    repo.commit();
    repo.git(&["tag", "main"]);
    assert_snapshot!(repo.run(&["branch", "--protected", "main"]), @"
    exit: 1
    --- stdout
    --- stderr
    error: branch `main` is protected
    ");
}

#[test]
fn applies_detached_head_rule_to_detached_head() {
    let repo = Workdir::repo("main");
    repo.commit();
    repo.git(&["switch", "-q", "--detach"]);
    assert_snapshot!(repo.run(&["branch", "--protected", "main"]), @"
    exit: 0
    --- stdout
    --- stderr
    ");
    assert_snapshot!(repo.run(&["branch", "--detached", "deny"]), @"
    exit: 1
    --- stdout
    --- stderr
    error: a detached HEAD is not allowed
    ");
}

#[test]
fn reads_current_branch_of_worktree() {
    let repo = Workdir::repo("main");
    repo.commit();
    repo.git(&["worktree", "add", "-q", "-b", "Feat/x", "wt"]);
    let output = BinaryOutput::of(
        repo.vcs_lint(&["branch"])
            .current_dir(repo.path().join("wt")),
    );
    assert_snapshot!(output, @"
    exit: 1
    --- stdout
    --- stderr
    error: invalid branch name `Feat/x`
      unknown type `Feat`
        allowed types: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert
    ");
}

#[test]
fn lints_given_name_without_git() {
    let dir = Workdir::new();
    let output = BinaryOutput::of(dir.vcs_lint(&["branch", "feat/x"]).env("PATH", ""));
    assert_snapshot!(output, @"
    exit: 0
    --- stdout
    --- stderr
    ");
}

#[test]
fn reports_rules_that_cannot_be_resolved_before_reading_git() {
    let dir = Workdir::new();
    assert_snapshot!(dir.run(&["branch", "--type-defaults", "never"]), @"
    exit: 2
    --- stdout
    --- stderr
    error: there are no allowed types
      give one with `--type`
    ");
}

#[test]
fn fails_outside_repository() {
    let dir = Workdir::new();
    assert_snapshot!(dir.run(&["branch"]), @"
    exit: 2
    --- stdout
    --- stderr
    error: `git symbolic-ref -q HEAD` failed with exit status: 128
      fatal: not a git repository (or any of the parent directories): .git
    ");
}

#[test]
fn fails_without_git() {
    let repo = Workdir::repo("main");
    let output = BinaryOutput::of(repo.vcs_lint(&["branch"]).env("PATH", ""));
    assert_snapshot!(output, @"
    exit: 2
    --- stdout
    --- stderr
    error: git is not found
      install git or add it to `PATH`
    ");
}
