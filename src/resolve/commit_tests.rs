use std::{io::Write, path::PathBuf};

use tempfile::NamedTempFile;

use super::CommitRawConfig;
use crate::{
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
    }
}

fn resolve_error(file: PathBuf) -> CommitResolveError {
    config(file, None).resolve().unwrap_err()
}

#[test]
fn reads_header_from_message_file() {
    let file = message_file(b"# comment\n\nfeat(core): x\n\nbody\n");
    let config = CommitRawConfig {
        file: file.path().to_owned(),
        types: values(&["feat", "fix"]),
        scopes: values(&["core", "cli", "core"]),
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
fn fails_on_missing_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("missing");
    let err = resolve_error(path.clone());
    assert_eq!(err.file, path);
    assert_eq!(err.source.kind(), std::io::ErrorKind::NotFound);
}

#[test]
fn fails_on_directory() {
    let dir = tempfile::tempdir().unwrap();
    let err = resolve_error(dir.path().to_owned());
    assert_eq!(err.file, dir.path());
    assert_eq!(err.source.kind(), std::io::ErrorKind::IsADirectory);
}

#[test]
fn fails_on_message_outside_utf8() {
    let file = message_file(b"feat: \xff\n");
    let err = resolve_error(file.path().to_owned());
    assert_eq!(err.file, file.path());
    assert_eq!(err.source.kind(), std::io::ErrorKind::InvalidData);
}
