use std::{ffi::OsString, fs, path::Path};

use tempfile::tempdir;

use super::{ConfigFile, ConfigReader};
use crate::{
    cli::CommandName,
    env::Env,
    report::config_file::{ConfigFileError, ConfigKey, ConfigKeyError, ConfigKeyErrorKind},
};

fn read(text: &str) -> Result<ConfigFile, Vec<ConfigKeyError>> {
    ConfigReader::new().read(text.parse().unwrap())
}

/// The flags each command reads from a valid file holding `text`.
fn flags(text: &str) -> [(CommandName, Vec<String>); 2] {
    let file = read(text).unwrap();
    CommandName::ALL.map(|command| {
        let flags = file.flags(command).into_iter();
        (
            command,
            flags.map(|flag| flag.into_string().unwrap()).collect(),
        )
    })
}

fn expected(branch: &[&str], commit: &[&str]) -> [(CommandName, Vec<String>); 2] {
    let strings = |flags: &[&str]| flags.iter().copied().map(String::from).collect();
    [
        (CommandName::Branch, strings(branch)),
        (CommandName::Commit, strings(commit)),
    ]
}

fn top(name: &str, kind: ConfigKeyErrorKind) -> ConfigKeyError {
    ConfigKeyError {
        key: ConfigKey {
            table: None,
            name: name.to_owned(),
        },
        kind,
    }
}

fn in_table(command: CommandName, name: &str, kind: ConfigKeyErrorKind) -> ConfigKeyError {
    ConfigKeyError {
        key: ConfigKey {
            table: Some(command),
            name: name.to_owned(),
        },
        kind,
    }
}

fn env(config: Option<&str>) -> Env {
    Env {
        required_version: None,
        config: config.map(OsString::from),
    }
}

#[test]
fn empty_file_gives_no_flags() {
    assert_eq!(flags(""), expected(&[], &[]));
}

#[test]
fn top_level_keys_are_read_by_commands_having_them() {
    assert_eq!(
        flags("type = [\"feat\"]\nscope = [\"core\"]\nprotected = [\"main\"]"),
        expected(
            &["--type=feat", "--protected=main"],
            &["--type=feat", "--scope=core"]
        )
    );
}

#[test]
fn table_keys_are_read_by_their_command_only() {
    assert_eq!(
        flags("[branch]\ntype = [\"feat\"]\n[commit]\ntype = [\"fix\"]"),
        expected(&["--type=feat"], &["--type=fix"])
    );
}

#[test]
fn top_level_keys_are_read_before_table_keys_written_before_them() {
    assert_eq!(
        flags("commit = { scope = [\"cli\"] }\nscope = [\"root\"]"),
        expected(&[], &["--scope=root", "--scope=cli"])
    );
}

#[test]
fn key_at_both_levels_is_read_twice() {
    assert_eq!(
        flags("detached = \"deny\"\n[branch]\ndetached = \"allow\""),
        expected(&["--detached=deny", "--detached=allow"], &[])
    );
}

#[test]
fn keys_are_read_in_order_written() {
    assert_eq!(
        flags("scope-mode = \"optional\"\ntype-defaults = \"never\"\nscope = [\"core\"]"),
        expected(
            &["--type-defaults=never"],
            &[
                "--scope-mode=optional",
                "--type-defaults=never",
                "--scope=core"
            ]
        )
    );
}

#[test]
fn list_option_gives_one_flag_per_string() {
    assert_eq!(
        flags("[commit]\nscope = [\"root\", \"cli,core\", \"\"]\nscope-exclude = []"),
        expected(&[], &["--scope=root", "--scope=cli,core", "--scope="])
    );
}

#[test]
fn values_are_bound_to_their_flags() {
    assert_eq!(
        flags("[commit]\nscope = [\"-x\", \"--scope\"]\nscope-mode = \"-h\""),
        expected(&[], &["--scope=-x", "--scope=--scope", "--scope-mode=-h"])
    );
}

#[test]
fn rejects_key_no_command_has() {
    for name in ["scopes", "file", "name", "version", "Type"] {
        assert_eq!(
            read(&format!("{name} = \"x\"")).unwrap_err(),
            [top(name, ConfigKeyErrorKind::Unknown)]
        );
    }
}

#[test]
fn rejects_key_its_command_does_not_have() {
    assert_eq!(
        read("[branch]\nscope = [\"core\"]\n[commit]\nprotected = [\"main\"]").unwrap_err(),
        [
            in_table(CommandName::Branch, "scope", ConfigKeyErrorKind::Unknown),
            in_table(
                CommandName::Commit,
                "protected",
                ConfigKeyErrorKind::Unknown
            ),
        ]
    );
}

