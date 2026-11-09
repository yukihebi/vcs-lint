use std::{ffi::OsString, path::PathBuf};

use clap::error::ErrorKind;

use super::{
    ClapCommand, Parsed, TypeDefaultsClapMode, branch::BranchDetachedClapRule,
    commit::CommitScopeClapMode,
};
use crate::resolve::{branch::BranchRawConfig, commit::CommitRawConfig, list::ListRawValues};

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

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().copied().map(String::from).collect()
}

fn values(values: &[&str]) -> Option<Vec<String>> {
    Some(strings(values))
}

fn raw(given: Option<&[&str]>, excluded: &[&str]) -> ListRawValues {
    ListRawValues {
        given: given.map(strings),
        dirs: None,
        excluded: strings(excluded),
    }
}

fn with_dirs(raw: ListRawValues, dirs: &[&str]) -> ListRawValues {
    ListRawValues {
        dirs: Some(dirs.iter().map(PathBuf::from).collect()),
        ..raw
    }
}

#[test]
fn branch_takes_arguments_and_options() {
    assert_eq!(
        branch(&[
            "feat/x",
            "--type",
            "feat",
            "--type-dir",
            "kinds",
            "--type-exclude",
            "fix",
            "--type-defaults",
            "always",
            "--protected",
            "main",
            "--protected-dir",
            "releases",
            "--protected-exclude",
            "develop",
            "--detached",
            "deny"
        ]),
        BranchRawConfig {
            branch: Some("feat/x".to_owned()),
            types: with_dirs(raw(Some(&["feat"]), &["fix"]), &["kinds"]),
            type_defaults: Some(TypeDefaultsClapMode::Always),
            protected: with_dirs(raw(Some(&["main"]), &["develop"]), &["releases"]),
            detached: Some(BranchDetachedClapRule::Deny),
        }
    );
}

#[test]
fn branch_options_are_none_when_omitted() {
    assert_eq!(
        branch(&["feat/x"]),
        BranchRawConfig {
            branch: Some("feat/x".to_owned()),
            types: ListRawValues::default(),
            type_defaults: None,
            protected: ListRawValues::default(),
            detached: None,
        }
    );
}

#[test]
fn branch_name_is_none_when_omitted() {
    assert_eq!(branch(&["--protected", "main"]).branch, None);
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
        branch(&["feat/x", "--type", "feat", "--type", "fix"])
            .types
            .given,
        values(&["feat", "fix"])
    );
}

#[test]
fn exclusions_accept_list_forms() {
    assert_eq!(
        branch(&[
            "feat/x",
            "--protected-exclude",
            " main,,develop",
            "--protected-exclude=release"
        ])
        .protected,
        raw(None, &["main", "develop", "release"])
    );
}

#[test]
fn dirs_accept_list_forms_without_giving_values() {
    assert_eq!(
        commit(&[
            "missing",
            "--scope-dir",
            " ./crates/ ,,apps",
            "--scope-dir=../libs"
        ])
        .scopes,
        with_dirs(raw(None, &[]), &["./crates/", "apps", "../libs"])
    );
}

#[test]
fn lists_accept_comma_separated_values() {
    assert_eq!(
        branch(&["feat/x", "--type", "feat,fix"]).types.given,
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
        .protected
        .given,
        values(&["main", "develop", "release"])
    );
}

#[test]
fn lists_trim_values() {
    assert_eq!(
        branch(&["feat/x", "--type", " feat ,\tfix", "--type", " docs"])
            .types
            .given,
        values(&["feat", "fix", "docs"])
    );
}

#[test]
fn lists_drop_empty_values_beside_others() {
    assert_eq!(
        branch(&["feat/x", "--type", ",feat,, ,fix,", "--type", "docs,"])
            .types
            .given,
        values(&["feat", "fix", "docs"])
    );
}

