//! Hints printed after a violation of the rules they are given to.

use insta::assert_snapshot;

use vcs_lint::RunStatus;

use super::Run;

#[test]
fn prints_hint_of_protected_branch_line_by_line() {
    let run = Run::new(&[
        "branch",
        "main",
        "--protected",
        "main",
        "--protected-hint",
        "update main through a pull request\n\nsee CONTRIBUTING.md\n",
    ]);
    assert_status!(run, RunStatus::BranchLintError(_));
    assert_snapshot!(run, @"
    exit: 1
    --- stdout
    --- stderr
    error: branch `main` is protected
    hint: update main through a pull request
    hint:
    hint: see CONTRIBUTING.md
    ");
}

#[test]
fn prints_hint_of_detached_head() {
    let run = Run::new(&[
        "branch",
        "HEAD",
        "--detached",
        "deny",
        "--detached-hint",
        "switch to a branch first",
    ]);
    assert_status!(run, RunStatus::BranchLintError(_));
    assert_snapshot!(run, @"
    exit: 1
    --- stdout
    --- stderr
    error: a detached HEAD is not allowed
    hint: switch to a branch first
    ");
}

#[test]
fn prints_hints_of_broken_rules_alone_in_report_order() {
    let run = Run::new(&[
        "branch",
        "feature/Add_Matcher",
        "--type",
        "feat",
        "--format-hint",
        "names are kebab-case",
        "--type-hint",
        "types are listed in CONTRIBUTING.md",
        "--protected-hint",
        "never printed",
    ]);
    assert_status!(run, RunStatus::BranchLintError(_));
    assert_snapshot!(run, @"
    exit: 1
    --- stdout
    --- stderr
    error: invalid branch name `feature/Add_Matcher`
      unknown type `feature`
        allowed types: feat
      invalid name `Add_Matcher`
        expected: lowercase kebab-case starting with a letter, e.g. `add-matcher`
    hint: types are listed in CONTRIBUTING.md
    hint: names are kebab-case
    ");
}

#[test]
fn prints_type_hint_of_branch_without_type_prefix() {
    let run = Run::new(&[
        "branch",
        "add-matcher",
        "--type-hint",
        "e.g. feat/add-matcher",
    ]);
    assert_status!(run, RunStatus::BranchLintError(_));
    assert_snapshot!(run, @"
    exit: 1
    --- stdout
    --- stderr
    error: invalid branch name `add-matcher`
      missing type prefix
        expected: <type>/<name>
        allowed types: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert
    hint: e.g. feat/add-matcher
    ");
}

#[test]
fn prints_scope_hint_of_commit() {
    let run = Run::commit(
        "feat(cli): add matcher\n",
        &[
            "--scope-hint",
            "scopes are not used",
            "--format-hint",
            "never printed",
        ],
    );
    assert_status!(run, RunStatus::CommitLintError(_));
    assert_snapshot!(run, @"
    exit: 1
    --- stdout
    --- stderr
    error: invalid commit header `feat(cli): add matcher`
      unexpected scope `cli`
        scopes are not used; remove `(cli)`
    hint: scopes are not used
    ");
}

#[test]
fn prints_format_hint_of_empty_commit_message() {
    let run = Run::commit("# comment\n", &["--format-hint", "write a header"]);
    assert_status!(run, RunStatus::CommitLintError(_));
    assert_snapshot!(run, @"
    exit: 1
    --- stdout
    --- stderr
    error: commit message is empty
    hint: write a header
    ");
}

#[test]
fn prints_text_given_to_several_broken_rules_once() {
    let run = Run::commit(
        "feature:add matcher\n",
        &[
            "--type-hint",
            "see CONTRIBUTING.md",
            "--format-hint",
            "see CONTRIBUTING.md",
        ],
    );
    assert_status!(run, RunStatus::CommitLintError(_));
    assert_snapshot!(run, @"
    exit: 1
    --- stdout
    --- stderr
    error: invalid commit header `feature:add matcher`
      unknown type `feature`
        allowed types: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert
      expected exactly one space after `:`
    hint: see CONTRIBUTING.md
    ");
}

#[test]
fn prints_hint_after_warning() {
    let run = Run::new(&[
        "branch",
        "main",
        "--protected",
        "main",
        "--protected-hint",
        "update main through a pull request",
        "--severity",
        "warning",
    ]);
    assert_status!(run, RunStatus::BranchLintWarning(_));
    assert_snapshot!(run, @"
    exit: 0
    --- stdout
    --- stderr
    warning: branch `main` is protected
    hint: update main through a pull request
    ");
}

#[test]
fn prints_unescaped_braces() {
    let run = Run::new(&[
        "branch",
        "main",
        "--protected",
        "main",
        "--protected-hint",
        "run `git branch -m {{new}}`",
    ]);
    assert_status!(run, RunStatus::BranchLintError(_));
    assert_snapshot!(run, @"
    exit: 1
    --- stdout
    --- stderr
    error: branch `main` is protected
    hint: run `git branch -m {new}`
    ");
}

#[test]
fn prints_nothing_without_violation() {
    let run = Run::new(&[
        "branch",
        "feat/add-matcher",
        "--protected",
        "main",
        "--protected-hint",
        "update main through a pull request",
        "--type-hint",
        "see CONTRIBUTING.md",
    ]);
    assert_status!(run, RunStatus::Success);
    assert_snapshot!(run, @"
    exit: 0
    --- stdout
    --- stderr
    ");
}

#[test]
fn prints_no_hint_when_nothing_is_linted() {
    let run = Run::new(&[
        "commit",
        "missing",
        "--format-hint",
        "write a header",
        "--type-hint",
        "see CONTRIBUTING.md",
    ]);
    assert_status!(run, RunStatus::CommitResolveError(_));
    assert_snapshot!(run, @"
    exit: 2
    --- stdout
    --- stderr
    error: cannot read commit message file `missing`
      No such file or directory (os error 2)
    ");
}

#[test]
fn rejects_text_that_is_not_hint() {
    for text in ["", "run `git branch -m {new}`", "a { b", "a } b"] {
        let run = Run::new(&["branch", "main", "--protected-hint", text]);
        assert_status!(run, RunStatus::SyntaxError(_));
        assert_eq!(run.output.code, Some(2), "{text:?}");
    }
    let run = Run::new(&["branch", "main", "--protected-hint", "git branch -m {new}"]);
    assert_snapshot!(run, @"
    exit: 2
    --- stdout
    --- stderr
    error: invalid value 'git branch -m {new}' for '--protected-hint <TEXT>': placeholder `{new}` is not supported; write `{{` and `}}` for braces

    For more information, try '--help'.
    ");
}
