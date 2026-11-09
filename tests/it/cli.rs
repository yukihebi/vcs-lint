use std::ffi::OsStr;

use insta::assert_snapshot;
use tempfile::tempdir;

use vcs_lint::RunStatus;

use super::Run;

#[test]
fn prints_help() {
    let run = Run::new(&["--help"]);
    assert_status!(run, RunStatus::Info(_));
    assert_snapshot!(run, @"
    exit: 0
    --- stdout
    A Simple, Fast Linter for Commit Messages and Branch Names in Version Control Systems

    Usage: vcs-lint <COMMAND>

    Commands:
      branch  Lint a branch name of the form `<type>/<name>`
      commit  Lint the header of a commit message file
      help    Print this message or the help of the given subcommand(s)

    Options:
      -h, --help     Print help
      -V, --version  Print version
    --- stderr
    ");
}

#[test]
fn prints_version() {
    let run = Run::new(&["--version"]);
    assert_status!(run, RunStatus::Info(_));
    assert_eq!(run.code, Some(0));
    assert_eq!(
        run.stdout,
        concat!("vcs-lint ", env!("CARGO_PKG_VERSION"), "\n")
    );
    assert_eq!(run.stderr, "");
}

#[test]
fn rejects_unknown_option() {
    let run = Run::new(&["branch", "feat/x", "--scope", "core"]);
    assert_status!(run, RunStatus::SyntaxError(_));
    assert_snapshot!(run, @"
    exit: 3
    --- stdout
    --- stderr
    error: unexpected argument '--scope' found

      tip: to pass '--scope' as a value, use '-- --scope'

    Usage: vcs-lint branch <NAME>

    For more information, try '--help'.
    ");
}

#[test]
fn rejects_list_flag_without_values() {
    let run = Run::new(&["branch", "feat/x", "--type", "feat", "--type", ""]);
    assert_status!(run, RunStatus::SyntaxError(_));
    assert_snapshot!(run, @"
    exit: 3
    --- stdout
    --- stderr
    error: invalid value '' for '--type <TYPE>': every value is empty

    For more information, try '--help'.
    ");
}

#[cfg(unix)]
#[test]
fn rejects_non_utf8_argument() {
    use std::os::unix::ffi::OsStrExt;

    let branch = OsStr::from_bytes(b"feat/\xff");
    let run = Run::new(&[OsStr::new("branch"), branch]);
    assert_status!(run, RunStatus::SyntaxError(_));
    assert_snapshot!(run, @"
    exit: 3
    --- stdout
    --- stderr
    error: invalid UTF-8 was detected in one or more arguments

    Usage: vcs-lint branch [OPTIONS] <NAME>

    For more information, try '--help'.
    ");
}

#[test]
fn rejects_missing_subcommand() {
    let run = Run::new(&[] as &[&str]);
    assert_status!(run, RunStatus::SyntaxError(_));
    assert_snapshot!(run, @"
    exit: 3
    --- stdout
    --- stderr
    A Simple, Fast Linter for Commit Messages and Branch Names in Version Control Systems

    Usage: vcs-lint <COMMAND>

    Commands:
      branch  Lint a branch name of the form `<type>/<name>`
      commit  Lint the header of a commit message file
      help    Print this message or the help of the given subcommand(s)

    Options:
      -h, --help     Print help
      -V, --version  Print version
    ");
}

#[test]
fn rejects_unreadable_commit_message_file() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("missing");
    let run = Run::new(&[OsStr::new("commit"), file.as_os_str()]);
    assert_status!(run, RunStatus::CommitResolveError(_));
    assert_eq!(run.code, Some(2));
    assert_eq!(run.stdout, "");
    assert_eq!(
        run.stderr,
        format!(
            "error: cannot read commit message file `{}`\n  No such file or directory (os error 2)\n",
            file.display()
        )
    );
}
