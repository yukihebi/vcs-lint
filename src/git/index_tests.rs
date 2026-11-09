use std::path::Path;

use super::GitIndexDir;

const BLOB: &str = "100644 e69de29bb2d1d6434b8b29ae775ad8c2e48c5391";
const GITLINK: &str = "160000 4b825dc642cb6eb9a060e54bf8d69288fbee4904";

/// The output of `git ls-files -z -s` listing `paths`, each with `meta`.
fn listing(entries: &[(&str, &str)]) -> Vec<u8> {
    entries
        .iter()
        .flat_map(|(meta, path)| format!("{meta} 0\t{path}\0").into_bytes())
        .collect()
}

fn subdirs(dir: &str, stdout: &[u8]) -> Option<Vec<String>> {
    let names = GitIndexDir::new(Path::new(dir)).parse_subdirs(stdout)?;
    Some(
        names
            .into_iter()
            .map(|name| String::from_utf8(name.to_vec()).unwrap())
            .collect(),
    )
}

#[test]
fn names_directories_holding_entries_once_in_byte_order() {
    let stdout = listing(&[
        (BLOB, "crates/a.b/Cargo.toml"),
        (BLOB, "crates/a/Cargo.toml"),
        (BLOB, "crates/a/src/lib.rs"),
        (BLOB, "crates/B/Cargo.toml"),
    ]);
    assert_eq!(
        subdirs("crates", &stdout),
        Some(vec!["B".to_owned(), "a".to_owned(), "a.b".to_owned()])
    );
}

#[test]
fn skips_files_directly_under_directory() {
    let stdout = listing(&[(BLOB, "crates/README.md"), (BLOB, "crates/a/lib.rs")]);
    assert_eq!(subdirs("crates", &stdout), Some(vec!["a".to_owned()]));
}

#[test]
fn names_submodules_directly_under_directory() {
    let stdout = listing(&[(GITLINK, "crates/sub"), (BLOB, "crates/a/lib.rs")]);
    assert_eq!(
        subdirs("crates", &stdout),
        Some(vec!["a".to_owned(), "sub".to_owned()])
    );
}

#[test]
fn names_directory_holding_deeper_submodule() {
    let stdout = listing(&[(GITLINK, "vendor/x/sub")]);
    assert_eq!(subdirs("vendor", &stdout), Some(vec!["x".to_owned()]));
}

#[test]
fn names_nothing_for_submodule_listed_as_directory_itself() {
    let stdout = listing(&[(GITLINK, "crates/sub")]);
    assert_eq!(subdirs("crates/sub", &stdout), Some(vec![]));
}

#[test]
fn names_entries_of_conflicting_stages_once() {
    let stdout = [
        format!("{BLOB} 1\tcrates/a/lib.rs\0"),
        format!("{BLOB} 2\tcrates/a/lib.rs\0"),
        format!("{BLOB} 3\tcrates/a/lib.rs\0"),
    ]
    .concat();
    assert_eq!(
        subdirs("crates", stdout.as_bytes()),
        Some(vec!["a".to_owned()])
    );
}

#[test]
fn names_directories_at_top_of_work_tree() {
    let stdout = listing(&[(BLOB, ".github/ci.yml"), (BLOB, "Cargo.toml")]);
    assert_eq!(subdirs("", &stdout), Some(vec![".github".to_owned()]));
}

#[test]
fn names_nothing_without_entries() {
    assert_eq!(subdirs("crates", b""), Some(vec![]));
}

/// The index may write the directory otherwise than its path on disk, as
/// with the composed `é` git keeps on macOS for a decomposed one on disk, or
/// a case changed on a case-insensitive file system.
#[test]
fn takes_entries_as_written_in_index() {
    let stdout = listing(&[(BLOB, "caf\u{e9}/p/x")]);
    assert_eq!(subdirs("cafe\u{301}", &stdout), Some(vec!["p".to_owned()]));
    let stdout = listing(&[(BLOB, "Crates/a/x")]);
    assert_eq!(subdirs("crates", &stdout), Some(vec!["a".to_owned()]));
}

#[test]
fn rejects_entry_above_directory() {
    let stdout = listing(&[(BLOB, "crates/a/lib.rs"), (BLOB, "Cargo.toml")]);
    assert_eq!(subdirs("crates/a", &stdout), None);
}

#[test]
fn rejects_entry_without_mode_and_path() {
    assert_eq!(subdirs("crates", b"crates/a/lib.rs\0"), None);
    assert_eq!(subdirs("crates", format!("{BLOB} 0\t\0").as_bytes()), None);
}

#[test]
fn rejects_output_not_ending_with_nul() {
    let mut stdout = listing(&[(BLOB, "crates/a/lib.rs")]);
    stdout.pop();
    assert_eq!(subdirs("crates", &stdout), None);
}
