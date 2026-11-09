//! Why a list option cannot be composed, worded for the user.

use std::fmt::{self, Display, Formatter};

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
        }
    }
}