#[test]
fn rejects_options_outside_rules_at_both_levels() {
    assert_eq!(
        read("config = \"a.toml\"\nhelp = \"x\"\n[commit]\nconfig = \"b.toml\"").unwrap_err(),
        [
            top("config", ConfigKeyErrorKind::Unwritable),
            top("help", ConfigKeyErrorKind::Unwritable),
            in_table(
                CommandName::Commit,
                "config",
                ConfigKeyErrorKind::Unwritable
            ),
        ]
    );
}

#[test]
fn rejects_command_key_other_than_table() {
    assert_eq!(
        read("branch = \"main\"\ncommit = [\"x\"]").unwrap_err(),
        [
            top("branch", ConfigKeyErrorKind::NotTable),
            top("commit", ConfigKeyErrorKind::NotTable),
        ]
    );
}

#[test]
fn rejects_list_option_other_than_array_of_strings() {
    for value in [
        "\"feat\"",
        "[\"feat\", 1]",
        "[[\"feat\"]]",
        "{ feat = \"x\" }",
    ] {
        assert_eq!(
            read(&format!("type = {value}")).unwrap_err(),
            [top("type", ConfigKeyErrorKind::NotStringArray)],
            "{value}"
        );
    }
}

#[test]
fn rejects_single_option_other_than_string() {
    for value in ["[\"deny\"]", "true", "1", "{ deny = true }"] {
        assert_eq!(
            read(&format!("[branch]\ndetached = {value}")).unwrap_err(),
            [in_table(
                CommandName::Branch,
                "detached",
                ConfigKeyErrorKind::NotString
            )],
            "{value}"
        );
    }
}

#[test]
fn rejects_file_invalid_for_one_command_with_every_invalid_key_in_order() {
    assert_eq!(
        read("type = \"feat\"\nscope = [\"core\"]\n[branch]\nscope-mode = \"required\"\n[commit]\nscope-mode = 1")
            .unwrap_err(),
        [
            top("type", ConfigKeyErrorKind::NotStringArray),
            in_table(CommandName::Branch, "scope-mode", ConfigKeyErrorKind::Unknown),
            in_table(CommandName::Commit, "scope-mode", ConfigKeyErrorKind::NotString),
        ]
    );
}

#[test]
fn path_given_takes_precedence_over_variable() {
    assert_eq!(
        ConfigFile::path(Some(Path::new("given.toml")), &env(Some("var.toml"))),
        Some(Path::new("given.toml"))
    );
}

#[test]
fn path_is_read_from_variable_when_not_given() {
    assert_eq!(
        ConfigFile::path(None, &env(Some(" var.toml"))),
        Some(Path::new(" var.toml"))
    );
}

#[test]
fn path_is_none_without_variable_or_with_empty_one() {
    assert_eq!(ConfigFile::path(None, &env(None)), None);
    assert_eq!(ConfigFile::path(None, &env(Some(""))), None);
}

#[test]
fn reads_valid_file() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("vcs-lint.toml");
    fs::write(&path, "[branch]\nprotected = [\"main\"]\n").unwrap();
    assert_eq!(
        ConfigFile::read(&path).unwrap().flags(CommandName::Branch),
        [OsString::from("--protected=main")]
    );
}

#[test]
fn read_fails_on_missing_file() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("missing.toml");
    let Err(ConfigFileError::Unreadable { path: failed, .. }) = ConfigFile::read(&path) else {
        panic!("read");
    };
    assert_eq!(failed, path);
}

#[test]
fn read_fails_on_file_other_than_utf8_text() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("vcs-lint.toml");
    fs::write(&path, b"type = [\"\xff\"]\n").unwrap();
    assert!(matches!(
        ConfigFile::read(&path),
        Err(ConfigFileError::Unreadable { .. })
    ));
}

#[test]
fn read_fails_on_file_other_than_toml() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("vcs-lint.toml");
    fs::write(&path, "type = feat\n").unwrap();
    assert!(matches!(
        ConfigFile::read(&path),
        Err(ConfigFileError::Syntax { .. })
    ));
}

#[test]
fn read_fails_on_invalid_key_with_path() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("vcs-lint.toml");
    fs::write(&path, "scopes = []\n").unwrap();
    let Err(ConfigFileError::Keys { path: failed, keys }) = ConfigFile::read(&path) else {
        panic!("read");
    };
    assert_eq!(failed, path);
    assert_eq!(keys, [top("scopes", ConfigKeyErrorKind::Unknown)]);
}
