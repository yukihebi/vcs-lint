use std::{ffi::OsStr, path::Path};

use super::{GitAbsentStatus, GitCommand};
use crate::report::git::GitError;

fn git(args: &[&str]) -> GitCommand {
    GitCommand::new(OsStr::new("git"), args)
}

fn stdout(stdout: &[u8]) -> Option<Vec<u8>> {
    Some(stdout.to_vec())
}

#[test]
fn reads_standard_output() {
    let stdout = git(&["version"])
        .read_optional(GitAbsentStatus::QUIET, stdout)
        .unwrap()
        .unwrap();
    assert!(stdout.starts_with(b"git version "));
}

#[test]
fn gives_none_when_query_finds_nothing() {
    let value = git(&["config", "--get", "vcs-lint.no-such-key"])
        .read_optional(GitAbsentStatus::QUIET, stdout);
    assert_eq!(value, Ok(None));
}

#[test]
fn fails_when_absent_status_comes_with_output() {
    let result = GitCommand::new(
        OsStr::new("sh"),
        &[
            "-c",
            "echo 'xcrun: error: invalid active developer path' >&2; exit 1",
        ],
    )
    .read_optional(GitAbsentStatus::QUIET, stdout);
    let Err(GitError::Failed { status, stderr, .. }) = result else {
        panic!("expected a failure, got {result:?}");
    };
    assert_eq!(status.code(), Some(1));
    assert_eq!(stderr, "xcrun: error: invalid active developer path");
}

#[test]
fn fails_when_git_is_not_found() {
    let result = GitCommand::new(OsStr::new("vcs-lint-no-such-git"), &["version"])
        .read_optional(GitAbsentStatus::QUIET, stdout);
    assert_eq!(result, Err(GitError::NotFound));
}

#[test]
fn fails_with_status_and_standard_error_when_command_fails() {
    let result =
        git(&["hash-object", "--no-such-option"]).read_optional(GitAbsentStatus::QUIET, stdout);
    let Err(GitError::Failed {
        command,
        status,
        stderr,
    }) = result
    else {
        panic!("expected a failure, got {result:?}");
    };
    assert_eq!(command, "git hash-object --no-such-option");
    assert_eq!(status.code(), Some(129));
    assert!(!stderr.is_empty());
    assert_eq!(stderr, stderr.trim_end());
}

#[test]
fn reads_standard_output_of_command_that_must_succeed() {
    let stdout = git(&["version"]).read(stdout).unwrap();
    assert!(stdout.starts_with(b"git version "));
}

#[test]
fn fails_when_command_that_must_succeed_finds_nothing() {
    let result = git(&["config", "--get", "vcs-lint.no-such-key"]).read(stdout);
    let Err(GitError::Failed {
        command, status, ..
    }) = result
    else {
        panic!("expected a failure, got {result:?}");
    };
    assert_eq!(command, "git config --get vcs-lint.no-such-key");
    assert_eq!(status.code(), Some(1));
}

#[test]
fn shows_path_arguments_in_command() {
    let result = GitCommand::new(
        OsStr::new("git"),
        &[OsStr::new("ls-files"), Path::new("no such/dir").as_os_str()],
    )
    .read(|_| None::<()>);
    let Err(GitError::UnexpectedOutput { command, .. }) = result else {
        panic!("expected unexpected output, got {result:?}");
    };
    assert_eq!(command, "git ls-files no such/dir");
}

#[test]
fn fails_with_output_when_parse_rejects_it() {
    let result = git(&["version"]).read_optional(GitAbsentStatus::QUIET, |_| None::<()>);
    let Err(GitError::UnexpectedOutput { command, stdout }) = result else {
        panic!("expected unexpected output, got {result:?}");
    };
    assert_eq!(command, "git version");
    assert!(stdout.starts_with("git version "));
}

fn sh(script: &str) -> GitCommand {
    GitCommand::new(OsStr::new("sh"), &["-c", script])
}

#[test]
fn gives_none_on_status_given_as_absent() {
    let value = sh("exit 2").read_optional(GitAbsentStatus::with_stderr(2), stdout);
    assert_eq!(value, Ok(None));
}

#[test]
fn gives_none_on_absent_status_with_standard_error_where_allowed() {
    let value = sh("echo 'Warning: Permanently added host' >&2; exit 2")
        .read_optional(GitAbsentStatus::with_stderr(2), stdout);
    assert_eq!(value, Ok(None));
}

#[test]
fn fails_on_absent_status_with_standard_output_even_where_standard_error_is_allowed() {
    let result = sh("echo found; exit 2").read_optional(GitAbsentStatus::with_stderr(2), stdout);
    let Err(GitError::Failed { status, .. }) = result else {
        panic!("expected a failure, got {result:?}");
    };
    assert_eq!(status.code(), Some(2));
}

#[test]
fn fails_on_status_other_than_absent() {
    let result = sh("exit 1").read_optional(GitAbsentStatus::with_stderr(2), stdout);
    let Err(GitError::Failed { status, .. }) = result else {
        panic!("expected a failure, got {result:?}");
    };
    assert_eq!(status.code(), Some(1));
}

#[test]
fn gives_input_on_standard_input() {
    let stdout = sh("cat").input("a\nb\n").read(stdout).unwrap();
    assert_eq!(stdout, b"a\nb\n");
}

#[test]
fn reads_output_larger_than_pipe_while_writing_input() {
    let input = "x".repeat(1 << 20);
    let stdout = sh("cat").input(input.as_str()).read(stdout).unwrap();
    assert_eq!(stdout.len(), input.len());
}

#[test]
fn judges_command_exiting_before_reading_input_by_its_status() {
    let result = sh("exit 3").input("x".repeat(1 << 20)).read(stdout);
    let Err(GitError::Failed { status, .. }) = result else {
        panic!("expected a failure, got {result:?}");
    };
    assert_eq!(status.code(), Some(3));
}

#[test]
fn closes_standard_input_without_input() {
    let stdout = sh("cat").read(stdout).unwrap();
    assert!(stdout.is_empty());
}
