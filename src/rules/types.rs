use super::list::List;

/// Types that branch names and commit headers may start with.
#[derive(Debug, PartialEq, Eq)]
pub struct TypeRule(List);

impl TypeRule {
    /// Types commonly used with Conventional Commits.
    pub const DEFAULT: [&str; 11] = [
        "feat", "fix", "docs", "style", "refactor", "perf", "test", "build", "ci", "chore",
        "revert",
    ];

    /// Allows `types`, or [`Self::DEFAULT`] when it is empty.
    pub fn new(types: List) -> Self {
        if types.is_empty() {
            Self(Self::DEFAULT.into_iter().collect())
        } else {
            Self(types)
        }
    }

    pub fn allowed(&self) -> &List {
        &self.0
    }

    pub fn check(&self, found: &str) -> Result<(), UnknownType> {
        if self.0.contains(found) {
            Ok(())
        } else {
            Err(UnknownType {
                found: found.to_owned(),
                allowed: self.0.clone(),
            })
        }
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("unknown type `{found}`\n  allowed types: {allowed}")]
pub struct UnknownType {
    pub found: String,
    pub allowed: List,
}

#[cfg(test)]
#[path = "types_tests.rs"]
mod tests;
