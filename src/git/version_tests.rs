use super::GitVersion;

fn version(major: u32, minor: u32, patch: u32) -> GitVersion {
    GitVersion {
        major,
        minor,
        patch,
    }
}

#[test]
fn parses_release() {
    assert_eq!(
        GitVersion::parse(b"git version 2.39.5\n"),
        Some(version(2, 39, 5))
    );
}

#[test]
fn parses_vendor_builds_from_release_numbers() {
    assert_eq!(
        GitVersion::parse(b"git version 2.39.5 (Apple Git-154)\n"),
        Some(version(2, 39, 5))
    );
    assert_eq!(
        GitVersion::parse(b"git version 2.45.1.windows.1\n"),
        Some(version(2, 45, 1))
    );
}

#[test]
fn parses_release_candidates_from_release_numbers() {
    assert_eq!(
        GitVersion::parse(b"git version 2.48.0.rc1\n"),
        Some(version(2, 48, 0))
    );
}

#[test]
fn rejects_output_without_prefix() {
    assert_eq!(GitVersion::parse(b"2.39.5\n"), None);
    assert_eq!(GitVersion::parse(b"hub version 2.39.5\n"), None);
}

#[test]
fn rejects_fewer_than_three_release_numbers() {
    assert_eq!(GitVersion::parse(b"git version 2.39\n"), None);
    assert_eq!(GitVersion::parse(b"git version 2.39.x\n"), None);
    assert_eq!(GitVersion::parse(b"git version \n"), None);
}

#[test]
fn rejects_output_that_is_not_utf8() {
    assert_eq!(GitVersion::parse(b"git version 2.39.5\xff\n"), None);
}

#[test]
fn orders_by_release_numbers() {
    assert!(version(2, 9, 0) < version(2, 10, 0));
    assert!(version(2, 39, 5) < version(3, 0, 0));
    assert!(version(2, 39, 4) < version(2, 39, 5));
}
