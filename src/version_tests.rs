use super::Version;
use crate::report::version::RequiredVersionError;

fn version(written: &str) -> Version {
    Version::parse(written).unwrap_or_else(|| panic!("{written:?} is not a version"))
}

mod parse {
    use super::{Version, version};

    #[test]
    fn writes_back_what_it_parses() {
        for written in [
            "0.0.0",
            "1.22.333",
            "0.1.0-alpha.6",
            "1.0.0-beta.0",
            "2.3.4-rc.10",
            "18446744073709551615.0.0",
        ] {
            assert_eq!(version(written).to_string(), written);
        }
    }

    #[test]
    fn rejects_missing_or_empty_numbers() {
        for written in ["", "1", "1.2", "1.2.", "1..3", ".2.3", "1.2.3.4"] {
            assert_eq!(Version::parse(written), None, "{written:?}");
        }
    }

    #[test]
    fn rejects_numbers_not_in_decimal_digits() {
        for written in [
            "01.2.3",
            "1.02.3",
            "1.2.03",
            "1.2.3-rc.01",
            "+1.2.3",
            "1.-2.3",
            "1.2.x",
        ] {
            assert_eq!(Version::parse(written), None, "{written:?}");
        }
    }

    #[test]
    fn rejects_numbers_out_of_range() {
        for written in [
            "18446744073709551616.0.0",
            "0.0.0-alpha.18446744073709551616",
        ] {
            assert_eq!(Version::parse(written), None, "{written:?}");
        }
    }

    #[test]
    fn rejects_pre_releases_other_than_numbered_alpha_beta_rc() {
        for written in [
            "1.2.3-",
            "1.2.3-alpha",
            "1.2.3-alpha.",
            "1.2.3-alpha1",
            "1.2.3-Alpha.1",
            "1.2.3-pre.1",
            "1.2.3-alpha.1.2",
            "1.2.3-alpha.beta",
        ] {
            assert_eq!(Version::parse(written), None, "{written:?}");
        }
    }

    #[test]
    fn rejects_build_metadata() {
        for written in ["1.2.3+build", "1.2.3-rc.1+build.5", "1.2.3+"] {
            assert_eq!(Version::parse(written), None, "{written:?}");
        }
    }

    #[test]
    fn rejects_surrounding_text() {
        for written in ["v1.2.3", " 1.2.3", "1.2.3 ", "1.2.3\n"] {
            assert_eq!(Version::parse(written), None, "{written:?}");
        }
    }
}

mod parse_package {
    use super::{Version, version};

    #[test]
    fn ignores_build_metadata() {
        assert_eq!(
            Version::parse_package("1.2.3+build.5"),
            Some(version("1.2.3"))
        );
        assert_eq!(
            Version::parse_package("1.2.3-rc.1+build"),
            Some(version("1.2.3-rc.1"))
        );
    }

    #[test]
    fn rejects_what_parse_rejects_before_build_metadata() {
        assert_eq!(Version::parse_package("1.2.3-pre.1+build"), None);
        assert_eq!(Version::parse_package("1.2+build"), None);
    }
}

mod order {
    use super::version;

    /// Asserts that each version is older than the next one.
    fn assert_ascending(written: &[&str]) {
        for pair in written.windows(2) {
            assert!(version(pair[0]) < version(pair[1]), "{pair:?}");
        }
    }

    #[test]
    fn orders_numbers_by_value_from_major_to_patch() {
        assert_ascending(&["0.0.9", "0.0.10", "0.9.10", "0.10.0", "1.0.0", "10.0.0"]);
    }

    #[test]
    fn orders_pre_releases_before_their_release() {
        assert_ascending(&[
            "0.1.0-alpha.5",
            "0.1.0-alpha.6",
            "0.1.0-alpha.10",
            "0.1.0-beta.0",
            "0.1.0-rc.1",
            "0.1.0",
            "0.1.1-alpha.0",
        ]);
    }
}

mod require {
    use std::ffi::OsStr;

    use super::{RequiredVersionError, version};

    fn require(current: &str, required: &str) -> Result<(), RequiredVersionError> {
        version(current).require(OsStr::new(required))
    }

    #[test]
    fn accepts_older_or_same_version() {
        assert_eq!(require("0.2.0", "0.1.9"), Ok(()));
        assert_eq!(require("0.2.0", "0.2.0-rc.3"), Ok(()));
        assert_eq!(require("0.2.0", "0.2.0"), Ok(()));
    }

    #[test]
    fn fails_on_newer_version() {
        assert_eq!(
            require("0.2.0-rc.3", "0.2.0"),
            Err(RequiredVersionError::Unmet {
                required: version("0.2.0"),
                current: version("0.2.0-rc.3"),
            })
        );
    }

    #[test]
    fn trims_surrounding_whitespace() {
        assert_eq!(require("0.2.0", " \t0.2.0\n"), Ok(()));
        assert!(matches!(
            require("0.2.0", " 0.3.0 "),
            Err(RequiredVersionError::Unmet { .. })
        ));
    }

    #[test]
    fn accepts_anything_when_blank() {
        assert_eq!(require("0.0.0-alpha.0", ""), Ok(()));
        assert_eq!(require("0.0.0-alpha.0", " \t\n"), Ok(()));
    }

    #[test]
    fn fails_on_invalid_version_trimmed() {
        assert_eq!(
            require("0.2.0", " 0.2 "),
            Err(RequiredVersionError::Invalid("0.2".to_owned()))
        );
    }
}

#[cfg(unix)]
#[test]
fn fails_on_required_version_not_in_utf8() {
    use std::{ffi::OsStr, os::unix::ffi::OsStrExt};

    assert_eq!(
        version("0.2.0").require(OsStr::from_bytes(b"0.2.\xff")),
        Err(RequiredVersionError::Invalid("0.2.\u{fffd}".to_owned()))
    );
}
