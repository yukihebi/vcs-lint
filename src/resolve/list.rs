//! Composition of a list option from its defaults, the values given to it and
//! the values excluded from it.

use crate::{
    name_set::NameSet,
    report::list::{ListName, ListResolveError},
};

/// A list option as written, before it is composed.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ListRawValues {
    /// The values the options adding to the list give, `None` when none of
    /// them appears.
    pub given: Option<Vec<String>>,
    pub excluded: Vec<String>,
}

impl ListRawValues {
    /// Composes `defaults` and the given values, in this order, without the
    /// excluded values, failing when an excluded value is in neither.
    pub fn resolve(self, list: ListName, defaults: &[&str]) -> Result<NameSet, ListResolveError> {
        let given = self.given.into_iter().flatten();
        let listed: NameSet = defaults
            .iter()
            .copied()
            .map(String::from)
            .chain(given)
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
}

#[cfg(test)]
#[path = "list_tests.rs"]
mod tests;
