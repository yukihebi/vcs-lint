//! Composition of a list option from its defaults, the values given to it and
//! the values excluded from it.

use std::path::PathBuf;

use crate::{
    git::Git,
    name_set::NameSet,
    report::list::{ListName, ListResolveError},
};

/// A list option as written, before it is composed.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ListRawValues {
    /// The values the options adding to the list give, `None` when none of
    /// them appears.
    pub given: Option<Vec<String>>,
    /// The directories whose subdirectories the options adding them to the
    /// list give, `None` when none of them appears.
    pub dirs: Option<Vec<PathBuf>>,
    pub excluded: Vec<String>,
}

impl ListRawValues {
    /// Whether an option adding to the list appears, even if it adds nothing.
    pub fn is_given(&self) -> bool {
        self.given.is_some() || self.dirs.is_some()
    }

    /// Composes `defaults`, the given values and the names of the
    /// subdirectories of the given directories, in this order, without the
    /// excluded values, failing when a directory cannot be read or an
    /// excluded value is in none of them.
    pub fn resolve(
        self,
        list: ListName,
        defaults: &[&str],
        git: &Git,
    ) -> Result<NameSet, ListResolveError> {
        let subdirs = Self::read_subdirs(list, self.dirs.unwrap_or_default(), git)?;
        let given = self.given.into_iter().flatten();
        let listed: NameSet = defaults
            .iter()
            .copied()
            .map(String::from)
            .chain(given)
            .chain(subdirs)
            .collect();
        let excluded: NameSet = self.excluded.into_iter().collect();
        let unknown: NameSet = excluded
            .iter()
            .filter(|value| !listed.contains(value))
            .collect();
        if unknown.is_empty() {
            Ok(listed
                .iter()
                .filter(|value| !excluded.contains(value))
                .collect())
        } else {
            Err(ListResolveError::UnknownExclusions {
                list,
                unknown,
                listed,
            })
        }
    }

    /// The names of the subdirectories of `dirs` that the index holds entries
    /// in, directory by directory.
    fn read_subdirs(
        list: ListName,
        dirs: Vec<PathBuf>,
        git: &Git,
    ) -> Result<Vec<String>, ListResolveError> {
        let mut names = Vec::new();
        for dir in dirs {
            match git.index_subdirs(&dir) {
                Ok(subdirs) => names.extend(subdirs),
                Err(cause) => return Err(ListResolveError::Dir { list, dir, cause }),
            }
        }
        Ok(names)
    }
}

#[cfg(test)]
#[path = "list_tests.rs"]
mod tests;
