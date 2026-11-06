use super::List;

#[test]
fn keeps_values_in_written_order() {
    let list: List = ["fix", "feat", "docs"].into_iter().collect();
    assert_eq!(list.to_string(), "fix, feat, docs");
}

#[test]
fn drops_empty_values() {
    let list: List = ["", "a", "", "b", ""].into_iter().collect();
    assert_eq!(list, ["a", "b"].into_iter().collect());
}

#[test]
fn trims_surrounding_whitespace() {
    let list: List = [" a", "b ", " c d ", " ", "\t"].into_iter().collect();
    assert_eq!(list.to_string(), "a, b, c d");
}

#[test]
fn keeps_first_position_of_repeated_value() {
    let list: List = ["b", "a", "b", "c", " a "].into_iter().collect();
    assert_eq!(list.to_string(), "b, a, c");
}

#[test]
fn is_empty_when_every_value_is_empty() {
    let list: List = ["", ""].into_iter().collect();
    assert!(list.is_empty());
    assert_eq!(list.to_string(), "");
}

#[test]
fn contains_only_exact_values() {
    let list: List = ["feat", "fix"].into_iter().collect();
    assert!(list.contains("feat"));
    assert!(!list.contains("fea"));
    assert!(!list.contains("Feat"));
    assert!(!list.contains(""));
}
