use super::{TypeRule, UnknownType};
use crate::rules::list::List;

fn list(values: &[&str]) -> List {
    values.iter().copied().collect()
}

fn unknown(found: &str, allowed: List) -> Result<(), UnknownType> {
    Err(UnknownType {
        found: found.to_owned(),
        allowed,
    })
}

#[test]
fn accepts_listed_type() {
    assert_eq!(TypeRule::new(list(&["feat", "fix"])).check("fix"), Ok(()));
}

#[test]
fn rejects_unlisted_type() {
    assert_eq!(
        TypeRule::new(list(&["feat", "fix"])).check("feature"),
        unknown("feature", list(&["feat", "fix"]))
    );
}

#[test]
fn rejects_empty_type() {
    assert_eq!(
        TypeRule::new(list(&["feat"])).check(""),
        unknown("", list(&["feat"]))
    );
}

#[test]
fn falls_back_to_default_types_when_none_are_given() {
    let rule = TypeRule::new(List::default());
    for ty in TypeRule::DEFAULT {
        assert_eq!(rule.check(ty), Ok(()), "{ty}");
    }
    assert_eq!(
        rule.check("feature"),
        unknown("feature", list(&TypeRule::DEFAULT))
    );
}

#[test]
fn given_types_replace_defaults() {
    assert_eq!(
        TypeRule::new(list(&["feat"])).check("fix"),
        unknown("fix", list(&["feat"]))
    );
}

#[test]
fn unknown_type_lists_allowed_types() {
    assert_eq!(
        unknown("feature", list(&["feat", "fix"]))
            .unwrap_err()
            .to_string(),
        "unknown type `feature`\n  allowed types: feat, fix"
    );
}
