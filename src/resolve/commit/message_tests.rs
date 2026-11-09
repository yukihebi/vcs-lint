use super::CommitMessage;
use crate::lint::commit::CommitLintTarget;

fn header(line: &str) -> CommitLintTarget {
    CommitLintTarget::Header(line.to_owned())
}

#[test]
fn takes_first_line_as_header() {
    assert_eq!(
        CommitMessage("feat: add matcher\n\nbody\n").resolve(),
        header("feat: add matcher")
    );
}

#[test]
fn accepts_message_without_trailing_newline() {
    assert_eq!(CommitMessage("fix: x").resolve(), header("fix: x"));
}

#[test]
fn strips_crlf_line_ending() {
    assert_eq!(
        CommitMessage("fix: x\r\n\r\nbody\r\n").resolve(),
        header("fix: x")
    );
}

#[test]
fn skips_leading_blank_lines() {
    assert_eq!(
        CommitMessage("\n  \n\tfix: x").resolve(),
        header("\tfix: x")
    );
}

#[test]
fn skips_comment_lines() {
    let text = "# Please enter the commit message\n#\nfix: x\n# trailing comment\n";
    assert_eq!(CommitMessage(text).resolve(), header("fix: x"));
}

#[test]
fn keeps_line_with_hash_after_start() {
    assert_eq!(CommitMessage("fix: #12").resolve(), header("fix: #12"));
}

#[test]
fn ignores_everything_after_scissors() {
    let text = "\
# ------------------------ >8 ------------------------
# Do not modify or remove the line above.
diff --git a/src/lib.rs b/src/lib.rs
";
    assert_eq!(CommitMessage(text).resolve(), CommitLintTarget::Empty);
}

#[test]
fn takes_header_before_scissors() {
    let text = "\
fix: x
# ------------------------ >8 ------------------------
diff --git a/src/lib.rs b/src/lib.rs
";
    assert_eq!(CommitMessage(text).resolve(), header("fix: x"));
}

#[test]
fn is_empty_without_lines() {
    assert_eq!(CommitMessage("").resolve(), CommitLintTarget::Empty);
}

#[test]
fn is_empty_with_only_comments_and_blank_lines() {
    assert_eq!(
        CommitMessage("\n# comment\n\n#\n").resolve(),
        CommitLintTarget::Empty
    );
}

#[test]
fn exempts_merge_messages() {
    for header in [
        "Merge pull request #1 from user/feat/x",
        "Merge branch 'main' into feat/x",
    ] {
        assert_eq!(
            CommitMessage(header).resolve(),
            CommitLintTarget::Exempt,
            "{header}"
        );
    }
}

#[test]
fn exempts_autosquash_messages() {
    for header in ["fixup! feat: x", "squash! feat: x", "amend! feat: x"] {
        assert_eq!(
            CommitMessage(header).resolve(),
            CommitLintTarget::Exempt,
            "{header}"
        );
    }
}

#[test]
fn exempts_only_exact_prefixes() {
    for line in ["Merged: x", "merge branch", "fixup: x", "fixup!x"] {
        assert_eq!(CommitMessage(line).resolve(), header(line), "{line}");
    }
}
