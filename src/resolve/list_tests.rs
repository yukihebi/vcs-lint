use super::ListRawValues;
use crate::{
    name_set::NameSet,
    report::list::{ListName, ListResolveError},
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

#[test]
fn keeps_defaults_without_given_values() {
    assert_eq!(
        raw(None, &[]).resolve(ListName::Types, &["feat", "fix"]),
        Ok(set(&["feat", "fix"]))
    );
}

#[test]
fn adds_given_values_after_defaults() {
    assert_eq!(
        raw(Some(&["wip", "fix", "wip"]), &[]).resolve(ListName::Types, &["feat", "fix"]),
        Ok(set(&["feat", "fix", "wip"]))
    );
}

#[test]
fn excludes_values_from_defaults_and_given_values() {
    assert_eq!(
        raw(Some(&["wip", "tmp"]), &["fix", "tmp"]).resolve(ListName::Types, &["feat", "fix"]),
        Ok(set(&["feat", "wip"]))
    );
}

#[test]
fn excludes_every_value() {
    assert_eq!(
        raw(Some(&["main"]), &["main"]).resolve(ListName::Protected, &[]),
        Ok(set(&[]))
    );
}

#[test]
fn fails_on_excluding_values_outside_list() {
    assert_eq!(
        raw(Some(&["core"]), &["cli", "core", "root", "cli"]).resolve(ListName::Scopes, &[]),
        Err(ListResolveError::UnknownExclusions {
            list: ListName::Scopes,
            unknown: set(&["cli", "root"]),
            listed: set(&["core"]),
        })
    );
}

#[test]
fn fails_on_excluding_value_from_empty_list() {
    assert_eq!(
        raw(None, &["main"]).resolve(ListName::Protected, &[]),
        Err(ListResolveError::UnknownExclusions {
            list: ListName::Protected,
            unknown: set(&["main"]),
            listed: set(&[]),
        })
    );
}
