use insta::assert_snapshot;
use tempfile::tempdir;

use super::Run;

#[test]
fn rejects_unknown_option() {
    assert_snapshot!(Run::new(&["branch", "feat/x", "--scope", "core"]), @"
    exit: 2
    --- stdout
    --- stderr
    error: unexpected argument '--scope' found

      tip: to pass '--scope' as a value, use '-- --scope'

    Usage: vcs-lint branch <NAME>

    For more information, try '--help'.
    ");
}

#[test]
fn rejects_missing_subcommand() {
    assert_snapshot!(Run::new(&[]), @"
    exit: 2
    --- stdout
    --- stderr
    Lint branch names and commit message headers against types and scopes given on the command line

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
    assert_snapshot!(Run::in_dir(dir.path(), &["commit", "missing"]), @"
    exit: 1
    --- stdout
    --- stderr
    error: cannot read commit message file `missing`
      No such file or directory (os error 2)
    ");
}
