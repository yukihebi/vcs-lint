use std::ffi::OsStr;

use super::GitCommand;
use crate::report::git::GitError;

fn git(args: &[&str]) -> GitCommand {
    GitCommand::new(OsStr::new("git"), args)
}

#[test]
fn reads_standard_output() {
    let stdout = git(&["version"]).read(|stdout| Some(stdout.to_vec()));
    assert!(stdout.unwrap().starts_with(b"git version "));
}

#[test]
fn reads_empty_input_without_waiting() {
    let hash = git(&["hash-object", "--stdin"]).read(|stdout| Some(stdout.to_vec()));
    assert_eq!(hash.unwrap(), b"e69de29bb2d1d6434b8b29ae775ad8c2e48c5391\n");
}

#[test]
fn fails_when_git_is_not_found() {
    let result =
        GitCommand::new(OsStr::new("vcs-lint-no-such-git"), &["version"]).read(|_| Some(()));
    assert!(matches!(result, Err(GitError::NotFound)));
}

#[test]
fn fails_with_status_and_standard_error_when_command_fails() {
    let result = git(&["no-such-command"]).read(|_| Some(()));
    let Err(GitError::Failed {
        command,
        status,
        stderr,
    }) = result
    else {
        panic!("expected a failure, got {result:?}");
    };
    assert_eq!(command, "git no-such-command");
    assert_eq!(status.code(), Some(1));
    assert!(!stderr.is_empty());
    assert_eq!(stderr, stderr.trim_end());
}

#[test]
fn fails_with_output_when_parse_rejects_it() {
    let result = git(&["version"]).read(|_| None::<()>);
    let Err(GitError::UnexpectedOutput { command, stdout }) = result else {
        panic!("expected unexpected output, got {result:?}");
    };
    assert_eq!(command, "git version");
    assert!(stdout.starts_with("git version "));
}
