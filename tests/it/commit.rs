use insta::assert_snapshot;

use vcs_lint::RunStatus;

use super::Run;

const SCOPES: [&str; 2] = ["--scope", "root,core"];
const OPTIONAL_SCOPES: [&str; 4] = ["--scope", "root,core", "--scope-mode", "optional"];

#[test]
fn accepts_valid_header_and_ignores_body() {
    let run = Run::commit("feat: add matcher\n\nNot: checked\n", &[]);
    assert_status!(run, RunStatus::Success);
    assert_snapshot!(run, @"
    exit: 0
    --- stdout
    --- stderr
    ");
}

#[test]
fn accepts_valid_scoped_header() {
    let run = Run::commit("fix(core): handle empty input\n", &SCOPES);
    assert_status!(run, RunStatus::Success);
    assert_snapshot!(run, @"
    exit: 0
    --- stdout
    --- stderr
    ");
}

#[test]
fn accepts_message_written_by_git_commit_verbose() {
    let message = "\
feat: add matcher

# Please enter the commit message for your changes. Lines starting
# with '#' will be ignored, and an empty message aborts the commit.
# ------------------------ >8 ------------------------
# Do not modify or remove the line above.
diff --git a/src/lib.rs b/src/lib.rs
";
    let run = Run::commit(message, &[]);
    assert_status!(run, RunStatus::Success);
    assert_snapshot!(run, @"
    exit: 0
    --- stdout
    --- stderr
    ");
}

#[test]
fn accepts_autosquash_message() {
    let run = Run::commit("fixup! feature: whatever\n", &[]);
    assert_status!(run, RunStatus::Success);
    assert_snapshot!(run, @"
    exit: 0
    --- stdout
    --- stderr
    ");
}

#[test]
fn rejects_unknown_type() {
    let run = Run::commit("feature: add matcher\n", &["--type", "feat,fix"]);
    assert_status!(run, RunStatus::CommitLintError(_));
    assert_snapshot!(run, @"
    exit: 1
    --- stdout
    --- stderr
    error: invalid commit header `feature: add matcher`
      unknown type `feature`
        allowed types: feat, fix
    ");
}

#[test]
fn rejects_scope_when_scopes_are_not_used() {
    let run = Run::commit("feat(core): add matcher\n", &[]);
    assert_status!(run, RunStatus::CommitLintError(_));
    assert_snapshot!(run, @"
    exit: 1
    --- stdout
    --- stderr
    error: invalid commit header `feat(core): add matcher`
      unexpected scope `core`
        scopes are not used; remove `(core)`
    ");
}

#[test]
fn rejects_missing_scope() {
    let run = Run::commit("feat: add matcher\n", &SCOPES);
    assert_status!(run, RunStatus::CommitLintError(_));
    assert_snapshot!(run, @"
    exit: 1
    --- stdout
    --- stderr
    error: invalid commit header `feat: add matcher`
      missing scope
        allowed scopes: root, core
    ");
}

#[test]
fn rejects_malformed_header_without_scopes() {
    let run = Run::commit("add matcher\n", &[]);
    assert_status!(run, RunStatus::CommitLintError(_));
    assert_snapshot!(run, @"
    exit: 1
    --- stdout
    --- stderr
    error: invalid commit header `add matcher`
      malformed header
        expected: <type>: <subject>
    ");
}

#[test]
fn accepts_header_without_scope_when_scope_is_optional() {
    let run = Run::commit("feat: add matcher\n", &OPTIONAL_SCOPES);
    assert_status!(run, RunStatus::Success);
    assert_snapshot!(run, @"
    exit: 0
    --- stdout
    --- stderr
    ");
}

#[test]
fn rejects_unknown_scope_when_scope_is_optional() {
    let run = Run::commit("feat(parser): add matcher\n", &OPTIONAL_SCOPES);
    assert_status!(run, RunStatus::CommitLintError(_));
    assert_snapshot!(run, @"
    exit: 1
    --- stdout
    --- stderr
    error: invalid commit header `feat(parser): add matcher`
      unknown scope `parser`
        allowed scopes: root, core
    ");
}

#[test]
fn rejects_scope_when_scope_is_forbidden_despite_scopes() {
    let run = Run::commit(
        "feat(core): add matcher\n",
        &["--scope", "root,core", "--scope-mode", "forbidden"],
    );
    assert_status!(run, RunStatus::CommitLintError(_));
    assert_snapshot!(run, @"
    exit: 1
    --- stdout
    --- stderr
    error: invalid commit header `feat(core): add matcher`
      unexpected scope `core`
        scopes are not used; remove `(core)`
    ");
}

#[test]
fn rejects_malformed_header_showing_optional_scope() {
    let run = Run::commit("add matcher\n", &OPTIONAL_SCOPES);
    assert_status!(run, RunStatus::CommitLintError(_));
    assert_snapshot!(run, @"
    exit: 1
    --- stdout
    --- stderr
    error: invalid commit header `add matcher`
      malformed header
        expected: <type>[(<scope>)]: <subject>
    ");
}

#[test]
fn rejects_malformed_header() {
    let run = Run::commit("add matcher\n", &SCOPES);
    assert_status!(run, RunStatus::CommitLintError(_));
    assert_snapshot!(run, @"
    exit: 1
    --- stdout
    --- stderr
    error: invalid commit header `add matcher`
      malformed header
        expected: <type>(<scope>): <subject>
    ");
}

#[test]
fn reports_every_violation_together() {
    let run = Run::commit("feature(parser):add matcher\n", &SCOPES);
    assert_status!(run, RunStatus::CommitLintError(_));
    assert_snapshot!(run, @"
    exit: 1
    --- stdout
    --- stderr
    error: invalid commit header `feature(parser):add matcher`
      unknown type `feature`
        allowed types: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert
      unknown scope `parser`
        allowed scopes: root, core
      expected exactly one space after `:`
    ");
}

#[test]
fn reports_empty_subject_with_other_violations() {
    let run = Run::commit("feature(parser):\n", &SCOPES);
    assert_status!(run, RunStatus::CommitLintError(_));
    assert_snapshot!(run, @"
    exit: 1
    --- stdout
    --- stderr
    error: invalid commit header `feature(parser):`
      unknown type `feature`
        allowed types: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert
      unknown scope `parser`
        allowed scopes: root, core
      empty subject
    ");
}

#[test]
fn rejects_empty_subject() {
    let run = Run::commit("feat: \n", &[]);
    assert_status!(run, RunStatus::CommitLintError(_));
    assert_snapshot!(run, @"
    exit: 1
    --- stdout
    --- stderr
    error: invalid commit header `feat: `
      empty subject
    ");
}

#[test]
fn rejects_message_without_content() {
    let run = Run::commit("\n# only a comment\n", &[]);
    assert_status!(run, RunStatus::CommitLintError(_));
    assert_snapshot!(run, @"
    exit: 1
    --- stdout
    --- stderr
    error: commit message is empty
    ");
}

#[test]
fn prints_help() {
    let run = Run::new(&["commit", "--help"]);
    assert_status!(run, RunStatus::Info(_));
    assert_snapshot!(run, @"
    exit: 0
    --- stdout
    Lint the header of a commit message file

    Usage: vcs-lint commit [OPTIONS] <FILE>

    Arguments:
      <FILE>
              Commit message file, such as the one given to the commit-msg hook

    Options:
          --type <TYPE>
              Allowed type, given as repeated flags or comma-separated values
              [default: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert]

          --scope <SCOPE>
              Allowed scope, given as repeated flags or comma-separated values

          --scope-mode <MODE>
              Whether commit headers have a scope
              [default: auto]

              Possible values:
              - auto:      Required when any scope is given, forbidden otherwise
              - required:  Headers must have an allowed scope
              - optional:  Headers may have an allowed scope, or none
              - forbidden: Headers must not have a scope; `--scope` is ignored

      -h, --help
              Print help (see a summary with '-h')
    --- stderr
    ");
}
