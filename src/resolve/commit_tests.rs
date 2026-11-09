use std::{
    io::{self, Write},
    path::PathBuf,
};

use tempfile::NamedTempFile;

use super::CommitRawConfig;
use crate::{
    cli::{TypeDefaultsClapMode, commit::CommitScopeClapMode},
    git::Git,
    lint::commit::{CommitLintRules, CommitLintTarget, CommitScopeRule},
    name_set::NameSet,
    report::{
        commit::CommitResolveError,
        list::{ListName, ListResolveError, UnmatchableList},
    },
    resolve::{DEFAULT_TYPES, list::ListRawValues},
};

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().copied().map(String::from).collect()
}

fn raw(given: Option<&[&str]>, excluded: &[&str]) -> ListRawValues {
    ListRawValues {
        given: given.map(strings),
        dirs: None,
        excluded: strings(excluded),
    }
}

fn given(values: &[&str]) -> ListRawValues {
    raw(Some(values), &[])
}

fn set(names: &[&str]) -> NameSet {
    names.iter().copied().collect()
}

fn message_file(content: &[u8]) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(content).unwrap();
    file
}

fn config(file: PathBuf, scopes: ListRawValues) -> CommitRawConfig {
    CommitRawConfig {
        file,
        types: ListRawValues::default(),
        type_defaults: None,
        scopes,
        scope_mode: None,
    }
}

fn read_error(file: PathBuf) -> (PathBuf, io::Error) {
    match config(file, ListRawValues::default()).resolve(&Git) {
        Err(CommitResolveError::Unreadable { file, source }) => (file, source),
        other => panic!("{other:?}"),
    }
}

fn unmatchable(config: CommitRawConfig) -> Vec<UnmatchableList> {
    match config.resolve(&Git) {
        Err(CommitResolveError::Unmatchable(lists)) => lists,
        other => panic!("{other:?}"),
    }
}

#[test]
fn reads_header_from_message_file() {
    let file = message_file(b"# comment\n\nfeat(core): x\n\nbody\n");
    let config = CommitRawConfig {
        file: file.path().to_owned(),
        types: given(&["feat", "fix"]),
        type_defaults: None,
        scopes: raw(Some(&["core", "cli", "root", "core"]), &["root"]),
        scope_mode: None,
    };
    assert_eq!(
        config.resolve(&Git).unwrap(),
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
    let (_, rules) = config(file.path().to_owned(), ListRawValues::default())
        .resolve(&Git)
        .unwrap();
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
            CommitRawConfig::resolve_scopes(mode, given(&["core", "cli"]), &Git).unwrap(),
            CommitScopeRule::Required(set(&["core", "cli"])),
            "{mode:?}"
        );
        assert_eq!(
            CommitRawConfig::resolve_scopes(mode, ListRawValues::default(), &Git).unwrap(),
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
            given(&["core", "cli", "core"]),
            &Git
        )
        .unwrap(),
        CommitScopeRule::Required(set(&["core", "cli"]))
    );
    assert_eq!(
        CommitRawConfig::resolve_scopes(
            Some(CommitScopeClapMode::Optional),
            given(&["core", "cli", "core"]),
            &Git
        )
        .unwrap(),
        CommitScopeRule::Optional(set(&["core", "cli"]))
    );
}

#[test]
fn forbidden_ignores_given_scopes() {
    assert_eq!(
        CommitRawConfig::resolve_scopes(
            Some(CommitScopeClapMode::Forbidden),
            given(&["core"]),
            &Git
        )
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
            ..config(dir.path().join("missing"), ListRawValues::default())
        };
        let result = config.resolve(&Git);
        assert!(
            matches!(result, Err(CommitResolveError::NoScopes { mode: found }) if found == mode),
            "{result:?}"
        );
    }
}

#[test]
fn auto_fails_when_given_scopes_are_all_excluded() {
    for mode in [None, Some(CommitScopeClapMode::Auto)] {
        let result = CommitRawConfig::resolve_scopes(mode, raw(Some(&["core"]), &["core"]), &Git);
        assert!(
            matches!(
                result,
                Err(CommitResolveError::NoScopes {
                    mode: CommitScopeClapMode::Auto
                })
            ),
            "{result:?}"
        );
    }
}

#[test]
fn rejects_unknown_exclusion_even_when_scopes_are_forbidden() {
    let result = CommitRawConfig::resolve_scopes(
        Some(CommitScopeClapMode::Forbidden),
        raw(Some(&["core"]), &["cli"]),
        &Git,
    );
    assert!(
        matches!(
            &result,
            Err(CommitResolveError::List(ListResolveError::UnknownExclusions {
                list: ListName::Scopes,
                unknown,
                ..
            })) if *unknown == set(&["cli"])
        ),
        "{result:?}"
    );
}

#[test]
fn fails_without_types_before_reading_file() {
    let dir = tempfile::tempdir().unwrap();
    let config = CommitRawConfig {
        type_defaults: Some(TypeDefaultsClapMode::Never),
        ..config(dir.path().join("missing"), ListRawValues::default())
    };
    let result = config.resolve(&Git);
    assert!(
        matches!(
            result,
            Err(CommitResolveError::List(ListResolveError::Empty {
                list: ListName::Types
            }))
        ),
        "{result:?}"
    );
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

#[test]
fn reports_unmatchable_types_and_scopes_together_before_reading_file() {
    let dir = tempfile::tempdir().unwrap();
    let config = CommitRawConfig {
        types: given(&["feat", "wip!", "#x", "a b"]),
        ..config(
            dir.path().join("missing"),
            given(&["core", "core(unix", "a\nb"]),
        )
    };
    assert_eq!(
        unmatchable(config),
        [
            UnmatchableList {
                list: ListName::Types,
                values: set(&["wip!", "#x", "a b"]),
                requirement: CommitRawConfig::TYPES.wording,
            },
            UnmatchableList {
                list: ListName::Scopes,
                values: set(&["core(unix", "a\nb"]),
                requirement: CommitRawConfig::SCOPES.wording,
            },
        ]
    );
}

#[test]
fn reports_unmatchable_scopes_alone() {
    let dir = tempfile::tempdir().unwrap();
    let config = config(dir.path().join("missing"), given(&["core)"]));
    assert_eq!(
        unmatchable(config),
        [UnmatchableList {
            list: ListName::Scopes,
            values: set(&["core)"]),
            requirement: CommitRawConfig::SCOPES.wording,
        }]
    );
}

#[test]
fn checks_only_values_entering_the_rules() {
    let file = message_file(b"feat: x\n");
    let config = CommitRawConfig {
        types: raw(Some(&["feat", "wip!"]), &["wip!"]),
        scope_mode: Some(CommitScopeClapMode::Forbidden),
        ..config(file.path().to_owned(), given(&["core(unix"]))
    };
    assert_eq!(
        config.resolve(&Git).unwrap().1,
        CommitLintRules {
            types: set(&["feat"]),
            scopes: CommitScopeRule::Forbidden,
        }
    );
}

#[test]
fn allows_scopes_with_balanced_parentheses() {
    let file = message_file(b"feat: x\n");
    let (_, rules) = config(file.path().to_owned(), given(&["core(unix)", "a b: c"]))
        .resolve(&Git)
        .unwrap();
    assert_eq!(
        rules.scopes,
        CommitScopeRule::Required(set(&["core(unix)", "a b: c"]))
    );
}