#[test]
fn lists_reject_flag_without_values() {
    for (command, option) in [
        (&["branch", "feat/x"][..], "--type"),
        (&["branch", "feat/x"], "--type-dir"),
        (&["branch", "feat/x"], "--type-exclude"),
        (&["branch", "feat/x"], "--protected"),
        (&["branch", "feat/x"], "--protected-dir"),
        (&["branch", "feat/x"], "--protected-exclude"),
        (&["commit", "missing"], "--type"),
        (&["commit", "missing"], "--type-dir"),
        (&["commit", "missing"], "--type-exclude"),
        (&["commit", "missing"], "--scope"),
        (&["commit", "missing"], "--scope-dir"),
        (&["commit", "missing"], "--scope-exclude"),
    ] {
        for written in [&["", "x"][..], &["x", ","], &[" , \t"]] {
            let flags = written.iter().flat_map(|value| [option, value]);
            let args: Vec<&str> = command.iter().copied().chain(flags).collect();
            let Err(err) = parse(&args) else {
                panic!("{args:?} parsed");
            };
            assert_eq!(err.kind(), ErrorKind::ValueValidation, "{args:?}");
        }
    }
}

#[test]
fn lists_reject_flag_given_empty_value_with_equals_sign() {
    let Err(err) = parse(&["branch", "feat/x", "--protected="]) else {
        panic!("parsed");
    };
    assert_eq!(err.kind(), ErrorKind::ValueValidation);
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
        commit(&[
            "missing",
            "--type",
            "feat",
            "--type-defaults",
            "never",
            "--type-dir",
            "kinds",
            "--scope",
            "core,cli",
            "--scope-dir",
            "crates",
            "--scope-exclude",
            "root",
            "--scope-mode",
            "optional"
        ]),
        CommitRawConfig {
            file: "missing".into(),
            types: with_dirs(raw(Some(&["feat"]), &[]), &["kinds"]),
            type_defaults: Some(TypeDefaultsClapMode::Never),
            scopes: with_dirs(raw(Some(&["core", "cli"]), &["root"]), &["crates"]),
            scope_mode: Some(CommitScopeClapMode::Optional),
        }
    );
}

#[test]
fn commit_options_are_none_when_omitted() {
    assert_eq!(
        commit(&["missing"]),
        CommitRawConfig {
            file: "missing".into(),
            types: ListRawValues::default(),
            type_defaults: None,
            scopes: ListRawValues::default(),
            scope_mode: None,
        }
    );
}

#[test]
fn type_defaults_accepts_each_mode_in_both_commands() {
    for (written, mode) in [
        ("auto", TypeDefaultsClapMode::Auto),
        ("always", TypeDefaultsClapMode::Always),
        ("never", TypeDefaultsClapMode::Never),
    ] {
        assert_eq!(
            branch(&["feat/x", "--type-defaults", written]).type_defaults,
            Some(mode),
            "{written}"
        );
        assert_eq!(
            commit(&["missing", "--type-defaults", written]).type_defaults,
            Some(mode),
            "{written}"
        );
    }
}

#[test]
fn scope_mode_accepts_each_mode_without_scopes() {
    for (written, mode) in [
        ("auto", CommitScopeClapMode::Auto),
        ("required", CommitScopeClapMode::Required),
        ("optional", CommitScopeClapMode::Optional),
        ("forbidden", CommitScopeClapMode::Forbidden),
    ] {
        assert_eq!(
            commit(&["missing", "--scope-mode", written]).scope_mode,
            Some(mode),
            "{written}"
        );
    }
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
        (&["commit"], ErrorKind::MissingRequiredArgument),
        (&["branch", "feat/x", "feat/y"], ErrorKind::UnknownArgument),
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
        (
            &["branch", "feat/x", "--scope-exclude", "core"],
            ErrorKind::UnknownArgument,
        ),
        (
            &["branch", "feat/x", "--type-defaults", "on"],
            ErrorKind::InvalidValue,
        ),
    ] {
        let Err(err) = parse(args) else {
            panic!("{args:?} parsed");
        };
        assert_eq!(err.kind(), kind, "{args:?}");
    }
}
