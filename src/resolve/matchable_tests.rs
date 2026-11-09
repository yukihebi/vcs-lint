use super::ListRequirement;
use crate::{
    name_set::NameSet,
    report::list::{ListName, UnmatchableList},
};

fn set(names: &[&str]) -> NameSet {
    names.iter().copied().collect()
}

const NO_SLASH: ListRequirement = ListRequirement {
    list: ListName::Types,
    is_matchable: |value| !value.contains('/'),
    wording: "no slash",
};

#[test]
fn finds_nothing_when_every_value_is_matchable() {
    assert_eq!(
        NO_SLASH.find_unmatchable(&set(&["feat", "a b", "機能"])),
        None
    );
}

#[test]
fn finds_unmatchable_values_in_order() {
    assert_eq!(
        NO_SLASH.find_unmatchable(&set(&["c/d", "feat", "a/b"])),
        Some(UnmatchableList {
            list: ListName::Types,
            values: set(&["c/d", "a/b"]),
            requirement: "no slash",
        })
    );
}

#[test]
fn finds_values_with_line_breaks_whatever_the_requirement() {
    assert_eq!(
        NO_SLASH.find_unmatchable(&set(&["a\nb", "a\rb", "feat"])),
        Some(UnmatchableList {
            list: ListName::Types,
            values: set(&["a\nb", "a\rb"]),
            requirement: "no slash",
        })
    );
}
