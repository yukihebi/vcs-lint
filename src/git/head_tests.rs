use super::GitHead;

#[test]
fn reads_branch_name_from_full_name() {
    assert_eq!(
        GitHead::parse_branch(b"refs/heads/main\n"),
        Some("main".to_owned())
    );
    assert_eq!(
        GitHead::parse_branch(b"refs/heads/feat/add-matcher\n"),
        Some("feat/add-matcher".to_owned())
    );
}

#[test]
fn keeps_branch_name_as_is() {
    assert_eq!(
        GitHead::parse_branch(b"refs/heads/heads/Feat X\n"),
        Some("heads/Feat X".to_owned())
    );
}

#[test]
fn rejects_refs_outside_branches() {
    assert_eq!(GitHead::parse_branch(b"refs/remotes/origin/main\n"), None);
    assert_eq!(GitHead::parse_branch(b"refs/tags/v1\n"), None);
    assert_eq!(GitHead::parse_branch(b"main\n"), None);
    assert_eq!(GitHead::parse_branch(b"refs/heads/\n"), None);
}

#[test]
fn rejects_output_not_ending_with_one_line_break() {
    assert_eq!(GitHead::parse_branch(b"refs/heads/main"), None);
    assert_eq!(GitHead::parse_branch(b"refs/heads/main\n\n"), None);
    assert_eq!(GitHead::parse_branch(b"refs/heads/ma\nin\n"), None);
}

#[test]
fn rejects_output_that_is_not_utf8() {
    assert_eq!(GitHead::parse_branch(b"refs/heads/\xff\n"), None);
}
