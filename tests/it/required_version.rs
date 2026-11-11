use insta::assert_snapshot;

use vcs_lint::RunStatus;

use super::Run;

const VAR: &str = "VCS_LINT_REQUIRED_VERSION";
const NEWER: &str = "18446744073709551615.0.0";

/// What the binary prints when `required` is newer than the running version.
fn unmet_stderr(required: &str) -> String {
    format!(
        "error: vcs-lint {required} or later is required, but this is {}\n  required by `{VAR}`\n",
        env!("CARGO_PKG_VERSION")
    )
}

#[test]
fn runs_when_required_version_is_older() {
    let run = Run::with_vars(&["branch", "feat/add-matcher"], &[(VAR, "0.0.0-alpha.0")]);
    assert_status!(run, RunStatus::Success);
    assert_eq!(run.output.code, Some(0));
}

#[test]
fn runs_when_required_version_is_running_one() {
    let run = Run::with_vars(
        &["branch", "feat/add-matcher"],
        &[(VAR, env!("CARGO_PKG_VERSION"))],
    );
    assert_status!(run, RunStatus::Success);
    assert_eq!(run.output.code, Some(0));
}

#[test]
fn runs_when_required_version_is_blank() {
    let run = Run::with_vars(&["branch", "feat/add-matcher"], &[(VAR, " ")]);
    assert_status!(run, RunStatus::Success);
    assert_eq!(run.output.code, Some(0));
}

#[test]
fn fails_when_required_version_is_newer() {
    let run = Run::with_vars(&["branch", "feat/add-matcher"], &[(VAR, NEWER)]);
    assert_status!(run, RunStatus::RequiredVersionError(_));
    assert_eq!(run.output.code, Some(3));
    assert_eq!(run.output.stdout, "");
    assert_eq!(run.output.stderr, unmet_stderr(NEWER));
}

#[test]
fn fails_on_invalid_required_version() {
    let run = Run::with_vars(&["branch", "feat/add-matcher"], &[(VAR, " 0.2.0+build ")]);
    assert_status!(run, RunStatus::RequiredVersionError(_));
    assert_snapshot!(run, @r#"
    exit: 3
    --- stdout
    --- stderr
    error: invalid version "0.2.0+build" in `VCS_LINT_REQUIRED_VERSION`
      expected `MAJOR.MINOR.PATCH`, optionally followed by `-alpha.N`, `-beta.N` or `-rc.N`
    "#);
}

#[test]
fn fails_before_reading_command_line() {
    for args in [
        &["--no-such-option"][..],
        &["--help"],
        &["--version"],
        &["help"],
    ] {
        let run = Run::with_vars(args, &[(VAR, NEWER)]);
        assert_status!(run, RunStatus::RequiredVersionError(_));
        assert_eq!(run.output.code, Some(3), "{args:?}");
        assert_eq!(run.output.stderr, unmet_stderr(NEWER), "{args:?}");
    }
}

#[test]
fn fails_on_invalid_required_version_before_reading_command_line() {
    let run = Run::with_vars(&["--version"], &[(VAR, "latest")]);
    assert_status!(run, RunStatus::RequiredVersionError(_));
    assert_eq!(run.output.code, Some(3));
}
