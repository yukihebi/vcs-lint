use super::{Error, Violation, lint};
use crate::{
    rules::{
        BranchRules,
        list::List,
        protected::{Protected, ProtectedRule},
        types::{TypeRule, UnknownType},
    },
    violations::Violations,
};

fn types() -> List {
    ["feat", "fix"].into_iter().collect()
}

fn rules() -> BranchRules {
    BranchRules {
        types: TypeRule::new(types()),
        protected: ProtectedRule::new(["main"].into_iter().collect()),
    }
}

fn invalid(branch: &str, violations: Vec<Violation>) -> Result<(), Error> {
    Err(Error::Invalid {
        branch: branch.to_owned(),
        violations: Violations(violations),
    })
}

fn unknown_type(found: &str) -> Violation {
    Violation::Type(UnknownType {
        found: found.to_owned(),
        allowed: types(),
    })
}

fn invalid_name(name: &str) -> Violation {
    Violation::Name {
        name: name.to_owned(),
    }
}

#[test]
fn accepts_type_and_kebab_case_name() {
    for branch in ["feat/add-matcher", "fix/a", "feat/v2", "feat/parse-v2-header"] {
        assert_eq!(lint(branch, &rules()), Ok(()), "{branch}");
    }
}

#[test]
fn rejects_protected_branch() {
    assert_eq!(
        lint("main", &rules()),
        Err(Error::Protected(Protected {
            branch: "main".to_owned()
        }))
    );
}

#[test]
fn accepts_protected_name_as_name_part() {
    assert_eq!(lint("fix/main", &rules()), Ok(()));
}

#[test]
fn rejects_missing_type_prefix() {
    assert_eq!(
        lint("add-matcher", &rules()),
        invalid(
            "add-matcher",
            vec![Violation::MissingType { allowed: types() }]
        )
    );
}

#[test]
fn rejects_unknown_type() {
    assert_eq!(
        lint("feature/add-matcher", &rules()),
        invalid("feature/add-matcher", vec![unknown_type("feature")])
    );
}

#[test]
fn rejects_empty_type() {
    assert_eq!(
        lint("/add-matcher", &rules()),
        invalid("/add-matcher", vec![unknown_type("")])
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
            lint(&branch, &rules()),
            invalid(&branch, vec![invalid_name(name)]),
            "{branch}"
        );
    }
}

#[test]
fn reports_type_and_name_together() {
    assert_eq!(
        lint("feature/Add_Matcher", &rules()),
        invalid(
            "feature/Add_Matcher",
            vec![unknown_type("feature"), invalid_name("Add_Matcher")]
        )
    );
}
