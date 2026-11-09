use super::NameSet;

fn set(names: &[&str]) -> NameSet {
    names.iter().copied().collect()
}

#[test]
fn keeps_names_in_given_order() {
    assert_eq!(set(&["fix", "feat", "docs"]).to_string(), "fix, feat, docs");
}

#[test]
fn keeps_first_position_of_repeated_name() {
    assert_eq!(set(&["b", "a", "b", "c", "a"]).to_string(), "b, a, c");
}

#[test]
fn keeps_names_as_given() {
    assert_eq!(set(&[" a", "a", ""]).to_string(), " a, a, ");
}

#[test]
fn contains_only_exact_names() {
    let set = set(&["feat", "fix"]);
    assert!(set.contains("feat"));
    assert!(!set.contains("fea"));
    assert!(!set.contains("Feat"));
    assert!(!set.contains(" feat"));
    assert!(!set.contains(""));
}
