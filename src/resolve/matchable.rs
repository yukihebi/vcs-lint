//! Values of a list option that nothing linted can match.

use crate::{
    name_set::NameSet,
    report::list::{ListName, UnmatchableList},
};

/// What every value of a list must meet for anything linted to match it.
///
/// What is linted is a single line, so a value with a line break never meets
/// a requirement.
pub struct ListRequirement {
    pub list: ListName,
    /// Whether a value without a line break can be matched.
    pub is_matchable: fn(&str) -> bool,
    /// The requirement worded as a sentence, mentioning line breaks unless
    /// `is_matchable` already rules them out.
    pub wording: &'static str,
}

impl ListRequirement {
    /// The values of `values` that fail the requirement, in order, or `None`
    /// when every value meets it.
    pub fn find_unmatchable(&self, values: &NameSet) -> Option<UnmatchableList> {
        let unmatchable: NameSet = values.iter().filter(|value| !self.is_met(value)).collect();
        (!unmatchable.is_empty()).then_some(UnmatchableList {
            list: self.list,
            values: unmatchable,
            requirement: self.wording,
        })
    }

    fn is_met(&self, value: &str) -> bool {
        !value.contains(['\n', '\r']) && (self.is_matchable)(value)
    }
}

#[cfg(test)]
#[path = "matchable_tests.rs"]
mod tests;
