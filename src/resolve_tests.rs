use super::{DEFAULT_TYPES, resolve_types};
use crate::name_set::NameSet;

fn values(values: &[&str]) -> Option<Vec<String>> {
    Some(values.iter().copied().map(String::from).collect())
}

fn set(names: &[&str]) -> NameSet {
    names.iter().copied().collect()
}

#[test]
fn types_default_when_omitted() {
    assert_eq!(resolve_types(None), set(&DEFAULT_TYPES));
}

#[test]
fn given_types_replace_defaults() {
    assert_eq!(
        resolve_types(values(&["feat", "wip", "feat"])),
        set(&["feat", "wip"])
    );
}
