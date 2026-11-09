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
fn rejects_type_excluded_from_defaults() {
    let run = Run::new(&["branch", "chore/bump", "--type-exclude", "chore,revert"]);
    assert_status!(run, RunStatus::BranchLintError(_));
    assert_snapshot!(run, @"
    exit: 1
    --- stdout
    --- stderr
    error: invalid branch name `chore/bump`
      unknown type `chore`
        allowed types: feat, fix, docs, style, refactor, perf, test, build, ci
    ");
}

#[test]
fn accepts_type_added_to_defaults() {
    let run = Run::new(&[
        "branch",
        "wip/x",
        "--type",
        "wip",
        "--type-defaults",
        "always",
    ]);
    assert_status!(run, RunStatus::Success);
    assert_snapshot!(run, @"
    exit: 0
    --- stdout
    --- stderr
    ");
}

#[test]
fn fails_on_excluding_branch_that_is_not_protected() {
    let run = Run::new(&[
        "branch",
        "feat/x",
        "--protected",
        "main",
        "--protected-exclude",
        "develop",
    ]);
    assert_status!(run, RunStatus::BranchResolveError(_));
    assert_snapshot!(run, @"
    exit: 2
    --- stdout
    --- stderr
    error: cannot exclude values outside the protected branches: develop
      protected branches: main
    ");
}

#[test]
fn fails_on_values_no_branch_name_can_match() {
    let run = Run::new(&[
        "branch",
        "feat/x",
        "--type",
        "feat,fix/hot",
        "--protected",
        "main,ma\nin",
    ]);
    assert_status!(run, RunStatus::BranchResolveError(_));
    assert_snapshot!(run, @r#"
    exit: 2
    --- stdout
    --- stderr
    error: no branch name can match these values
      allowed types: "fix/hot"
        a type cannot contain `/` or a line break
      protected branches: "ma\nin"
        a branch name cannot contain a line break
    "#);
}

#[test]
fn fails_without_types() {
    let run = Run::new(&["branch", "feat/x", "--type-defaults", "never"]);
    assert_status!(run, RunStatus::BranchResolveError(_));
    assert_snapshot!(run, @"
    exit: 2
    --- stdout
    --- stderr
    error: there are no allowed types
      give one with `--type`
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
              [defaults: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert]

          --type-exclude <TYPE>
              Type excluded from the allowed types, given as repeated flags or comma-separated values

          --type-defaults <MODE>
              When the default types are allowed
              [default: auto]

              Possible values:
              - auto:   Allowed only when no type is given
              - always: Allowed besides the given types
              - never:  Never allowed

          --protected <BRANCH>
              Branch that must not be committed to directly, given as repeated flags or comma-separated values

          --protected-exclude <BRANCH>
              Branch excluded from the protected branches, given as repeated flags or comma-separated values

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
