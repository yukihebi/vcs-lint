use super::HistoryRawConfig;
use crate::{
    cli::{NetworkClapAccess, history::HistoryBaseClapSource},
    git::Git,
    lint::history::{HistoryBase, HistoryBaseSource},
    report::history::HistoryResolveError,
};

fn source(
    base_from: Option<HistoryBaseClapSource>,
    remote: Option<&str>,
    network: Option<NetworkClapAccess>,
) -> HistoryBaseSource {
    HistoryRawConfig::resolve_base(
        Some("main".to_owned()),
        base_from,
        remote.map(String::from),
        network,
    )
    .unwrap()
    .source
}

fn remote(remote: &str) -> HistoryBaseSource {
    HistoryBaseSource::Remote {
        remote: remote.to_owned(),
    }
}

fn remote_tracking(remote: &str) -> HistoryBaseSource {
    HistoryBaseSource::RemoteTracking {
        remote: remote.to_owned(),
    }
}

#[test]
fn reads_base_from_origin_over_network_by_default() {
    assert_eq!(source(None, None, None), remote("origin"));
}

#[test]
fn reads_base_from_given_remote() {
    assert_eq!(source(None, Some("upstream"), None), remote("upstream"));
    assert_eq!(
        source(
            Some(HistoryBaseClapSource::Remote),
            Some("upstream"),
            Some(NetworkClapAccess::Allow)
        ),
        remote("upstream")
    );
}

#[test]
fn reads_remote_tracking_branch_without_network() {
    assert_eq!(
        source(None, None, Some(NetworkClapAccess::Deny)),
        remote_tracking("origin")
    );
    assert_eq!(
        source(None, Some("upstream"), Some(NetworkClapAccess::Deny)),
        remote_tracking("upstream")
    );
}

#[test]
fn reads_local_branch_whatever_remote_and_network() {
    for network in [
        None,
        Some(NetworkClapAccess::Allow),
        Some(NetworkClapAccess::Deny),
    ] {
        assert_eq!(
            source(
                Some(HistoryBaseClapSource::Local),
                Some("upstream"),
                network
            ),
            HistoryBaseSource::Local,
            "{network:?}"
        );
    }
}

#[test]
fn fails_without_base_before_reading_git() {
    let config = HistoryRawConfig {
        rev: Some("no such revision".to_owned()),
        base: None,
        base_from: None,
        remote: None,
        network: None,
    };
    assert_eq!(config.resolve(&Git), Err(HistoryResolveError::MissingBase));
}

#[test]
fn fails_on_base_with_revision_syntax_before_reading_revision() {
    let config = HistoryRawConfig {
        rev: Some("no such revision".to_owned()),
        base: Some("main~1".to_owned()),
        base_from: Some(HistoryBaseClapSource::Local),
        remote: None,
        network: None,
    };
    assert_eq!(
        config.resolve(&Git),
        Err(HistoryResolveError::InvalidBase {
            base: HistoryBase {
                branch: "main~1".to_owned(),
                source: HistoryBaseSource::Local,
            },
            ref_name: "refs/heads/main~1".to_owned(),
        })
    );
}
