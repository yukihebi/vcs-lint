use super::{DEFAULT_TYPES, list::ListRawValues, resolve_types};
use crate::{
    cli::TypeDefaultsClapMode,
    git::Git,
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
        dirs: None,
        excluded: strings(excluded),
    }
}

fn defaults_without(excluded: &[&str]) -> NameSet {
    DEFAULT_TYPES
        .into_iter()
        .filter(|ty| !excluded.contains(ty))
        .collect()
}

#[test]
fn auto_includes_defaults_only_without_given_types() {
    for mode in [None, Some(TypeDefaultsClapMode::Auto)] {
        assert_eq!(
            resolve_types(raw(None, &[]), mode, &Git),
            Ok(set(&DEFAULT_TYPES)),
            "{mode:?}"
        );
        assert_eq!(
            resolve_types(raw(Some(&["feat", "wip", "feat"]), &[]), mode, &Git),
            Ok(set(&["feat", "wip"])),
            "{mode:?}"
        );
    }
}

#[test]
fn auto_keeps_defaults_when_only_excluding() {
    assert_eq!(
        resolve_types(raw(None, &["chore", "style"]), None, &Git),
        Ok(defaults_without(&["chore", "style"]))
    );
}

#[test]
fn always_adds_given_types_to_defaults() {
    let mut expected: Vec<&str> = DEFAULT_TYPES
        .into_iter()
        .filter(|ty| *ty != "chore")
        .collect();
    expected.push("wip");
    assert_eq!(
        resolve_types(
            raw(Some(&["wip", "feat"]), &["chore"]),
            Some(TypeDefaultsClapMode::Always),
            &Git
        ),
        Ok(set(&expected))
    );
}

#[test]
fn never_leaves_only_given_types() {
    assert_eq!(
        resolve_types(
            raw(Some(&["wip"]), &[]),
            Some(TypeDefaultsClapMode::Never),
            &Git
        ),
        Ok(set(&["wip"]))
    );
}

#[test]
fn never_rejects_excluding_a_default() {
    assert_eq!(
        resolve_types(
            raw(Some(&["wip"]), &["feat"]),
            Some(TypeDefaultsClapMode::Never),
            &Git
        ),
        Err(ListResolveError::UnknownExclusions {
            list: ListName::Types,
            unknown: set(&["feat"]),
            listed: set(&["wip"]),
        })
    );
}

#[test]
fn fails_when_no_type_is_left() {
    for (raw, mode) in [
        (raw(None, &[]), Some(TypeDefaultsClapMode::Never)),
        (raw(Some(&["wip"]), &["wip"]), None),
        (raw(None, &DEFAULT_TYPES), None),
    ] {
        assert_eq!(
            resolve_types(raw, mode, &Git),
            Err(ListResolveError::Empty {
                list: ListName::Types
            }),
            "{mode:?}"
        );
    }
}
