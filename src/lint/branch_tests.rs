use super::{BranchDetachedRule, BranchLintRules, BranchLintTarget};
use crate::{
    name_set::NameSet,
    report::branch::{BranchLintError, BranchNameViolation, BranchTypeViolation},
};

fn types() -> NameSet {
    ["feat", "fix"].into_iter().collect()
}

fn rules(detached: BranchDetachedRule) -> BranchLintRules {
    BranchLintRules {
        types: types(),
        protected: ["main", "HEAD"].into_iter().collect(),
        detached,
    }
}

fn lint(branch: &str) -> Result<(), BranchLintError> {
    rules(BranchDetachedRule::Allowed).lint(&BranchLintTarget::Branch(branch.to_owned()))
}

fn lint_detached(detached: BranchDetachedRule) -> Result<(), BranchLintError> {
    rules(detached).lint(&BranchLintTarget::Detached)
}

fn invalid_parts(
    branch: &str,
    ty: Option<BranchTypeViolation>,
    name: Option<BranchNameViolation>,
) -> Result<(), BranchLintError> {
    Err(BranchLintError::InvalidParts {
        branch: branch.to_owned(),
        ty,
        name,
    })
}

fn unknown_type(found: &str) -> Option<BranchTypeViolation> {
    Some(BranchTypeViolation {
        found: found.to_owned(),
        allowed: types(),
    })
}

fn invalid_name(found: &str) -> Option<BranchNameViolation> {
    Some(BranchNameViolation {
        found: found.to_owned(),
    })
}

#[test]
fn accepts_type_and_kebab_case_name() {
    for branch in [
        "feat/add-matcher",
        "fix/a",
        "feat/v2",
        "feat/parse-v2-header",
    ] {
        assert_eq!(lint(branch), Ok(()), "{branch}");
    }
}

#[test]
fn rejects_protected_branch_without_checking_format() {
    assert_eq!(
        lint("main"),
        Err(BranchLintError::Protected {
            branch: "main".to_owned()
        })
    );
}

#[test]
fn accepts_protected_name_as_name_part() {
    assert_eq!(lint("fix/main"), Ok(()));
}

#[test]
fn rejects_branch_without_slash() {
    assert_eq!(
        lint("add-matcher"),
        Err(BranchLintError::Malformed {
            branch: "add-matcher".to_owned(),
            allowed: types(),
        })
    );
}

#[test]
fn rejects_unknown_type() {
    assert_eq!(
        lint("feature/add-matcher"),
        invalid_parts("feature/add-matcher", unknown_type("feature"), None)
    );
}

#[test]
fn rejects_empty_type() {
    assert_eq!(
        lint("/add-matcher"),
        invalid_parts("/add-matcher", unknown_type(""), None)
    );
}

#[test]
fn rejects_name_outside_kebab_case() {
    for name in [
        "",
        "Add-matcher",
        "add_matcher",
        "addMatcher",
        "2-add",
        "-add",
        "add-",
        "add--matcher",
        "add/matcher",
        "add matcher",
    ] {
        let branch = format!("feat/{name}");
        assert_eq!(
            lint(&branch),
            invalid_parts(&branch, None, invalid_name(name)),
            "{branch}"
        );
    }
}

#[test]
fn reports_type_and_name_together() {
    assert_eq!(
        lint("feature/Add_Matcher"),
        invalid_parts(
            "feature/Add_Matcher",
            unknown_type("feature"),
            invalid_name("Add_Matcher")
        )
    );
}

#[test]
fn accepts_allowed_detached_head_without_checking_protection_or_format() {
    assert_eq!(lint_detached(BranchDetachedRule::Allowed), Ok(()));
}

#[test]
fn rejects_forbidden_detached_head() {
    assert_eq!(
        lint_detached(BranchDetachedRule::Forbidden),
        Err(BranchLintError::Detached)
    );
}
