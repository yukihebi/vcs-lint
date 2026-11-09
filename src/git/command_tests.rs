use std::{ffi::OsStr, path::Path};

use super::GitCommand;
use crate::report::git::GitError;

fn git(args: &[&str]) -> GitCommand {
    GitCommand::new(OsStr::new("git"), args)
}

fn stdout(stdout: &[u8]) -> Option<Vec<u8>> {
    Some(stdout.to_vec())
}

#[test]
fn reads_standard_output() {
    let stdout = git(&["version"]).read_optional(stdout).unwrap().unwrap();
    assert!(stdout.starts_with(b"git version "));
}

#[test]
fn gives_none_when_query_finds_nothing() {
    let value = git(&["config", "--get", "vcs-lint.no-such-key"]).read_optional(stdout);
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
    .read_optional(stdout);
    let Err(GitError::Failed { status, stderr, .. }) = result else {
        panic!("expected a failure, got {result:?}");
    };
    assert_eq!(status.code(), Some(1));
    assert_eq!(stderr, "xcrun: error: invalid active developer path");
}

#[test]
fn fails_when_git_is_not_found() {
    let result =
        GitCommand::new(OsStr::new("vcs-lint-no-such-git"), &["version"]).read_optional(stdout);
    assert_eq!(result, Err(GitError::NotFound));
}

#[test]
fn fails_with_status_and_standard_error_when_command_fails() {
    let result = git(&["hash-object", "--no-such-option"]).read_optional(stdout);
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
    let result = git(&["version"]).read_optional(|_| None::<()>);
    let Err(GitError::UnexpectedOutput { command, stdout }) = result else {
        panic!("expected unexpected output, got {result:?}");
    };
    assert_eq!(command, "git version");
    assert!(stdout.starts_with("git version "));
}
