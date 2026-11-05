use std::io::Write;

use clap::Parser;
use tempfile::NamedTempFile;

use super::Cli;
use crate::{
    Lint,
    rules::{
        BranchRules, CommitRules, list::List, protected::ProtectedRule, scopes::ScopeRule,
        types::TypeRule,
    },
};

fn lint(args: &[&str]) -> Lint {
    Cli::try_parse_from(args).unwrap().into_lint().unwrap()
}

fn branch_rules(args: &[&str]) -> BranchRules {
    let Lint::Branch { rules, .. } = lint(&[&["branch", "feat/x"], args].concat()) else {
        panic!("not a branch lint");
    };
    rules
}

fn types(values: &[&str]) -> TypeRule {
    TypeRule::new(values.iter().copied().collect())
}

fn protected(values: &[&str]) -> ProtectedRule {
    ProtectedRule::new(values.iter().copied().collect())
}

fn message_file(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(content.as_bytes()).unwrap();
    file
}

#[test]
fn branch_takes_name_and_rules() {
    assert_eq!(
        lint(&["branch", "feat/x", "--type", "feat", "--protected", "main"]),
        Lint::Branch {
            name: "feat/x".to_owned(),
            rules: BranchRules {
                types: types(&["feat"]),
                protected: protected(&["main"]),
            },
        }
    );
}

#[test]
fn types_default_when_omitted() {
    assert_eq!(branch_rules(&[]).types, TypeRule::new(List::default()));
}

#[test]
fn types_accept_repeated_flags() {
    assert_eq!(
        branch_rules(&["--type", "feat", "--type", "fix"]).types,
        types(&["feat", "fix"])
    );
}

#[test]
fn types_accept_comma_separated_values() {
    assert_eq!(
        branch_rules(&["--type", "feat,fix"]).types,
        types(&["feat", "fix"])
    );
}

#[test]
fn types_accept_mixed_forms() {
    assert_eq!(
        branch_rules(&["--type", "feat,fix", "--type=docs", "--type", "ci"]).types,
        types(&["feat", "fix", "docs", "ci"])
    );
}

#[test]
fn types_ignore_empty_values() {
    assert_eq!(
        branch_rules(&["--type", ",feat,,fix,", "--type", ""]).types,
        types(&["feat", "fix"])
    );
}

#[test]
fn types_default_when_every_value_is_empty() {
    assert_eq!(
        branch_rules(&["--type", ",,"]).types,
        TypeRule::new(List::default())
    );
}

#[test]
fn protected_accepts_mixed_forms() {
    assert_eq!(
        branch_rules(&["--protected", "main,develop,", "--protected", "release"]).protected,
        protected(&["main", "develop", "release"])
    );
}

#[test]
fn protected_is_empty_when_omitted() {
    assert_eq!(branch_rules(&[]).protected, ProtectedRule::default());
}

#[test]
fn options_may_precede_positional_argument() {
    assert_eq!(
        lint(&["branch", "--type", "feat", "feat/x"]),
        lint(&["branch", "feat/x", "--type", "feat"])
    );
}

#[test]
fn commit_reads_message_file() {
    let file = message_file("feat(core): x\n");
    let path = file.path().to_str().unwrap();
    assert_eq!(
        lint(&["commit", path, "--scope", "root,core", "--scope", "cli"]),
        Lint::Commit {
            message: "feat(core): x\n".to_owned(),
            rules: CommitRules {
                types: TypeRule::new(List::default()),
                scopes: ScopeRule::new(["root", "core", "cli"].into_iter().collect()),
            },
        }
    );
}

#[test]
fn commit_forbids_scopes_when_omitted() {
    let file = message_file("feat: x\n");
    let path = file.path().to_str().unwrap();
    let Lint::Commit { rules, .. } = lint(&["commit", path]) else {
        panic!("not a commit lint");
    };
    assert_eq!(rules.scopes, ScopeRule::Forbidden);
}

#[test]
fn commit_fails_on_unreadable_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("missing");
    let cli = Cli::try_parse_from(["commit", path.to_str().unwrap()]).unwrap();
    let err = cli.into_lint().unwrap_err();
    assert_eq!(err.file, path);
    assert_eq!(err.source.kind(), std::io::ErrorKind::NotFound);
}
