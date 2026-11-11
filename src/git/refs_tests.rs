use super::GitListedRef;
use crate::git::object_id::GitObjectId;

const MAIN: &str = "0123456789abcdef0123456789abcdef01234567";
const OTHER: &str = "89abcdef0123456789abcdef0123456789abcdef";

fn id(name: &str) -> GitObjectId {
    GitObjectId::parse(name.as_bytes()).unwrap()
}

fn find(stdout: &str, name: &str) -> Option<Option<GitObjectId>> {
    GitListedRef::find(stdout.as_bytes(), name)
}

#[test]
fn finds_ref_of_exact_name() {
    let stdout = format!("{MAIN}\trefs/heads/main\n");
    assert_eq!(find(&stdout, "refs/heads/main"), Some(Some(id(MAIN))));
}

#[test]
fn passes_over_refs_of_other_names() {
    let stdout = format!(
        "{OTHER}\trefs/heads/fix/refs/heads/main\n{MAIN}\trefs/heads/main\n{OTHER}\trefs/heads/main/x\n"
    );
    assert_eq!(find(&stdout, "refs/heads/main"), Some(Some(id(MAIN))));
    let stdout = format!("{OTHER}\trefs/heads/fix/refs/heads/main\n");
    assert_eq!(find(&stdout, "refs/heads/main"), Some(None));
}

#[test]
fn finds_nothing_in_empty_list() {
    assert_eq!(find("", "refs/heads/main"), Some(None));
}

#[test]
fn rejects_output_not_in_listed_form() {
    for stdout in [
        format!("{MAIN}\trefs/heads/main"),
        format!("{MAIN} refs/heads/main\n"),
        format!("{MAIN}\t\n"),
        "abc\trefs/heads/main\n".to_owned(),
        format!("{MAIN}\trefs/heads/main\n\n"),
    ] {
        assert_eq!(find(&stdout, "refs/heads/main"), None, "{stdout:?}");
    }
}
