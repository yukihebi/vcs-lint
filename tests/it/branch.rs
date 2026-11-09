use insta::assert_snapshot;

use vcs_lint::RunStatus;

use super::Run;

#[test]
fn accepts_valid_branch() {
    let run = Run::new(&["branch", "feat/add-matcher"]);
    assert_status!(run, RunStatus::Success);
    assert_snapshot!(run, @"
    exit: 0
    --- stdout
    --- stderr
    ");
}

#[test]
fn rejects_protected_branch_without_checking_format() {
    let run = Run::new(&["branch", "main", "--protected", "main,develop"]);
    assert_status!(run, RunStatus::BranchLintError(_));
    assert_snapshot!(run, @"
    exit: 1
    --- stdout
    --- stderr
    error: direct commits to `main` are not allowed
      create a branch and open a pull request instead
    ");
}

#[test]
fn rejects_missing_type_prefix() {
    let run = Run::new(&["branch", "add-matcher", "--type", "feat,fix"]);
    assert_status!(run, RunStatus::BranchLintError(_));
    assert_snapshot!(run, @"
    exit: 1
    --- stdout
    --- stderr
    error: invalid branch name `add-matcher`
      missing type prefix
        expected: <type>/<name>
        allowed types: feat, fix
    ");
}

#[test]
fn rejects_invalid_name() {
    let run = Run::new(&["branch", "feat/Add_Matcher"]);
    assert_status!(run, RunStatus::BranchLintError(_));
    assert_snapshot!(run, @"
    exit: 1
    --- stdout
    --- stderr
    error: invalid branch name `feat/Add_Matcher`
      invalid name `Add_Matcher`
        expected: lowercase kebab-case starting with a letter, e.g. `add-matcher`
    ");
}

#[test]
fn reports_unknown_type_and_invalid_name_together() {
    let run = Run::new(&["branch", "feature/Add_Matcher", "--type", "feat,fix"]);
    assert_status!(run, RunStatus::BranchLintError(_));
    assert_snapshot!(run, @"
    exit: 1
    --- stdout
    --- stderr
    error: invalid branch name `feature/Add_Matcher`
      unknown type `feature`
        allowed types: feat, fix
      invalid name `Add_Matcher`
        expected: lowercase kebab-case starting with a letter, e.g. `add-matcher`
    ");
}

#[test]
fn accepts_detached_head_without_checking_protection_or_format() {
    let run = Run::new(&["branch", "HEAD", "--protected", "HEAD"]);
    assert_status!(run, RunStatus::Success);
    assert_snapshot!(run, @"
    exit: 0
    --- stdout
    --- stderr
    ");
}

#[test]
fn rejects_detached_head_when_denied() {
    let run = Run::new(&["branch", "HEAD", "--detached", "deny"]);
    assert_status!(run, RunStatus::BranchLintError(_));
    assert_snapshot!(run, @"
    exit: 1
    --- stdout
    --- stderr
    error: commits on a detached HEAD are not allowed
      switch to a branch or create one before committing
    ");
}

#[test]
fn prints_help() {
    let run = Run::new(&["branch", "--help"]);
    assert_status!(run, RunStatus::Info(_));
    assert_snapshot!(run, @"
    exit: 0
    --- stdout
    Lint a branch name of the form `<type>/<name>`

    Usage: vcs-lint branch [OPTIONS] <NAME>

    Arguments:
      <NAME>
              Branch name to lint

    Options:
          --type <TYPE>
              Allowed type, given as repeated flags or comma-separated values
              [default: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert]

          --protected <BRANCH>
              Branch that must not be committed to directly, given as repeated flags or comma-separated values

          --detached <RULE>
              Whether a detached HEAD, given as `HEAD`, passes
              [default: allow]

              Possible values:
              - allow: A detached HEAD passes
              - deny:  A detached HEAD fails

      -h, --help
              Print help (see a summary with '-h')
    --- stderr
    ");
}
