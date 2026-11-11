//! Options read from a config file: a TOML file whose keys are the long names
//! of options, read at its top level by every command that has the option and
//! in the table named after a command by that command alone.
//!
//! A command reads the options of a file as flags given before its command
//! line, so the rules of the command line apply to them as they are. They are
//! checked against those rules before, so that what breaks them is reported by
//! the keys of the file.

use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

use toml::{Table, Value};

use crate::{
    cli::{CommandName, CommandSyntax},
    env::Env,
    report::config_file::{ConfigFileError, ConfigKey, ConfigKeyError, ConfigKeyErrorKind},
};

/// The options a valid config file gives each command.
#[derive(Debug)]
pub struct ConfigFile {
    /// The file as given.
    path: PathBuf,
    /// The options each command reads: those at the top level that it has,
    /// then those in its table.
    commands: Vec<(CommandName, Vec<ConfigOption>)>,
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
        let path = path.to_owned();
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(source) => return Err(ConfigFileError::Unreadable { path, source }),
        };
        let table: Table = match text.parse() {
            Ok(table) => table,
            Err(source) => return Err(ConfigFileError::Syntax { path, source }),
        };
        match ConfigReader::new().read(table) {
            Ok(commands) => Ok(Self { path, commands }),
            Err(keys) => Err(ConfigFileError::Keys { path, keys }),
        }
    }

    /// The flags `command` reads from the file, in the order it reads them,
    /// failing when the file gives an option given once whose long name is in
    /// `written`, the options written on the command line.
    pub fn flags(
        &self,
        command: CommandName,
        written: &[String],
    ) -> Result<Vec<OsString>, ConfigFileError> {
        let options = self
            .commands
            .iter()
            .find(|(name, _)| *name == command)
            .map_or(&[][..], |(_, options)| options);
        let keys: Vec<ConfigKey> = options
            .iter()
            .filter(|option| !option.is_list && written.contains(&option.key.name))
            .map(|option| option.key.clone())
            .collect();
        if !keys.is_empty() {
            return Err(ConfigFileError::Written {
                path: self.path.clone(),
                keys,
            });
        }
        Ok(options.iter().flat_map(ConfigOption::flags).collect())
    }
}

/// An option given by a key, with the value of each of its flags.
#[derive(Debug, Clone)]
struct ConfigOption {
    /// The key giving the option, named after it.
    key: ConfigKey,
    /// Whether each flag of the option adds to its values.
    is_list: bool,
    values: Vec<String>,
}

impl ConfigOption {
    /// Each value written as `--NAME=VALUE`, which takes a value starting with
    /// `-` as the value rather than as another option.
    fn flags(&self) -> impl Iterator<Item = OsString> {
        self.values
            .iter()
            .map(|value| format!("--{}={value}", self.key.name).into())
    }
}

/// Reads the keys of a file against the syntax of every command, collecting
/// every invalid key.
struct ConfigReader {
    commands: Vec<(CommandName, CommandSyntax)>,
    errors: Vec<ConfigKeyError>,
}

impl ConfigReader {
    fn new() -> Self {
        Self {
            commands: CommandName::ALL
                .into_iter()
                .map(|command| (command, command.syntax()))
                .collect(),
            errors: Vec::new(),
        }
    }

    /// Reads `table`, the top level of a file, into the options each command
    /// reads, failing with every invalid key.
    fn read(
        mut self,
        table: Table,
    ) -> Result<Vec<(CommandName, Vec<ConfigOption>)>, Vec<ConfigKeyError>> {
        let mut shared = Vec::new();
        let mut tables = Vec::new();
        for (name, value) in table {
            match CommandName::from_name(&name) {
                Some(command) => tables.extend(self.read_table(command, value)),
                None => shared.extend(self.read_option(ConfigKey { table: None, name }, value)),
            }
        }
        let commands = self.compose(shared, tables);
        for (_, options) in &commands {
            self.check_repeated(options);
        }
        if self.errors.is_empty() {
            Ok(commands)
        } else {
            Err(self.errors)
        }
    }

