use super::{Header, Violation, lint};
use crate::{
    rules::{
        CommitRules,
        list::List,
        scopes::{ScopeError, ScopeRule},
        types::{TypeRule, UnknownType},
    },
    violations::Violations,
};

fn list(values: &[&str]) -> List {
    values.iter().copied().collect()
}

fn types() -> List {
    list(&["feat", "fix"])
}

fn scopes() -> List {
    list(&["root", "core"])
}

/// Rules of a repository without scopes.
fn unscoped() -> CommitRules {
    CommitRules {
        types: TypeRule::new(types()),
        scopes: ScopeRule::Forbidden,
    }
}

/// Rules of a repository whose modules are its scopes.
fn scoped() -> CommitRules {
    CommitRules {
        types: TypeRule::new(types()),
        scopes: ScopeRule::new(scopes()),
    }
}

fn violations(violations: Vec<Violation>) -> Result<(), Violations<Violation>> {
    Violations::collect(violations.into_iter().map(Err))
}

fn unknown_type(found: &str) -> Violation {
    Violation::Type(UnknownType {
        found: found.to_owned(),
        allowed: types(),
    })
}

mod parse {
    use super::Header;

    #[test]
    fn splits_header_without_scope() {
        assert_eq!(
            Header::parse("feat: add matcher"),
            Some(Header {
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
            Header::parse("feat(core): add matcher"),
            Some(Header {
                ty: "feat",
                scope: Some("core"),
                spacing: " ",
                subject: "add matcher",
            })
        );
    }

    #[test]
    fn keeps_empty_parts() {
        assert_eq!(
            Header::parse("():"),
            Some(Header {
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
            Header::parse("fix:  \tx: y"),
            Some(Header {
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
            "feat(core)(x): y",
            "feat!: x",
            "feat(core)!: x",
        ] {
            assert_eq!(Header::parse(line), None, "{line}");
        }
    }
}

#[test]
fn unscoped_accepts_type_and_subject() {
    assert_eq!(lint("feat: add matcher", &unscoped()), Ok(()));
}

#[test]
fn scoped_accepts_type_scope_and_subject() {
    assert_eq!(lint("fix(core): handle empty input", &scoped()), Ok(()));
}

#[test]
fn rejects_unknown_type() {
    assert_eq!(
        lint("feature: add matcher", &unscoped()),
        violations(vec![unknown_type("feature")])
    );
}

#[test]
fn unscoped_rejects_scope() {
    assert_eq!(
        lint("feat(core): add matcher", &unscoped()),
        violations(vec![Violation::Scope(ScopeError::Unexpected {
            found: "core".to_owned(),
        })])
    );
}

#[test]
fn scoped_rejects_missing_scope() {
    assert_eq!(
        lint("feat: add matcher", &scoped()),
        violations(vec![Violation::Scope(ScopeError::Missing {
            allowed: scopes(),
        })])
    );
}

#[test]
fn scoped_rejects_unknown_scope() {
    assert_eq!(
        lint("feat(parser): add matcher", &scoped()),
        violations(vec![Violation::Scope(ScopeError::Unknown {
            found: "parser".to_owned(),
            allowed: scopes(),
        })])
    );
}

#[test]
fn rejects_missing_space_after_colon() {
    assert_eq!(
        lint("feat:add matcher", &unscoped()),
        violations(vec![Violation::Spacing])
    );
}

#[test]
fn rejects_extra_space_after_colon() {
    for header in ["feat:  add matcher", "feat:\tadd matcher"] {
        assert_eq!(
            lint(header, &unscoped()),
            violations(vec![Violation::Spacing]),
            "{header}"
        );
    }
}

#[test]
fn rejects_empty_subject() {
    for header in ["feat:", "feat: ", "feat:   "] {
        assert_eq!(
            lint(header, &unscoped()),
            violations(vec![Violation::EmptySubject]),
            "{header}"
        );
    }
}

#[test]
fn malformed_header_shows_format_without_scope() {
    assert_eq!(
        lint("add matcher", &unscoped()),
        violations(vec![Violation::Malformed {
            expected: "<type>: <subject>",
        }])
    );
}

#[test]
fn malformed_header_shows_format_with_scope() {
    assert_eq!(
        lint("add matcher", &scoped()),
        violations(vec![Violation::Malformed {
            expected: "<type>(<scope>): <subject>",
        }])
    );
}

#[test]
fn reports_every_violation_together() {
    assert_eq!(
        lint("feature(parser):", &scoped()),
        violations(vec![
            unknown_type("feature"),
            Violation::Scope(ScopeError::Unknown {
                found: "parser".to_owned(),
                allowed: scopes(),
            }),
            Violation::EmptySubject,
        ])
    );
}
