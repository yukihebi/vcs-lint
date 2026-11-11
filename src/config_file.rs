//! Options read from a config file: a TOML file whose keys are the long names
//! of options, read at its top level by every command that has the option and
//! in the table named after a command by that command alone.
//!
//! A command reads the options of a file as flags given before its command
//! line, so the rules of the command line apply to them as they are.

use std::{collections::HashMap, ffi::OsString, fs, path::Path};

use toml::{Table, Value};

use crate::{
    cli::{CommandName, CommandOption},
    env::Env,
    report::config_file::{ConfigFileError, ConfigKey, ConfigKeyError, ConfigKeyErrorKind},
};

/// The options a valid config file gives each command.
#[derive(Debug)]
pub struct ConfigFile {
    /// The options of each command that the file gives any: those at the top
    /// level that it has, then those in its table.
    commands: HashMap<CommandName, Vec<ConfigOption>>,
}

impl ConfigFile {
    /// Options a command has that a file cannot give, as they are not rules.
    const UNWRITABLE: [&str; 2] = ["config", "help"];

    /// The file a run reads: the one given with `--config`, otherwise the one
    /// named by `VCS_LINT_CONFIG` unless it is empty.
    pub fn path<'a>(given: Option<&'a Path>, env: &'a Env) -> Option<&'a Path> {
        given.or_else(|| {
            env.config
                .as_deref()
                .filter(|path| !path.is_empty())
                .map(Path::new)
        })
    }

    /// Reads the file at `path`, failing unless every key of it is valid for
    /// every command.
    pub fn read(path: &Path) -> Result<Self, ConfigFileError> {
        let text = fs::read_to_string(path).map_err(|source| ConfigFileError::Unreadable {
            path: path.to_owned(),
            source,
        })?;
        let table: Table = text.parse().map_err(|source| ConfigFileError::Syntax {
            path: path.to_owned(),
            source,
        })?;
        ConfigReader::new()
            .read(table)
            .map_err(|keys| ConfigFileError::Keys {
                path: path.to_owned(),
                keys,
            })
    }

    /// The flags `command` reads from the file, in the order it reads them.
    pub fn flags(&self, command: CommandName) -> Vec<OsString> {
        self.commands
            .get(&command)
            .into_iter()
            .flatten()
            .flat_map(ConfigOption::flags)
            .collect()
    }
}

/// An option given by a key, with the value of each of its flags.
#[derive(Debug, Clone)]
struct ConfigOption {
    name: String,
    values: Vec<String>,
}

impl ConfigOption {
    /// Each value written as `--NAME=VALUE`, which takes a value starting with
    /// `-` as the value rather than as another option.
    fn flags(&self) -> impl Iterator<Item = OsString> {
        self.values
            .iter()
            .map(|value| format!("--{}={value}", self.name).into())
    }
}

/// Reads the keys of a file against the options of every command, collecting
/// every invalid key.
struct ConfigReader {
    options: Vec<(CommandName, Vec<CommandOption>)>,
    errors: Vec<ConfigKeyError>,
}

impl ConfigReader {
    fn new() -> Self {
        Self {
            options: CommandName::ALL
                .into_iter()
                .map(|command| (command, command.options()))
                .collect(),
            errors: Vec::new(),
        }
    }

    /// Reads `table`, the top level of a file, failing with every invalid key.
    fn read(mut self, table: Table) -> Result<ConfigFile, Vec<ConfigKeyError>> {
        let mut shared = Vec::new();
        let mut tables = HashMap::new();
        for (name, value) in table {
            match CommandName::from_name(&name) {
                Some(command) => {
                    if let Some(options) = self.read_table(command, value) {
                        tables.insert(command, options);
                    }
                }
                None => shared.extend(self.read_option(ConfigKey { table: None, name }, value)),
            }
        }
        if !self.errors.is_empty() {
            return Err(self.errors);
        }
        let commands = self
            .options
            .iter()
            .map(|(command, options)| {
                let has = |option: &&ConfigOption| options.iter().any(|o| o.name == option.name);
                let read = shared.iter().filter(has).cloned();
                let own = tables.remove(command).unwrap_or_default();
                (*command, read.chain(own).collect())
            })
            .collect();
        Ok(ConfigFile { commands })
    }

    /// Reads `value`, the table of `command`, unless it is not a table.
    fn read_table(&mut self, command: CommandName, value: Value) -> Option<Vec<ConfigOption>> {
        let Value::Table(table) = value else {
            self.errors.push(ConfigKeyError {
                key: ConfigKey {
                    table: None,
                    name: command.as_str().to_owned(),
                },
                kind: ConfigKeyErrorKind::NotTable,
            });
            return None;
        };
        let options = table
            .into_iter()
            .filter_map(|(name, value)| {
                let key = ConfigKey {
                    table: Some(command),
                    name,
                };
                self.read_option(key, value)
            })
            .collect();
        Some(options)
    }

    /// Reads `value` as the values of the option `key` names, unless it is
    /// invalid.
    fn read_option(&mut self, key: ConfigKey, value: Value) -> Option<ConfigOption> {
        let values = if ConfigFile::UNWRITABLE.contains(&key.name.as_str()) {
            Err(ConfigKeyErrorKind::Unwritable)
        } else {
            match self.find(&key) {
                Some(option) => Self::values(option.is_list, value),
                None => Err(ConfigKeyErrorKind::Unknown),
            }
        };
        match values {
            Ok(values) => Some(ConfigOption {
                name: key.name,
                values,
            }),
            Err(kind) => {
                self.errors.push(ConfigKeyError { key, kind });
                None
            }
        }
    }

    /// The option `key` names among those of the commands reading it: the
    /// command whose table holds it, or every command at the top level.
    fn find(&self, key: &ConfigKey) -> Option<&CommandOption> {
        self.options
            .iter()
            .filter(|(command, _)| key.table.is_none_or(|table| table == *command))
            .flat_map(|(_, options)| options)
            .find(|option| option.name == key.name)
    }

    /// The value of each flag `value` gives: one string for an option given
    /// once, or each string of an array for a list option.
    fn values(is_list: bool, value: Value) -> Result<Vec<String>, ConfigKeyErrorKind> {
        match (is_list, value) {
            (false, Value::String(value)) => Ok(vec![value]),
            (false, _) => Err(ConfigKeyErrorKind::NotString),
            (true, Value::Array(values)) => values
                .into_iter()
                .map(|value| match value {
                    Value::String(value) => Some(value),
                    _ => None,
                })
                .collect::<Option<_>>()
                .ok_or(ConfigKeyErrorKind::NotStringArray),
            (true, _) => Err(ConfigKeyErrorKind::NotStringArray),
        }
    }
}

#[cfg(test)]
#[path = "config_file_tests.rs"]
mod tests;
