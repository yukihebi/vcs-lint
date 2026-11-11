use super::GitObjectId;

const SHA1: &str = "0123456789abcdef0123456789abcdef01234567";
const SHA256: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

#[test]
fn reads_sha1_and_sha256_names() {
    for name in [SHA1, SHA256] {
        assert_eq!(
            GitObjectId::parse(name.as_bytes()).map(|id| id.to_string()),
            Some(name.to_owned())
        );
    }
}

#[test]
fn rejects_abbreviated_or_overlong_names() {
    assert_eq!(GitObjectId::parse(&SHA1.as_bytes()[..39]), None);
    assert_eq!(GitObjectId::parse(format!("{SHA1}0").as_bytes()), None);
    assert_eq!(GitObjectId::parse(b""), None);
}

#[test]
fn rejects_names_outside_lowercase_hexadecimal() {
    assert_eq!(GitObjectId::parse(SHA1.to_uppercase().as_bytes()), None);
    assert_eq!(GitObjectId::parse(SHA1.replace('0', "g").as_bytes()), None);
}

fn lookup(stdout: &str, name: &str) -> Option<Option<GitObjectId>> {
    GitObjectId::parse_lookup(stdout.as_bytes(), name)
}

#[test]
fn looks_up_object_named_in_any_form() {
    let found = Some(GitObjectId::parse(SHA1.as_bytes()));
    assert_eq!(lookup(&format!("{SHA1}\n"), ":/fix bug"), found);
    assert_eq!(lookup(&format!("{SHA1}\n"), SHA1), found);
}

#[test]
fn looks_up_nothing_for_name_missing_or_ambiguous() {
    assert_eq!(lookup(":/fix bug missing\n", ":/fix bug"), Some(None));
    assert_eq!(lookup(&format!("{SHA1} missing\n"), SHA1), Some(None));
    assert_eq!(lookup("0123 ambiguous\n", "0123"), Some(None));
    assert_eq!(lookup(" missing\n", ""), Some(None));
}

#[test]
fn rejects_lookup_of_other_name_or_form() {
    for (stdout, name) in [
        ("main missing\n", "dev"),
        ("main missing", "main"),
        ("main gone\n", "main"),
        ("main missing\nmain missing\n", "main"),
        (&format!("{SHA1}\n{SHA1}\n"), "main"),
        ("", "main"),
    ] {
        assert_eq!(lookup(stdout, name), None, "{stdout:?}");
    }
}
