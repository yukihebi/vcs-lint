use super::{CommitLintRules, CommitLintTarget, CommitScopeRule};
use crate::{
    name_set::NameSet,
    report::commit::{
        CommitLintError, CommitScopeViolation, CommitSpacingViolation, CommitSubjectViolation,
        CommitTypeViolation,
    },
};

fn set(names: &[&str]) -> NameSet {
    names.iter().copied().collect()
}

fn types() -> NameSet {
    set(&["feat", "fix"])
}

fn scopes() -> NameSet {
    set(&["root", "core"])
}

/// Rules of a repository without scopes.
fn unscoped() -> CommitLintRules {
    CommitLintRules {
        types: types(),
        scopes: CommitScopeRule::Forbidden,
    }
}

/// Rules of a repository whose modules are its scopes.
fn scoped() -> CommitLintRules {
    CommitLintRules {
        types: types(),
        scopes: CommitScopeRule::Required(scopes()),
    }
}

fn lint(header: &str, rules: &CommitLintRules) -> Result<(), CommitLintError> {
    rules.lint(&CommitLintTarget::Header(header.to_owned()))
}

/// The parts of a header that break the rules, none unless given.
#[derive(Default)]
struct Parts {
    ty: Option<CommitTypeViolation>,
    scope: Option<CommitScopeViolation>,
    spacing: Option<CommitSpacingViolation>,
    subject: Option<CommitSubjectViolation>,
}

impl Parts {
    fn of(self, header: &str) -> Result<(), CommitLintError> {
        Err(CommitLintError::InvalidParts {
            header: header.to_owned(),
            ty: self.ty,
            scope: self.scope,
            spacing: self.spacing,
            subject: self.subject,
        })
    }
}

fn unknown_type(found: &str) -> Option<CommitTypeViolation> {
    Some(CommitTypeViolation {
        found: found.to_owned(),
        allowed: types(),
    })
}

#[test]
fn rejects_empty_message() {
    assert_eq!(
        unscoped().lint(&CommitLintTarget::Empty),
        Err(CommitLintError::Empty)
    );
}

#[test]
fn accepts_exempt_message_without_checking_it() {
    assert_eq!(unscoped().lint(&CommitLintTarget::Exempt), Ok(()));
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
    let header = "feature: add matcher";
    assert_eq!(
        lint(header, &unscoped()),
        Parts {
            ty: unknown_type("feature"),
            ..Parts::default()
        }
        .of(header)
    );
}

#[test]
fn unscoped_rejects_scope() {
    let header = "feat(core): add matcher";
    assert_eq!(
        lint(header, &unscoped()),
        Parts {
            scope: Some(CommitScopeViolation::Unexpected {
                found: "core".to_owned(),
            }),
            ..Parts::default()
        }
        .of(header)
    );
}

#[test]
fn scoped_rejects_missing_scope() {
    let header = "feat: add matcher";
    assert_eq!(
        lint(header, &scoped()),
        Parts {
            scope: Some(CommitScopeViolation::Missing { allowed: scopes() }),
            ..Parts::default()
        }
        .of(header)
    );
}

#[test]
fn scoped_rejects_unknown_scope() {
    for (header, found) in [("feat(parser): add matcher", "parser"), ("feat(): x", "")] {
        assert_eq!(
            lint(header, &scoped()),
            Parts {
                scope: Some(CommitScopeViolation::Unknown {
                    found: found.to_owned(),
                    allowed: scopes(),
                }),
                ..Parts::default()
            }
            .of(header),
            "{header}"
        );
    }
}

#[test]
fn malformed_header_shows_form_without_scope() {
    assert_eq!(
        lint("add matcher", &unscoped()),
        Err(CommitLintError::Malformed {
            header: "add matcher".to_owned(),
            expected: "<type>: <subject>",
        })
    );
}

#[test]
fn malformed_header_shows_form_with_scope() {
    assert_eq!(
        lint("add matcher", &scoped()),
        Err(CommitLintError::Malformed {
            header: "add matcher".to_owned(),
            expected: "<type>(<scope>): <subject>",
        })
    );
}

#[test]
fn reports_every_part_together() {
    let unknown_scope = || {
        Some(CommitScopeViolation::Unknown {
            found: "parser".to_owned(),
            allowed: scopes(),
        })
    };
    let header = "feature(parser):x";
    assert_eq!(
        lint(header, &scoped()),
        Parts {
            ty: unknown_type("feature"),
            scope: unknown_scope(),
            spacing: Some(CommitSpacingViolation),
            subject: None,
        }
        .of(header)
    );
    let header = "feature(parser):";
    assert_eq!(
        lint(header, &scoped()),
        Parts {
            ty: unknown_type("feature"),
            scope: unknown_scope(),
            spacing: None,
            subject: Some(CommitSubjectViolation),
        }
        .of(header)
    );
}
