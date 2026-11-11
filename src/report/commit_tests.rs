use super::{
    CommitLintError, CommitScopeViolation, CommitSpacingViolation, CommitSubjectViolation,
    CommitTypeViolation,
};
use crate::{lint::commit::CommitRule, name_set::NameSet};

/// A header whose parts break the rules where `broken` names them, among
/// `type`, `scope`, `spacing` and `subject`.
fn invalid_parts(broken: &[&str]) -> CommitLintError {
    CommitLintError::InvalidParts {
        header: "feature(x):y".to_owned(),
        ty: broken.contains(&"type").then(|| CommitTypeViolation {
            found: "feature".to_owned(),
            allowed: NameSet::default(),
        }),
        scope: broken
            .contains(&"scope")
            .then(|| CommitScopeViolation::Unexpected {
                found: "x".to_owned(),
            }),
        spacing: broken
            .contains(&"spacing")
            .then_some(CommitSpacingViolation),
        subject: broken
            .contains(&"subject")
            .then_some(CommitSubjectViolation),
    }
}

#[test]
fn breaks_format_rule_when_empty_or_malformed() {
    assert_eq!(CommitLintError::Empty.broken_rules(), [CommitRule::Format]);
    let malformed = CommitLintError::Malformed {
        header: "add".to_owned(),
        expected: "<type>: <subject>",
    };
    assert_eq!(malformed.broken_rules(), [CommitRule::Format]);
}

#[test]
fn breaks_format_rule_by_spacing_or_subject() {
    assert_eq!(
        invalid_parts(&["spacing"]).broken_rules(),
        [CommitRule::Format]
    );
    assert_eq!(
        invalid_parts(&["subject"]).broken_rules(),
        [CommitRule::Format]
    );
}

#[test]
fn breaks_rule_of_each_invalid_part_in_report_order() {
    assert_eq!(invalid_parts(&["type"]).broken_rules(), [CommitRule::Type]);
    assert_eq!(
        invalid_parts(&["scope"]).broken_rules(),
        [CommitRule::Scope]
    );
    assert_eq!(
        invalid_parts(&["type", "scope", "spacing"]).broken_rules(),
        [CommitRule::Type, CommitRule::Scope, CommitRule::Format]
    );
}
