use super::Git;

#[test]
fn reads_count_followed_by_line_break() {
    assert_eq!(Git::parse_count(b"0\n"), Some(0));
    assert_eq!(Git::parse_count(b"42\n"), Some(42));
}

#[test]
fn rejects_count_not_in_decimal_digits() {
    for stdout in [
        &b"+5\n"[..],
        b"-1\n",
        b" 5\n",
        b"\n",
        b"5",
        b"5\n\n",
        b"0x5\n",
    ] {
        assert_eq!(Git::parse_count(stdout), None, "{stdout:?}");
    }
}

#[test]
fn rejects_count_beyond_range() {
    assert_eq!(Git::parse_count(b"18446744073709551616\n"), None);
}

#[test]
fn reads_true_or_false_followed_by_line_break() {
    assert_eq!(Git::parse_bool(b"true\n"), Some(true));
    assert_eq!(Git::parse_bool(b"false\n"), Some(false));
}

#[test]
fn rejects_other_booleans() {
    for stdout in [&b"True\n"[..], b"true", b"1\n", b"yes\n", b""] {
        assert_eq!(Git::parse_bool(stdout), None, "{stdout:?}");
    }
}

#[test]
fn names_no_object_by_name_holding_line_break() {
    assert_eq!(Git.commit("HEAD\nHEAD"), Ok(None));
}
