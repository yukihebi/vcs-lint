//! Violations reported as warnings with `--severity warning`.

use std::io::Write;

use insta::assert_snapshot;
use tempfile::NamedTempFile;

use vcs_lint::RunStatus;

use super::Run;

const WARNING: [&str; 2] = ["--severity", "warning"];

#[test]
fn reports_branch_violation_as_warning() {
    let run = Run::new(&[&["branch", "main", "--protected", "main"][..], &WARNING].concat());
    assert_status!(run, RunStatus::BranchLintWarning(_));
    assert_snapshot!(run, @"
    exit: 0
    --- stdout
    --- stderr
    warning: branch `main` is protected
    ");
}

#[test]
fn reports_commit_violation_as_warning() {
    let run = Run::commit("feat(cli): add matcher\n", &WARNING);
    assert_status!(run, RunStatus::CommitLintWarning(_));
    assert_snapshot!(run, @"
    exit: 0
    --- stdout
    --- stderr
    warning: invalid commit header `feat(cli): add matcher`
      unexpected scope `cli`
        scopes are not used; remove `(cli)`
    ");
}

#[test]
fn reports_violation_as_error_when_given_error() {
    let run = Run::new(&[
        "branch",
        "main",
        "--protected",
        "main",
        "--severity",
        "error",
    ]);
    assert_status!(run, RunStatus::BranchLintError(_));
    assert_eq!(run.output.code, Some(1));
}

#[test]
fn reports_nothing_without_violation() {
    let run = Run::new(&[&["branch", "feat/add-matcher"][..], &WARNING].concat());
    assert_status!(run, RunStatus::Success);
    assert_snapshot!(run, @"
    exit: 0
    --- stdout
    --- stderr
    ");
}

#[test]
fn keeps_status_of_errors() {
    let run = Run::new(
        &[
            &[
                "branch",
                "feat/x",
                "--protected",
                "main",
                "--protected-exclude",
                "develop",
            ][..],
            &WARNING,
        ]
        .concat(),
    );
    assert_status!(run, RunStatus::BranchResolveError(_));
    assert_eq!(run.output.code, Some(2));

    let run = Run::commit(
        "feat: add matcher\n",
        &[&["--scope-mode", "required"][..], &WARNING].concat(),
    );
    assert_status!(run, RunStatus::CommitResolveError(_));
    assert_eq!(run.output.code, Some(2));

    let run = Run::new(&[&["branch", "feat/x", "--scope", "core"][..], &WARNING].concat());
    assert_status!(run, RunStatus::SyntaxError(_));
    assert_eq!(run.output.code, Some(2));
}

#[test]
fn reads_severity_from_config_file() {
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(b"severity = \"warning\"\n").unwrap();
    let path = file.path().to_str().unwrap();
    let run = Run::new(&["branch", "main", "--protected", "main", "--config", path]);
    assert_status!(run, RunStatus::BranchLintWarning(_));
    assert_eq!(run.output.code, Some(0));
}
