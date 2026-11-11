use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use clap::error::ErrorKind;

use super::{
    ClapCommand, CommandName, NetworkClapAccess, Parsed, SeverityClapLevel, TypeDefaultsClapMode,
    branch::BranchDetachedClapRule, commit::CommitScopeClapMode, history::HistoryBaseClapSource,
};
use crate::{
    report::config_file::ValueRejection,
    resolve::{
        branch::BranchRawConfig, commit::CommitRawConfig, history::HistoryRawConfig,
        list::ListRawValues,
    },
};

fn os_strings(args: &[&str]) -> Vec<OsString> {
    args.iter().map(OsString::from).collect()
}

fn parse(args: &[&str]) -> Result<Parsed, clap::Error> {
    Parsed::parse(os_strings(args))
}

fn command(args: &[&str]) -> ClapCommand {
    let Ok(Parsed::Command { command, .. }) = parse(args) else {
        panic!("{args:?} is not a command");
    };
    *command
}

fn written(args: &[&str]) -> Vec<String> {
    let Ok(Parsed::Command { written, .. }) = parse(args) else {
        panic!("{args:?} is not a command");
    };
    written
}

fn branch(args: &[&str]) -> BranchRawConfig {
    let ClapCommand::Branch(args) = command(&[&["branch"], args].concat()) else {
        panic!("not a branch command");
    };
    args.into_raw_config()
}

fn commit(args: &[&str]) -> CommitRawConfig {
    let ClapCommand::Commit(args) = command(&[&["commit"], args].concat()) else {
        panic!("not a commit command");
    };
    args.into_raw_config()
}