    /// The options each command reads: those of `shared`, at the top level,
    /// that it has, then those of its table in `tables`.
    fn compose(
        &self,
        shared: Vec<ConfigOption>,
        mut tables: Vec<(CommandName, Vec<ConfigOption>)>,
    ) -> Vec<(CommandName, Vec<ConfigOption>)> {
        self.commands
            .iter()
            .map(|(command, syntax)| {
                let read = shared
                    .iter()
                    .filter(|option| syntax.option(&option.key.name).is_some())
                    .cloned();
                let own = tables
                    .iter()
                    .position(|(table, _)| table == command)
                    .map(|i| tables.swap_remove(i).1)
                    .unwrap_or_default();
                (*command, read.chain(own).collect())
            })
            .collect()
    }

    /// Fails on each option given once that `options`, those of one command,
    /// give again, which only a table can do after the top level.
    fn check_repeated(&mut self, options: &[ConfigOption]) {
        for (i, option) in options.iter().enumerate() {
            let repeats = !option.is_list
                && options[..i]
                    .iter()
                    .any(|earlier| earlier.key.name == option.key.name);
            if repeats {
                self.errors.push(ConfigKeyError {
                    key: option.key.clone(),
                    kind: ConfigKeyErrorKind::Repeated,
                });
            }
        }
    }

    /// Reads `value`, the table of `command`, unless it is not a table.
    fn read_table(
        &mut self,
        command: CommandName,
        value: Value,
    ) -> Option<(CommandName, Vec<ConfigOption>)> {
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
        Some((command, options))
    }

    /// Reads `value` as the option `key` names, unless it is invalid.
    fn read_option(&mut self, key: ConfigKey, value: Value) -> Option<ConfigOption> {
        match self.option(key, value) {
            Ok(option) => Some(option),
            Err(errors) => {
                self.errors.extend(errors);
                None
            }
        }
    }

    /// The option `key` names, with the values `value` gives, each checked as
    /// the command line checks the value of one flag.
    fn option(&self, key: ConfigKey, value: Value) -> Result<ConfigOption, Vec<ConfigKeyError>> {
        let fail = |kinds: Vec<ConfigKeyErrorKind>| -> Vec<ConfigKeyError> {
            kinds
                .into_iter()
                .map(|kind| ConfigKeyError {
                    key: key.clone(),
                    kind,
                })
                .collect()
        };
        if ConfigFile::UNWRITABLE.contains(&key.name.as_str()) {
            return Err(fail(vec![ConfigKeyErrorKind::Unwritable]));
        }
        let Some(option) = self
            .syntax(key.table)
            .find_map(|syntax| syntax.option(&key.name))
        else {
            return Err(fail(vec![ConfigKeyErrorKind::Unknown]));
        };
        let is_list = option.is_list();
        let values = Self::values(is_list, value).map_err(|kind| fail(vec![kind]))?;
        let rejected: Vec<ConfigKeyErrorKind> = values
            .iter()
            .filter_map(|value| {
                let rejection = option.check(value).err()?;
                Some(ConfigKeyErrorKind::InvalidValue {
                    value: value.clone(),
                    rejection,
                })
            })
            .collect();
        if !rejected.is_empty() {
            return Err(fail(rejected));
        }
        Ok(ConfigOption {
            key,
            is_list,
            values,
        })
    }

    /// The syntax of the commands reading a key in the table of `table`, or at
    /// the top level when `None`. The commands share the syntax of an option
    /// they both have, so a key at the top level is checked against the first
    /// command having it.
    fn syntax(&self, table: Option<CommandName>) -> impl Iterator<Item = &CommandSyntax> {
        self.commands
            .iter()
            .filter(move |(command, _)| table.is_none_or(|table| table == *command))
            .map(|(_, syntax)| syntax)
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
