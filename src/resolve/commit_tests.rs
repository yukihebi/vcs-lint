use std::{
    io::{self, Write},
    path::PathBuf,
};

use tempfile::NamedTempFile;

use super::CommitRawConfig;
use crate::{
    cli::commit::CommitScopeClapMode,
    lint::commit::{CommitLintRules, CommitLintTarget, CommitScopeRule},
    name_set::NameSet,
    report::commit::CommitResolveError,
    resolve::DEFAULT_TYPES,
};

fn values(values: &[&str]) -> Option<Vec<String>> {
    Some(values.iter().copied().map(String::from).collect())
}

fn set(names: &[&str]) -> NameSet {
    names.iter().copied().collect()
}

fn message_file(content: &[u8]) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(content).unwrap();
    file
}

fn config(file: PathBuf, scopes: Option<Vec<String>>) -> CommitRawConfig {
    CommitRawConfig {
        file,
        types: None,
        scopes,
        scope_mode: None,
    }
}

fn read_error(file: PathBuf) -> (PathBuf, io::Error) {
    match config(file, None).resolve() {
        Err(CommitResolveError::Unreadable { file, source }) => (file, source),
        other => panic!("{other:?}"),
    }
}

#[test]
fn reads_header_from_message_file() {
    let file = message_file(b"# comment\n\nfeat(core): x\n\nbody\n");
    let config = CommitRawConfig {
        file: file.path().to_owned(),
        types: values(&["feat", "fix"]),
        scopes: values(&["core", "cli", "core"]),
        scope_mode: None,
    };
    assert_eq!(
        config.resolve().unwrap(),
        (
            CommitLintTarget::Header("feat(core): x".to_owned()),
            CommitLintRules {
                types: set(&["feat", "fix"]),
                scopes: CommitScopeRule::Required(set(&["core", "cli"])),
            }
        )
    );
}

#[test]
fn forbids_scopes_and_allows_default_types_when_omitted() {
    let file = message_file(b"feat: x\n");
    let (_, rules) = config(file.path().to_owned(), None).resolve().unwrap();
    assert_eq!(
        rules,
        CommitLintRules {
            types: set(&DEFAULT_TYPES),
            scopes: CommitScopeRule::Forbidden,
        }
    );
}

#[test]
fn auto_requires_scopes_only_when_given() {
    for mode in [None, Some(CommitScopeClapMode::Auto)] {
        assert_eq!(
            CommitRawConfig::resolve_scopes(mode, values(&["core", "cli"])).unwrap(),
            CommitScopeRule::Required(set(&["core", "cli"])),
            "{mode:?}"
        );
        assert_eq!(
            CommitRawConfig::resolve_scopes(mode, None).unwrap(),
            CommitScopeRule::Forbidden,
            "{mode:?}"
        );
    }
}

#[test]
fn required_and_optional_allow_given_scopes() {
    assert_eq!(
        CommitRawConfig::resolve_scopes(
            Some(CommitScopeClapMode::Required),
            values(&["core", "cli", "core"])
        )
        .unwrap(),
        CommitScopeRule::Required(set(&["core", "cli"]))
    );
    assert_eq!(
        CommitRawConfig::resolve_scopes(
            Some(CommitScopeClapMode::Optional),
            values(&["core", "cli", "core"])
        )
        .unwrap(),
        CommitScopeRule::Optional(set(&["core", "cli"]))
    );
}

#[test]
fn forbidden_ignores_given_scopes() {
    assert_eq!(
        CommitRawConfig::resolve_scopes(Some(CommitScopeClapMode::Forbidden), values(&["core"]))
            .unwrap(),
        CommitScopeRule::Forbidden
    );
}

#[test]
fn required_and_optional_fail_without_scopes_before_reading_file() {
    let dir = tempfile::tempdir().unwrap();
    for mode in [CommitScopeClapMode::Required, CommitScopeClapMode::Optional] {
        let config = CommitRawConfig {
            scope_mode: Some(mode),
            ..config(dir.path().join("missing"), None)
        };
        let result = config.resolve();
        assert!(
            matches!(result, Err(CommitResolveError::NoScopes { mode: found }) if found == mode),
            "{result:?}"
        );
    }
}

#[test]
fn fails_on_missing_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("missing");
    let (file, source) = read_error(path.clone());
    assert_eq!(file, path);
    assert_eq!(source.kind(), io::ErrorKind::NotFound);
}

#[test]
fn fails_on_directory() {
    let dir = tempfile::tempdir().unwrap();
    let (file, source) = read_error(dir.path().to_owned());
    assert_eq!(file, dir.path());
    assert_eq!(source.kind(), io::ErrorKind::IsADirectory);
}

#[test]
fn fails_on_message_outside_utf8() {
    let file = message_file(b"feat: \xff\n");
    let (path, source) = read_error(file.path().to_owned());
    assert_eq!(path, file.path());
    assert_eq!(source.kind(), io::ErrorKind::InvalidData);
}
