use std::ffi::OsString;

use clap::error::ErrorKind;

use super::{ClapCommand, Parsed, branch::BranchDetachedClapRule};
use crate::resolve::{branch::BranchRawConfig, commit::CommitRawConfig};

fn parse(args: &[&str]) -> Result<Parsed, clap::Error> {
    Parsed::parse(args.iter().map(OsString::from).collect())
}

fn branch(args: &[&str]) -> BranchRawConfig {
    let Ok(Parsed::Command(ClapCommand::Branch(args))) = parse(&[&["branch"], args].concat())
    else {
        panic!("not a branch command");
    };
    args.into_raw_config()
}

fn commit(args: &[&str]) -> CommitRawConfig {
    let Ok(Parsed::Command(ClapCommand::Commit(args))) = parse(&[&["commit"], args].concat())
    else {
        panic!("not a commit command");
    };
    args.into_raw_config()
}

fn values(values: &[&str]) -> Option<Vec<String>> {
    Some(values.iter().copied().map(String::from).collect())
}

#[test]
fn branch_keeps_values_as_written() {
    assert_eq!(
        branch(&[
            "feat/x",
            "--type",
            " feat ",
            "--protected",
            "main",
            "--detached",
            "deny"
        ]),
        BranchRawConfig {
            branch: "feat/x".to_owned(),
            types: values(&[" feat "]),
            protected: values(&["main"]),
            detached: Some(BranchDetachedClapRule::Deny),
        }
    );
}

#[test]
fn branch_options_are_none_when_omitted() {
    assert_eq!(
        branch(&["feat/x"]),
        BranchRawConfig {
            branch: "feat/x".to_owned(),
            types: None,
            protected: None,
            detached: None,
        }
    );
}

#[test]
fn detached_accepts_allow() {
    assert_eq!(
        branch(&["HEAD", "--detached", "allow"]).detached,
        Some(BranchDetachedClapRule::Allow)
    );
}

#[test]
fn lists_accept_repeated_flags() {
    assert_eq!(
        branch(&["feat/x", "--type", "feat", "--type", "fix"]).types,
        values(&["feat", "fix"])
    );
}

#[test]
fn lists_accept_comma_separated_values() {
    assert_eq!(
        branch(&["feat/x", "--type", "feat,fix"]).types,
        values(&["feat", "fix"])
    );
}

#[test]
fn lists_accept_mixed_forms() {
    assert_eq!(
        branch(&[
            "feat/x",
            "--protected",
            "main,develop",
            "--protected=release"
        ])
        .protected,
        values(&["main", "develop", "release"])
    );
}

#[test]
fn lists_keep_empty_values() {
    assert_eq!(
        branch(&["feat/x", "--type", ",feat,", "--type", ""]).types,
        values(&["", "feat", "", ""])
    );
}

#[test]
fn options_may_precede_positional_argument() {
    assert_eq!(
        branch(&["--type", "feat", "feat/x"]),
        branch(&["feat/x", "--type", "feat"])
    );
}

#[test]
fn commit_keeps_file_without_reading_it() {
    assert_eq!(
        commit(&["missing", "--type", "feat", "--scope", "core,cli"]),
        CommitRawConfig {
            file: "missing".into(),
            types: values(&["feat"]),
            scopes: values(&["core", "cli"]),
        }
    );
}

#[test]
fn commit_lists_are_none_when_omitted() {
    assert_eq!(
        commit(&["missing"]),
        CommitRawConfig {
            file: "missing".into(),
            types: None,
            scopes: None,
        }
    );
}

#[test]
fn help_and_version_are_info() {
    for args in [
        &["--help"][..],
        &["--version"],
        &["help"],
        &["branch", "--help"],
        &["commit", "-h"],
    ] {
        assert!(matches!(parse(args), Ok(Parsed::Info(_))), "{args:?}");
    }
}

#[test]
fn rejects_command_line_outside_syntax() {
    for (args, kind) in [
        (&[][..], ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand),
        (&["branch"], ErrorKind::MissingRequiredArgument),
        (
            &["branch", "feat/x", "--scope", "core"],
            ErrorKind::UnknownArgument,
        ),
        (&["tag"], ErrorKind::InvalidSubcommand),
        (
            &["branch", "HEAD", "--detached", "Deny"],
            ErrorKind::InvalidValue,
        ),
        (
            &["branch", "HEAD", "--detached", "allow,deny"],
            ErrorKind::InvalidValue,
        ),
    ] {
        let Err(err) = parse(args) else {
            panic!("{args:?} parsed");
        };
        assert_eq!(err.kind(), kind, "{args:?}");
    }
}
