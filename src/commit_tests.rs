use super::{Error, header::Violation, lint};
use crate::{
    rules::{CommitRules, scopes::ScopeRule, types::TypeRule},
    violations::Violations,
};

fn rules() -> CommitRules {
    CommitRules {
        types: TypeRule::new(["feat", "fix"].into_iter().collect()),
        scopes: ScopeRule::Forbidden,
    }
}

#[test]
fn accepts_valid_header() {
    assert_eq!(lint("feat: add matcher\n\nbody\n", &rules()), Ok(()));
}

#[test]
fn reports_header_with_its_violations() {
    assert_eq!(
        lint("# comment\nfeat:\n", &rules()),
        Err(Error::Invalid {
            header: "feat:".to_owned(),
            violations: Violations(vec![Violation::EmptySubject]),
        })
    );
}

#[test]
fn rejects_empty_message() {
    assert_eq!(lint("# comment only\n", &rules()), Err(Error::Empty));
}

#[test]
fn accepts_exempt_message_without_checking_it() {
    assert_eq!(lint("fixup! feature: whatever", &rules()), Ok(()));
}
