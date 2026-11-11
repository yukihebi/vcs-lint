use super::{HistoryBase, HistoryBaseSource, HistoryLintRules, HistoryLintTarget, HistoryTip};
use crate::{git::object_id::GitObjectId, report::history::HistoryLintError};

const TIP: &str = "0123456789abcdef0123456789abcdef01234567";

fn base(source: HistoryBaseSource) -> HistoryBase {
    HistoryBase {
        branch: "main".to_owned(),
        source,
    }
}

fn remote() -> HistoryBase {
    base(HistoryBaseSource::Remote {
        remote: "origin".to_owned(),
    })
}

fn tip_id() -> GitObjectId {
    GitObjectId::parse(TIP.as_bytes()).unwrap()
}

fn lint(tip: HistoryTip) -> Result<(), HistoryLintError> {
    HistoryLintRules { base: remote() }.lint(&HistoryLintTarget {
        rev: "feat/x".to_owned(),
        tip,
    })
}

#[test]
fn accepts_history_missing_nothing_of_base() {
    assert_eq!(lint(HistoryTip::Local { missing: 0 }), Ok(()));
}

#[test]
fn rejects_history_missing_commits_of_base() {
    assert_eq!(
        lint(HistoryTip::Local { missing: 3 }),
        Err(HistoryLintError::Behind {
            rev: "feat/x".to_owned(),
            base: remote(),
            missing: 3,
        })
    );
}

#[test]
fn rejects_tip_not_in_repository() {
    assert_eq!(
        lint(HistoryTip::NotLocal { id: tip_id() }),
        Err(HistoryLintError::TipNotLocal {
            rev: "feat/x".to_owned(),
            base: remote(),
            id: tip_id(),
        })
    );
}

#[test]
fn reads_branch_by_full_name_where_its_tip_is_read() {
    assert_eq!(base(HistoryBaseSource::Local).ref_name(), "refs/heads/main");
    assert_eq!(remote().ref_name(), "refs/heads/main");
    assert_eq!(
        base(HistoryBaseSource::RemoteTracking {
            remote: "upstream".to_owned(),
        })
        .ref_name(),
        "refs/remotes/upstream/main"
    );
}
