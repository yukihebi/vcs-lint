//! Why a config file cannot be read, worded for the user.

use std::{
    fmt::{self, Display, Formatter},
    io,
    path::PathBuf,
};

use derive_more::Display;

use crate::cli::CommandName;

/// Why a config file cannot be read, so no command reads it.
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
}

/// A key of a config file that no command can read as written.
#[derive(Debug, PartialEq, Eq)]
pub struct ConfigKeyError {
    pub key: ConfigKey,
    pub kind: ConfigKeyErrorKind,
}

/// Why a key cannot be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}

impl Display for ConfigKeyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self { key, kind } = self;
        let name = &key.name;
        match (kind, key.table) {
            (ConfigKeyErrorKind::Unknown, None) => {
                write!(f, "unknown key `{key}`\n  no command has `--{name}`")
            }
            (ConfigKeyErrorKind::Unknown, Some(command)) => write!(
                f,
                "unknown key `{key}`\n  `vcs-lint {}` has no `--{name}`",
                command.as_str()
            ),
            (ConfigKeyErrorKind::Unwritable, _) => write!(
                f,
                "disallowed key `{key}`\n  `--{name}` cannot be given in a config file"
            ),
            (ConfigKeyErrorKind::NotTable, _) => write!(
                f,
                "invalid value of `{key}`\n  expected a table of the options of `vcs-lint {name}`"
            ),
            (ConfigKeyErrorKind::NotString, _) => {
                write!(f, "invalid value of `{key}`\n  expected a string")
            }
            (ConfigKeyErrorKind::NotStringArray, _) => write!(
                f,
                "invalid value of `{key}`\n  expected an array of strings, each given to one `--{name}`"
            ),
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
