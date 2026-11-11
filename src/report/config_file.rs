//! Why a config file cannot be read, worded for the user in terms of its keys.

use std::{
    fmt::{self, Display, Formatter},
    io,
    path::PathBuf,
};

use derive_more::Display;

use crate::cli::CommandName;

/// Why the options of a config file cannot be read.
#[derive(Debug, Display)]
pub enum ConfigFileError {
    #[display("cannot read config file `{}`\n  {source}", path.display())]
    Unreadable { path: PathBuf, source: io::Error },
    /// The file is not TOML.
    #[display(
        "invalid config file `{}`{}",
        path.display(),
        ConfigFileError::indent(source)
    )]
    Syntax {
        path: PathBuf,
        source: toml::de::Error,
    },
    /// Keys of the file are invalid; at least one key is present.
    #[display(
        "invalid config file `{}`{}",
        path.display(),
        keys.iter().map(ConfigFileError::indent).collect::<String>()
    )]
    Keys {
        path: PathBuf,
        keys: Vec<ConfigKeyError>,
    },
    /// Keys of the file give options that the command line gives as well,
    /// which can be given once; at least one key is present.
    #[display(
        "config file `{}` and the command line both give options that can be given once{}",
        path.display(),
        ConfigFileError::render_written(keys)
    )]
    Written { path: PathBuf, keys: Vec<ConfigKey> },
}

impl ConfigFileError {
    /// Renders `cause` as lines indented under the error holding it.
    fn indent(cause: &impl Display) -> String {
        cause
            .to_string()
            .lines()
            .map(|line| format!("\n  {line}"))
            .collect()
    }

    /// Renders each key beside the option written on the command line.
    fn render_written(keys: &[ConfigKey]) -> String {
        keys.iter()
            .map(|key| format!("\n  `{key}` and `--{}`", key.name))
            .collect()
    }
}

/// A key of a config file that no command can read as written.
#[derive(Debug, PartialEq, Eq)]
pub struct ConfigKeyError {
    pub key: ConfigKey,
    pub kind: ConfigKeyErrorKind,
}

/// Why a key cannot be read.
#[derive(Debug, PartialEq, Eq)]
pub enum ConfigKeyErrorKind {
    /// The key names no option of the commands reading it.
    Unknown,
    /// The key names an option that cannot be given in a file.
    Unwritable,
    /// The key of a command is not a table.
    NotTable,
    /// The key of an option given once is not a string.
    NotString,
    /// The key of a list option is not an array of strings.
    NotStringArray,
    /// A string of the key is a value the option does not take.
    InvalidValue {
        value: String,
        rejection: ValueRejection,
    },
    /// The key in the table of a command gives an option given once that the
    /// command reads at the top level as well.
    Repeated,
}

impl Display for ConfigKeyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self { key, kind } = self;
        match (kind, key.table) {
            (ConfigKeyErrorKind::Unknown, None) => {
                write!(
                    f,
                    "unknown key `{key}`\n  no command has an option of this name"
                )
            }
            (ConfigKeyErrorKind::Unknown, Some(command)) => write!(
                f,
                "unknown key `{key}`\n  `vcs-lint {}` has no option of this name",
                command.as_str()
            ),
            (ConfigKeyErrorKind::Unwritable, _) => write!(
                f,
                "disallowed key `{key}`\n  this option cannot be given in a config file"
            ),
            (ConfigKeyErrorKind::NotTable, _) => write!(
                f,
                "invalid value of `{key}`\n  expected a table of the options of `vcs-lint {}`",
                key.name
            ),
            (ConfigKeyErrorKind::NotString, _) => {
                write!(f, "invalid value of `{key}`\n  expected a string")
            }
            (ConfigKeyErrorKind::NotStringArray, _) => {
                write!(
                    f,
                    "invalid value of `{key}`\n  expected an array of strings"
                )
            }
            (ConfigKeyErrorKind::InvalidValue { value, rejection }, _) => {
                write!(f, "invalid value {value:?} of `{key}`\n  {rejection}")
            }
            (ConfigKeyErrorKind::Repeated, _) => write!(
                f,
                "repeated key `{key}`\n  `{}` at the top level gives the same option, which can be given once",
                ConfigKey {
                    table: None,
                    name: key.name.clone(),
                }
            ),
        }
    }
}

/// Why the command line rejects a value of an option.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueRejection {
    /// The value is not one of the values the option takes, listed in the
    /// order the option lists them; at least one value is listed.
    NotPossible { possible: Vec<String> },
    /// The value is invalid for `reason`.
    Invalid { reason: String },
}

impl Display for ValueRejection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotPossible { possible } => {
                let quoted: Vec<String> =
                    possible.iter().map(|value| format!("`{value}`")).collect();
                let (last, rest) = quoted.split_last().expect("a value is listed");
                if rest.is_empty() {
                    write!(f, "expected {last}")
                } else {
                    write!(f, "expected {} or {last}", rest.join(", "))
                }
            }
            Self::Invalid { reason } => f.write_str(reason),
        }
    }
}

/// A key of a config file, at its top level or in the table of a command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigKey {
    /// The command whose table holds the key, `None` at the top level.
    pub table: Option<CommandName>,
    pub name: String,
}

/// Written as a dotted key, each part quoted unless it is a bare key, so it
/// reads as the key would be written in TOML.
impl Display for ConfigKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(command) = self.table {
            write!(f, "{}.", command.as_str())?;
        }
        let is_bare = !self.name.is_empty()
            && self
                .name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
        if is_bare {
            f.write_str(&self.name)
        } else {
            write!(f, "{:?}", self.name)
        }
    }
}
