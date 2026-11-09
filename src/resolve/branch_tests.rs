use super::BranchRawConfig;
use crate::{
    cli::branch::BranchDetachedClapRule,
    lint::branch::{BranchDetachedRule, BranchLintRules, BranchLintTarget},
    name_set::NameSet,
    resolve::DEFAULT_TYPES,
};

fn values(values: &[&str]) -> Option<Vec<String>> {
    Some(values.iter().copied().map(String::from).collect())
}

fn set(names: &[&str]) -> NameSet {
    names.iter().copied().collect()
}

fn config(branch: &str) -> BranchRawConfig {
    BranchRawConfig {
        branch: branch.to_owned(),
        types: None,
        protected: None,
        detached: None,
    }
}

fn detached_rule(rule: Option<BranchDetachedClapRule>) -> BranchDetachedRule {
    let config = BranchRawConfig {
        detached: rule,
        ..config("feat/x")
    };
    config.resolve().1.detached
}

#[test]
fn takes_branch_as_target_and_interprets_lists() {
    let config = BranchRawConfig {
        branch: "feat/x".to_owned(),
        types: values(&["feat", "fix", "feat"]),
        protected: values(&["main", "develop", "main"]),
        detached: Some(BranchDetachedClapRule::Deny),
    };
    assert_eq!(
        config.resolve(),
        (
            BranchLintTarget::Branch("feat/x".to_owned()),
            BranchLintRules {
                types: set(&["feat", "fix"]),
                protected: set(&["main", "develop"]),
                detached: BranchDetachedRule::Forbidden,
            }
        )
    );
}

#[test]
fn takes_head_as_detached_head() {
    assert_eq!(config("HEAD").resolve().0, BranchLintTarget::Detached);
}

#[test]
fn takes_names_other_than_head_as_branches() {
    for branch in ["head", "Head", "HEAD/x", "feat/HEAD", " HEAD"] {
        assert_eq!(
            config(branch).resolve().0,
            BranchLintTarget::Branch(branch.to_owned()),
            "{branch}"
        );
    }
}

#[test]
fn protects_nothing_and_allows_default_types_and_detached_head_when_omitted() {
    let (_, rules) = config("main").resolve();
    assert_eq!(
        rules,
        BranchLintRules {
            types: set(&DEFAULT_TYPES),
            protected: NameSet::default(),
            detached: BranchDetachedRule::Allowed,
        }
    );
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
