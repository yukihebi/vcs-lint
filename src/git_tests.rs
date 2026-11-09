use super::Git;

#[test]
fn reads_version_of_installed_git() {
    let version = Git::new().version().unwrap();
    assert!(version.major >= 2);
}
