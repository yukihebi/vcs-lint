use super::{Protected, ProtectedRule};

fn rule(branches: &[&str]) -> ProtectedRule {
    ProtectedRule::new(branches.iter().copied().collect())
}

#[test]
fn rejects_protected_branch() {
    assert_eq!(
        rule(&["main", "develop"]).check("develop"),
        Err(Protected {
            branch: "develop".to_owned()
        })
    );
}

#[test]
fn accepts_other_branch() {
    assert_eq!(rule(&["main"]).check("feat/main"), Ok(()));
}

#[test]
fn protects_nothing_by_default() {
    assert_eq!(ProtectedRule::default().check("main"), Ok(()));
}
