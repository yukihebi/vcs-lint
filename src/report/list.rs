//! Why a list option cannot be composed or holds values nothing can match,
//! worded for the user.

use std::{
    fmt::{self, Display, Formatter},
    path::PathBuf,
};

use super::git::GitSubdirsError;
use crate::name_set::NameSet;

/// A list option, named after what its values are.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListName {
    Types,
    Scopes,
    Protected,
}

impl ListName {
    /// What the values of the list are, in the plural.
    fn values(self) -> &'static str {
        match self {
            Self::Types => "allowed types",
            Self::Scopes => "allowed scopes",
            Self::Protected => "protected branches",
        }
    }

    /// The option that adds values to the list.
    fn option(self) -> &'static str {
        match self {
            Self::Types => "--type",
            Self::Scopes => "--scope",
            Self::Protected => "--protected",
        }
    }

    /// The option that adds the names of the subdirectories of a directory to
    /// the list.
    fn dir_option(self) -> &'static str {
        match self {
            Self::Types => "--type-dir",
            Self::Scopes => "--scope-dir",
            Self::Protected => "--protected-dir",
        }
    }
}

/// Why the values of a list option cannot be composed.
#[derive(Debug, PartialEq, Eq)]
pub enum ListResolveError {
    /// Values to exclude that the list does not hold before exclusion.
    UnknownExclusions {
        list: ListName,
        unknown: NameSet,
        /// The values of the list before exclusion.
        listed: NameSet,
    },
    /// The list holds no value, but a value is needed for anything to pass.
    Empty { list: ListName },
    /// The subdirectories of a directory given to the list cannot be read.
    Dir {
        list: ListName,
        /// The directory as given.
        dir: PathBuf,
        cause: GitSubdirsError,
    },
}

impl Display for ListResolveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownExclusions {
                list,
                unknown,
                listed,
            } => {
                let values = list.values();
                write!(f, "cannot exclude values outside the {values}: {unknown}")?;
                if listed.is_empty() {
                    write!(f, "\n  there are no {values}")
                } else {
                    write!(f, "\n  {values}: {listed}")
                }
            }
            Self::Empty { list } => write!(
                f,
                "there are no {}\n  give one with `{}`",
                list.values(),
                list.option()
            ),
            Self::Dir { list, dir, cause } => {
                write!(
                    f,
                    "cannot read the directories under `{}` for `{}`",
                    dir.display(),
                    list.dir_option()
                )?;
                for line in cause.to_string().lines() {
                    write!(f, "\n  {line}")?;
                }
                Ok(())
            }
        }
    }
}

/// Values of a list that nothing linted can match, rendered as lines indented
/// under the error holding them.
#[derive(Debug, PartialEq, Eq)]
pub struct UnmatchableList {
    pub list: ListName,
    pub values: NameSet,
    /// What every value must meet, worded as a sentence.
    pub requirement: &'static str,
}

impl UnmatchableList {
    /// Renders `lists` one after another, each starting on a new line.
    pub fn render_all(lists: &[Self]) -> String {
        lists.iter().map(|list| format!("\n{list}")).collect()
    }
}

/// Values are quoted with their line breaks escaped, as they may hold any
/// character.
impl Display for UnmatchableList {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let values: Vec<String> = self
            .values
            .iter()
            .map(|value| format!("{value:?}"))
            .collect();
        write!(
            f,
            "  {}: {}\n    {}",
            self.list.values(),
            values.join(", "),
            self.requirement
        )
    }
}
