use super::{TypeRule, UnknownType};
use crate::rules::list::List;

fn rule(types: &[&str]) -> TypeRule {
    TypeRule::new(types.iter().copied().collect())
}

#[test]
fn accepts_listed_type() {
    assert_eq!(rule(&["feat", "fix"]).check("fix"), Ok(()));
}

#[test]
fn rejects_unlisted_type() {
    let rule = rule(&["feat", "fix"]);
    assert_eq!(
        rule.check("feature"),
        Err(UnknownType {
            found: "feature".to_owned(),
            allowed: ["feat", "fix"].into_iter().collect(),
        })
    );
}

#[test]
fn rejects_empty_type() {
    let rule = rule(&["feat"]);
    assert!(rule.check("").is_err());
}

#[test]
fn falls_back_to_default_types_when_none_are_given() {
    let rule = TypeRule::new(List::default());
    for ty in TypeRule::DEFAULT {
        assert_eq!(rule.check(ty), Ok(()));
    }
    assert!(rule.check("feature").is_err());
}

#[test]
fn given_types_replace_defaults() {
    assert!(rule(&["feat"]).check("fix").is_err());
}

#[test]
fn unknown_type_lists_allowed_types() {
    let rule = rule(&["feat", "fix"]);
    assert_eq!(
        rule.check("feature").unwrap_err().to_string(),
        "unknown type `feature`\n  allowed types: feat, fix"
    );
}