fn history(args: &[&str]) -> HistoryRawConfig {
    let ClapCommand::History(args) = command(&[&["history"], args].concat()) else {
        panic!("not a history command");
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
fn history_takes_arguments_and_options() {
    assert_eq!(
        history(&[
            "feat/x",
            "--base",
            "main",
            "--base-from",
            "remote",
            "--remote",
            "upstream",
            "--network",
            "deny",
        ]),
        HistoryRawConfig {
            rev: Some("feat/x".to_owned()),
            base: Some("main".to_owned()),
            base_from: Some(HistoryBaseClapSource::Remote),
            remote: Some("upstream".to_owned()),
            network: Some(NetworkClapAccess::Deny),
        }
    );
}

#[test]
fn history_options_are_none_when_omitted() {
    assert_eq!(
        history(&[]),
        HistoryRawConfig {
            rev: None,
            base: None,
            base_from: None,
            remote: None,
            network: None,
        }
    );
}

#[test]
fn history_accepts_each_source_and_network_access() {
    assert_eq!(
        history(&["--base-from", "local"]).base_from,
        Some(HistoryBaseClapSource::Local)
    );
    assert_eq!(
        history(&["--network", "allow"]).network,
        Some(NetworkClapAccess::Allow)
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
        &["history", "--help"],
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
        (&["history", "--base", ""], ErrorKind::InvalidValue),
        (&["history", "--remote", ""], ErrorKind::InvalidValue),
        (&["history", "--base-from", "both"], ErrorKind::InvalidValue),
        (&["history", "--network", "Deny"], ErrorKind::InvalidValue),
        (&["history", "HEAD", "main"], ErrorKind::UnknownArgument),
        (&["history", "--type", "feat"], ErrorKind::UnknownArgument),
    ] {
        let Err(err) = parse(args) else {
            panic!("{args:?} parsed");
        };
        assert_eq!(err.kind(), kind, "{args:?}");
    }
}

#[test]
fn severity_accepts_each_level_in_every_command() {
    for (written, level) in [
        ("error", SeverityClapLevel::Error),
        ("warning", SeverityClapLevel::Warning),
    ] {
        assert_eq!(
            command(&["branch", "--severity", written]).severity(),
            level,
            "{written}"
        );
        assert_eq!(
            command(&["commit", "missing", "--severity", written]).severity(),
            level,
            "{written}"
        );
        assert_eq!(
            command(&["history", "--severity", written]).severity(),
            level,
            "{written}"
        );
    }
}

#[test]
fn severity_is_error_when_omitted() {
    assert_eq!(command(&["branch"]).severity(), SeverityClapLevel::Error);
    assert_eq!(
        command(&["commit", "missing"]).severity(),
        SeverityClapLevel::Error
    );
    assert_eq!(command(&["history"]).severity(), SeverityClapLevel::Error);
}

#[test]
fn config_is_given_to_every_command() {
    for args in [
        &["branch", "--config", "vcs-lint.toml"][..],
        &["commit", "missing", "--config=vcs-lint.toml"],
        &["history", "--config", "vcs-lint.toml"],
    ] {
        assert_eq!(
            command(args).config(),
            Some(Path::new("vcs-lint.toml")),
            "{args:?}"
        );
    }
}

#[test]
fn config_is_none_when_omitted() {
    assert_eq!(command(&["branch"]).config(), None);
    assert_eq!(command(&["commit", "missing"]).config(), None);
    assert_eq!(command(&["history"]).config(), None);
}

#[test]
fn config_rejects_empty_path() {
    let Err(err) = parse(&["branch", "--config", ""]) else {
        panic!("parsed");
    };
    assert_eq!(err.kind(), ErrorKind::InvalidValue);
}

#[test]
fn command_is_named_as_written() {
    assert_eq!(command(&["branch"]).name(), CommandName::Branch);
    assert_eq!(command(&["commit", "missing"]).name(), CommandName::Commit);
    assert_eq!(command(&["history"]).name(), CommandName::History);
    for name in CommandName::ALL {
        assert_eq!(CommandName::from_name(name.as_str()), Some(name));
    }
    assert_eq!(CommandName::from_name("Branch"), None);
}

#[test]
fn command_options_are_those_with_long_names() {
    let options = |command: CommandName| -> Vec<(String, bool)> {
        let syntax = command.syntax();
        syntax
            .options()
            .map(|option| (option.name.to_owned(), option.is_list()))
            .collect()
    };
    let expected = |names: &[(&str, bool)]| -> Vec<(String, bool)> {
        names
            .iter()
            .map(|&(name, is_list)| (name.to_owned(), is_list))
            .collect()
    };
    assert_eq!(
        options(CommandName::Branch),
        expected(&[
            ("type", true),
            ("type-dir", true),
            ("type-exclude", true),
            ("type-defaults", false),
            ("protected", true),
            ("protected-dir", true),
            ("protected-exclude", true),
            ("detached", false),
            ("severity", false),
            ("config", false),
        ])
    );
    assert_eq!(
        options(CommandName::Commit),
        expected(&[
            ("type", true),
            ("type-dir", true),
            ("type-exclude", true),
            ("type-defaults", false),
            ("scope", true),
            ("scope-dir", true),
            ("scope-exclude", true),
            ("scope-mode", false),
            ("severity", false),
            ("config", false),
        ])
    );
    assert_eq!(
        options(CommandName::History),
        expected(&[
            ("base", false),
            ("base-from", false),
            ("remote", false),
            ("network", false),
            ("severity", false),
            ("config", false),
        ])
    );
}

#[test]
fn command_option_is_found_by_long_name() {
    let syntax = CommandName::Commit.syntax();
    assert_eq!(
        syntax.option("scope-mode").map(|option| option.name),
        Some("scope-mode")
    );
    for name in ["scope_mode", "protected", "file", "help"] {
        assert!(syntax.option(name).is_none(), "{name}");
    }
}

#[test]
fn option_accepts_values_command_line_accepts() {
    let syntax = CommandName::Commit.syntax();
    for (name, value) in [
        ("scope", "core"),
        ("scope", " core,,cli "),
        ("scope", "-x"),
        ("scope-dir", "crates"),
        ("scope-mode", "optional"),
        ("type-defaults", "never"),
    ] {
        let option = syntax.option(name).unwrap();
        assert_eq!(option.check(value), Ok(()), "{name} {value:?}");
    }
}

#[test]
fn option_rejects_value_outside_its_possible_values() {
    let syntax = CommandName::Branch.syntax();
    for value in ["Deny", "allow,deny", ""] {
        assert_eq!(
            syntax.option("detached").unwrap().check(value),
            Err(ValueRejection::NotPossible {
                possible: vec!["allow".to_owned(), "deny".to_owned()]
            }),
            "{value:?}"
        );
    }
}

#[test]
fn option_rejects_value_its_parser_rejects_with_reason() {
    let syntax = CommandName::Branch.syntax();
    for value in ["", " , "] {
        assert_eq!(
            syntax.option("protected").unwrap().check(value),
            Err(ValueRejection::Invalid {
                reason: "every value is empty".to_owned()
            }),
            "{value:?}"
        );
    }
}

#[test]
fn written_options_are_those_given_on_command_line() {
    assert_eq!(
        written(&[
            "commit",
            "missing",
            "--scope",
            "core",
            "--scope=cli",
            "--scope-mode",
            "optional",
            "--config",
            "vcs-lint.toml"
        ]),
        ["scope", "scope-mode", "config"]
    );
    assert_eq!(written(&["branch", "feat/x"]), [] as [&str; 0]);
}

#[test]
fn leading_flags_are_given_right_after_command_name() {
    let Ok(Parsed::Command { command, written }) = Parsed::parse_with_leading(
        os_strings(&["commit", "missing", "--scope", "cli"]),
        os_strings(&["--scope=root", "--scope-mode=optional"]),
    ) else {
        panic!("not a command");
    };
    let ClapCommand::Commit(args) = *command else {
        panic!("not a commit command");
    };
    let config = args.into_raw_config();
    assert_eq!(written, ["scope", "scope-mode"]);
    assert_eq!(config.scopes.given, values(&["root", "cli"]));
    assert_eq!(config.scope_mode, Some(CommitScopeClapMode::Optional));
}

#[test]
fn leading_flags_follow_rules_of_command_line() {
    for (leading, kind) in [
        (&["--detached=deny"][..], ErrorKind::ArgumentConflict),
        (&["--protected="], ErrorKind::ValueValidation),
        (&["--scope=core"], ErrorKind::UnknownArgument),
    ] {
        let Err(err) = Parsed::parse_with_leading(
            os_strings(&["branch", "HEAD", "--detached", "allow"]),
            os_strings(leading),
        ) else {
            panic!("{leading:?} parsed");
        };
        assert_eq!(err.kind(), kind, "{leading:?}");
    }
}
