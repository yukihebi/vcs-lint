use super::BranchRawConfig;
use crate::{
    cli::{TypeDefaultsClapMode, branch::BranchDetachedClapRule},
    lint::branch::{BranchDetachedRule, BranchLintRules, BranchLintTarget},
    name_set::NameSet,
    report::{
        branch::BranchResolveError,
        list::{ListName, ListResolveError},
    },
    resolve::{DEFAULT_TYPES, list::ListRawValues},
};

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().copied().map(String::from).collect()
}

fn set(names: &[&str]) -> NameSet {
    names.iter().copied().collect()
}

fn raw(given: Option<&[&str]>, excluded: &[&str]) -> ListRawValues {
    ListRawValues {
        given: given.map(strings),
        excluded: strings(excluded),
    }
}

fn config(branch: &str) -> BranchRawConfig {
    BranchRawConfig {
        branch: branch.to_owned(),
        types: ListRawValues::default(),
        type_defaults: None,
        protected: ListRawValues::default(),
        detached: None,
    }
}

fn rules(config: BranchRawConfig) -> BranchLintRules {
    config.resolve().unwrap().1
}

fn target(branch: &str) -> BranchLintTarget {
    config(branch).resolve().unwrap().0
}

fn detached_rule(rule: Option<BranchDetachedClapRule>) -> BranchDetachedRule {
    rules(BranchRawConfig {
        detached: rule,
        ..config("feat/x")
    })
    .detached
}

#[test]
fn takes_branch_as_target_and_composes_lists() {
    let config = BranchRawConfig {
        branch: "feat/x".to_owned(),
        types: raw(Some(&["feat", "fix", "wip", "feat"]), &["wip"]),
        type_defaults: Some(TypeDefaultsClapMode::Never),
        protected: raw(Some(&["main", "develop", "main"]), &["develop"]),
        detached: Some(BranchDetachedClapRule::Deny),
    };
    assert_eq!(
        config.resolve().unwrap(),
        (
            BranchLintTarget::Branch("feat/x".to_owned()),
            BranchLintRules {
                types: set(&["feat", "fix"]),
                protected: set(&["main"]),
                detached: BranchDetachedRule::Forbidden,
            }
        )
    );
}

#[test]
fn takes_head_as_detached_head() {
    assert_eq!(target("HEAD"), BranchLintTarget::Detached);
}

#[test]
fn takes_names_other_than_head_as_branches() {
    for branch in ["head", "Head", "HEAD/x", "feat/HEAD", " HEAD"] {
        assert_eq!(
            target(branch),
            BranchLintTarget::Branch(branch.to_owned()),
            "{branch}"
        );
    }
}

#[test]
fn protects_nothing_and_allows_default_types_and_detached_head_when_omitted() {
    assert_eq!(
        rules(config("main")),
        BranchLintRules {
            types: set(&DEFAULT_TYPES),
            protected: NameSet::default(),
            detached: BranchDetachedRule::Allowed,
        }
    );
}

#[test]
fn protects_nothing_when_every_protected_branch_is_excluded() {
    let config = BranchRawConfig {
        protected: raw(Some(&["main"]), &["main"]),
        ..config("main")
    };
    assert_eq!(rules(config).protected, NameSet::default());
}

#[test]
fn fails_on_lists_that_cannot_be_composed() {
    for (config, err) in [
        (
            BranchRawConfig {
                protected: raw(None, &["main"]),
                ..config("feat/x")
            },
            ListResolveError::UnknownExclusions {
                list: ListName::Protected,
                unknown: set(&["main"]),
                listed: NameSet::default(),
            },
        ),
        (
            BranchRawConfig {
                type_defaults: Some(TypeDefaultsClapMode::Never),
                ..config("feat/x")
            },
            ListResolveError::Empty {
                list: ListName::Types,
            },
        ),
    ] {
        assert_eq!(config.resolve(), Err(BranchResolveError::List(err)));
    }
}

#[test]
fn interprets_detached_head_rule() {
    assert_eq!(
        detached_rule(Some(BranchDetachedClapRule::Allow)),
        BranchDetachedRule::Allowed
    );
    assert_eq!(
        detached_rule(Some(BranchDetachedClapRule::Deny)),
        BranchDetachedRule::Forbidden
    );
}
