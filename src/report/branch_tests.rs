use super::{BranchLintError, BranchNameViolation, BranchTypeViolation};
use crate::{lint::branch::BranchRule, name_set::NameSet};

fn invalid_parts(ty: bool, name: bool) -> BranchLintError {
    BranchLintError::InvalidParts {
        branch: "feature/Add".to_owned(),
        ty: ty.then(|| BranchTypeViolation {
            found: "feature".to_owned(),
            allowed: NameSet::default(),
        }),
        name: name.then(|| BranchNameViolation {
            found: "Add".to_owned(),
        }),
    }
}

#[test]
fn breaks_rule_of_detached_head_or_protected_branch_alone() {
    assert_eq!(
        BranchLintError::Detached.broken_rules(),
        [BranchRule::Detached]
    );
    let protected = BranchLintError::Protected {
        branch: "main".to_owned(),
    };
    assert_eq!(protected.broken_rules(), [BranchRule::Protected]);
}

#[test]
fn breaks_type_rule_without_type_prefix() {
    let malformed = BranchLintError::Malformed {
        branch: "add".to_owned(),
        allowed: NameSet::default(),
    };
    assert_eq!(malformed.broken_rules(), [BranchRule::Type]);
}

#[test]
fn breaks_rule_of_each_invalid_part_in_report_order() {
    assert_eq!(
        invalid_parts(true, false).broken_rules(),
        [BranchRule::Type]
    );
    assert_eq!(
        invalid_parts(false, true).broken_rules(),
        [BranchRule::Format]
    );
    assert_eq!(
        invalid_parts(true, true).broken_rules(),
        [BranchRule::Type, BranchRule::Format]
    );
}
