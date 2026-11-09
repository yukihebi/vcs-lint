use super::CommitHeader;
use crate::report::commit::{CommitSpacingViolation, CommitSubjectViolation};

fn header(line: &str) -> CommitHeader<'_> {
    CommitHeader::parse(line).unwrap()
}

#[test]
fn splits_header_without_scope() {
    assert_eq!(
        CommitHeader::parse("feat: add matcher"),
        Some(CommitHeader {
            ty: "feat",
            scope: None,
            spacing: " ",
            subject: "add matcher",
        })
    );
}

#[test]
fn splits_header_with_scope() {
    assert_eq!(
        CommitHeader::parse("feat(core): add matcher"),
        Some(CommitHeader {
            ty: "feat",
            scope: Some("core"),
            spacing: " ",
            subject: "add matcher",
        })
    );
}

#[test]
fn closes_scope_at_parenthesis_matching_its_opening() {
    for (line, scope) in [
        ("feat(core(unix)): x", "core(unix)"),
        ("feat(a():b): x", "a():b"),
        ("feat((a)(b)): x", "(a)(b)"),
    ] {
        assert_eq!(
            CommitHeader::parse(line).unwrap().scope,
            Some(scope),
            "{line}"
        );
    }
}

#[test]
fn keeps_empty_parts() {
    assert_eq!(
        CommitHeader::parse("():"),
        Some(CommitHeader {
            ty: "",
            scope: Some(""),
            spacing: "",
            subject: "",
        })
    );
}

#[test]
fn takes_all_whitespace_after_colon_as_spacing() {
    assert_eq!(
        CommitHeader::parse("fix:  \tx: y"),
        Some(CommitHeader {
            ty: "fix",
            scope: None,
            spacing: "  \t",
            subject: "x: y",
        })
    );
}

#[test]
fn rejects_header_without_colon_after_type_and_scope() {
    for line in [
        "add matcher",
        "feat add: matcher",
        "feat (core): x",
        "feat(core) : x",
        "feat(core: x",
        "feat((core): x",
        "feat(core)): x",
        "feat(core)(x): y",
        "feat!: x",
        "feat(core)!: x",
    ] {
        assert_eq!(CommitHeader::parse(line), None, "{line}");
    }
}

#[test]
fn accepts_one_space_before_subject() {
    assert_eq!(header("feat: x").lint_spacing(), None);
    assert_eq!(header("feat: x").lint_subject(), None);
}

#[test]
fn rejects_other_spacing_before_subject() {
    for line in ["feat:x", "feat:  x", "feat:\tx"] {
        assert_eq!(
            header(line).lint_spacing(),
            Some(CommitSpacingViolation),
            "{line}"
        );
    }
}

#[test]
fn rejects_empty_subject_without_judging_spacing() {
    for line in ["feat:", "feat: ", "feat:   "] {
        let header = header(line);
        assert_eq!(
            header.lint_subject(),
            Some(CommitSubjectViolation),
            "{line}"
        );
        assert_eq!(header.lint_spacing(), None, "{line}");
    }
}

#[test]
fn matchable_type_is_parsed_back_as_type() {
    for (ty, matchable) in [
        ("feat", true),
        ("Feat-2_x.y", true),
        ("機能", true),
        ("#x", true),
        ("wip!", false),
        ("a:b", false),
        ("a(b", false),
        ("a)b", false),
        ("a b", false),
        ("a\u{3000}b", false),
        ("a\nb", false),
    ] {
        assert_eq!(CommitHeader::is_matchable_type(ty), matchable, "{ty:?}");
        let line = format!("{ty}: x");
        let parsed = CommitHeader::parse(&line).map(|header| header.ty);
        assert_eq!(parsed == Some(ty), matchable, "{ty:?}");
    }
}

#[test]
fn matchable_scope_is_parsed_back_as_scope() {
    for (scope, matchable) in [
        ("core", true),
        ("a b: c!", true),
        ("core(unix)", true),
        ("(a)(b)", true),
        ("a():b", true),
        ("core(unix", false),
        ("core)", false),
        (")(", false),
        ("a)(b", false),
    ] {
        assert_eq!(
            CommitHeader::is_matchable_scope(scope),
            matchable,
            "{scope:?}"
        );
        let line = format!("feat({scope}): x");
        let parsed = CommitHeader::parse(&line).and_then(|header| header.scope);
        assert_eq!(parsed == Some(scope), matchable, "{scope:?}");
    }
}
