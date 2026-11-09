use super::{DEFAULT_TYPES, resolve_list, resolve_types};
use crate::name_set::NameSet;

fn values(values: &[&str]) -> Option<Vec<String>> {
    Some(values.iter().copied().map(String::from).collect())
}

fn set(names: &[&str]) -> NameSet {
    names.iter().copied().collect()
}

#[test]
fn list_is_none_when_omitted() {
    assert_eq!(resolve_list(None), None);
}

#[test]
fn list_keeps_values_in_written_order() {
    assert_eq!(
        resolve_list(values(&["fix", "feat", "fix"])),
        Some(set(&["fix", "feat"]))
    );
}

#[test]
fn list_trims_surrounding_whitespace() {
    assert_eq!(
        resolve_list(values(&[" a", "b ", "\tc d ", "a"])),
        Some(set(&["a", "b", "c d"]))
    );
}

#[test]
fn list_drops_empty_values() {
    assert_eq!(
        resolve_list(values(&["", "a", " ", "b", "\t"])),
        Some(set(&["a", "b"]))
    );
}

#[test]
fn list_without_values_counts_as_omitted() {
    for written in [values(&[]), values(&[""]), values(&["", " ", "\t"])] {
        assert_eq!(resolve_list(written.clone()), None, "{written:?}");
    }
}

#[test]
fn types_default_when_omitted() {
    assert_eq!(resolve_types(None), set(&DEFAULT_TYPES));
}

#[test]
fn types_default_when_every_value_is_empty() {
    assert_eq!(resolve_types(values(&["", " "])), set(&DEFAULT_TYPES));
}

#[test]
fn given_types_replace_defaults() {
    assert_eq!(
        resolve_types(values(&["feat", "wip"])),
        set(&["feat", "wip"])
    );
}
