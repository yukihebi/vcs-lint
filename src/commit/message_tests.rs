use super::Message;

#[test]
fn takes_first_line_as_header() {
    assert_eq!(
        Message::parse("feat: add matcher\n\nbody\n"),
        Message::Header("feat: add matcher")
    );
}

#[test]
fn accepts_message_without_trailing_newline() {
    assert_eq!(Message::parse("fix: x"), Message::Header("fix: x"));
}

#[test]
fn strips_crlf_line_ending() {
    assert_eq!(
        Message::parse("fix: x\r\n\r\nbody\r\n"),
        Message::Header("fix: x")
    );
}

#[test]
fn skips_leading_blank_lines() {
    assert_eq!(
        Message::parse("\n  \n\tfix: x"),
        Message::Header("\tfix: x")
    );
}

#[test]
fn skips_comment_lines() {
    let text = "# Please enter the commit message\n#\nfix: x\n# trailing comment\n";
    assert_eq!(Message::parse(text), Message::Header("fix: x"));
}

#[test]
fn keeps_line_with_hash_after_start() {
    assert_eq!(Message::parse("fix: #12"), Message::Header("fix: #12"));
}

#[test]
fn ignores_everything_after_scissors() {
    let text = "\
# ------------------------ >8 ------------------------
# Do not modify or remove the line above.
diff --git a/src/lib.rs b/src/lib.rs
";
    assert_eq!(Message::parse(text), Message::Empty);
}

#[test]
fn takes_header_before_scissors() {
    let text = "\
fix: x
# ------------------------ >8 ------------------------
diff --git a/src/lib.rs b/src/lib.rs
";
    assert_eq!(Message::parse(text), Message::Header("fix: x"));
}

#[test]
fn is_empty_without_lines() {
    assert_eq!(Message::parse(""), Message::Empty);
}

#[test]
fn is_empty_with_only_comments_and_blank_lines() {
    assert_eq!(Message::parse("\n# comment\n\n#\n"), Message::Empty);
}

#[test]
fn exempts_merge_messages() {
    for header in [
        "Merge pull request #1 from user/feat/x",
        "Merge branch 'main' into feat/x",
    ] {
        assert_eq!(Message::parse(header), Message::Exempt, "{header}");
    }
}

#[test]
fn exempts_autosquash_messages() {
    for header in ["fixup! feat: x", "squash! feat: x", "amend! feat: x"] {
        assert_eq!(Message::parse(header), Message::Exempt, "{header}");
    }
}

#[test]
fn exempts_only_exact_prefixes() {
    for header in ["Merged: x", "merge branch", "fixup: x", "fixup!x"] {
        assert_eq!(Message::parse(header), Message::Header(header), "{header}");
    }
}
