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
fn reads_name_followed_by_line_break() {
    let line = format!("{SHA1}\n");
    assert_eq!(
        GitObjectId::parse_line(line.as_bytes()),
        GitObjectId::parse(SHA1.as_bytes())
    );
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

#[test]
fn rejects_line_not_ending_with_one_line_break() {
    assert_eq!(GitObjectId::parse_line(SHA1.as_bytes()), None);
    assert_eq!(
        GitObjectId::parse_line(format!("{SHA1}\n\n").as_bytes()),
        None
    );
}
