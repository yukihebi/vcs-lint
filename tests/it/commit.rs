use insta::assert_snapshot;

use super::Run;

const SCOPES: [&str; 2] = ["--scope", "root,core"];

#[test]
fn accepts_valid_header_and_ignores_body() {
    assert_snapshot!(Run::commit("feat: add matcher\n\nNot: checked\n", &[]), @"
    exit: 0
    --- stdout
    --- stderr
    ");
}

#[test]
fn accepts_valid_scoped_header() {
    assert_snapshot!(Run::commit("fix(core): handle empty input\n", &SCOPES), @"
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
    assert_snapshot!(Run::commit(message, &[]), @"
    exit: 0
    --- stdout
    --- stderr
    ");
}

#[test]
fn accepts_autosquash_message() {
    assert_snapshot!(Run::commit("fixup! feature: whatever\n", &[]), @"
    exit: 0
    --- stdout
    --- stderr
    ");
}

#[test]
fn rejects_unknown_type() {
    assert_snapshot!(Run::commit("feature: add matcher\n", &["--type", "feat,fix"]), @"
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
    assert_snapshot!(Run::commit("feat(core): add matcher\n", &[]), @"
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
    assert_snapshot!(Run::commit("feat: add matcher\n", &SCOPES), @"
    exit: 1
    --- stdout
    --- stderr
    error: invalid commit header `feat: add matcher`
      missing scope
        allowed scopes: root, core
    ");
}

#[test]
fn rejects_malformed_header() {
    assert_snapshot!(Run::commit("add matcher\n", &SCOPES), @"
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
    assert_snapshot!(Run::commit("feature(parser):add matcher\n", &SCOPES), @"
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
fn rejects_empty_subject() {
    assert_snapshot!(Run::commit("feat: \n", &[]), @"
    exit: 1
    --- stdout
    --- stderr
    error: invalid commit header `feat: `
      empty subject
    ");
}

#[test]
fn rejects_message_without_content() {
    assert_snapshot!(Run::commit("\n# only a comment\n", &[]), @"
    exit: 1
    --- stdout
    --- stderr
    error: commit message is empty
    ");
}
