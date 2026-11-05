use super::{ScopeError, ScopeRule};
use crate::rules::list::List;

fn list(values: &[&str]) -> List {
    values.iter().copied().collect()
}

#[test]
fn empty_list_forbids_scopes() {
    assert_eq!(ScopeRule::new(List::default()), ScopeRule::Forbidden);
}

#[test]
fn non_empty_list_requires_scope() {
    assert_eq!(
        ScopeRule::new(list(&["core"])),
        ScopeRule::Required(list(&["core"]))
    );
}

#[test]
fn forbidden_accepts_no_scope() {
    assert_eq!(ScopeRule::Forbidden.check(None), Ok(()));
}

#[test]
fn forbidden_rejects_any_scope() {
    assert_eq!(
        ScopeRule::Forbidden.check(Some("core")),
        Err(ScopeError::Unexpected {
            found: "core".to_owned()
        })
    );
}

#[test]
fn required_accepts_listed_scope() {
    assert_eq!(
        ScopeRule::new(list(&["root", "core"])).check(Some("core")),
        Ok(())
    );
}

#[test]
fn required_rejects_missing_scope() {
    assert_eq!(
        ScopeRule::new(list(&["root", "core"])).check(None),
        Err(ScopeError::Missing {
            allowed: list(&["root", "core"])
        })
    );
}

#[test]
fn required_rejects_unlisted_scope() {
    assert_eq!(
        ScopeRule::new(list(&["root", "core"])).check(Some("parser")),
        Err(ScopeError::Unknown {
            found: "parser".to_owned(),
            allowed: list(&["root", "core"]),
        })
    );
}

#[test]
fn required_rejects_empty_scope() {
    assert_eq!(
        ScopeRule::new(list(&["core"])).check(Some("")),
        Err(ScopeError::Unknown {
            found: String::new(),
            allowed: list(&["core"]),
        })
    );
}
