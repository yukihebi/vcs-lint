use super::BranchRawConfig;
use crate::{
    lint::branch::{BranchLintRules, BranchLintTarget},
    name_set::NameSet,
    resolve::DEFAULT_TYPES,
};

fn values(values: &[&str]) -> Option<Vec<String>> {
    Some(values.iter().copied().map(String::from).collect())
}

fn set(names: &[&str]) -> NameSet {
    names.iter().copied().collect()
}

#[test]
fn takes_branch_as_target_and_interprets_lists() {
    let config = BranchRawConfig {
        branch: "feat/x".to_owned(),
        types: values(&["feat", " fix", "feat"]),
        protected: values(&["main,", "", "develop"]),
    };
    assert_eq!(
        config.resolve(),
        (
            BranchLintTarget {
                branch: "feat/x".to_owned()
            },
            BranchLintRules {
                types: set(&["feat", "fix"]),
                protected: set(&["main,", "develop"]),
            }
        )
    );
}

#[test]
fn protects_nothing_and_allows_default_types_when_omitted() {
    let config = BranchRawConfig {
        branch: "main".to_owned(),
        types: None,
        protected: None,
    };
    let (_, rules) = config.resolve();
    assert_eq!(
        rules,
        BranchLintRules {
            types: set(&DEFAULT_TYPES),
            protected: NameSet::default(),
        }
    );
}

#[test]
fn protects_nothing_when_every_value_is_empty() {
    let config = BranchRawConfig {
        branch: "main".to_owned(),
        types: None,
        protected: values(&["", " "]),
    };
    assert_eq!(config.resolve().1.protected, NameSet::default());
}
