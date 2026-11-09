use std::ffi::OsStr;

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
fn reads_empty_input_without_waiting() {
    let hash = git(&["hash-object", "--stdin"]).read_optional(stdout);
    assert_eq!(
        hash,
        Ok(Some(b"e69de29bb2d1d6434b8b29ae775ad8c2e48c5391\n".to_vec()))
    );
}

#[test]
fn gives_none_when_query_finds_nothing() {
    let value = git(&["config", "--get", "vcs-lint.no-such-key"]).read_optional(stdout);
    assert_eq!(value, Ok(None));
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
fn fails_with_output_when_parse_rejects_it() {
    let result = git(&["version"]).read_optional(|_| None::<()>);
    let Err(GitError::UnexpectedOutput { command, stdout }) = result else {
        panic!("expected unexpected output, got {result:?}");
    };
    assert_eq!(command, "git version");
    assert!(stdout.starts_with("git version "));
}
