use super::list::List;

/// Branches that must not receive direct commits.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProtectedRule(List);

impl ProtectedRule {
    pub fn new(branches: List) -> Self {
        Self(branches)
    }

    pub fn check(&self, branch: &str) -> Result<(), Protected> {
        if self.0.contains(branch) {
            Err(Protected {
                branch: branch.to_owned(),
            })
        } else {
            Ok(())
        }
    }
}

/// A direct commit to a protected branch.
///
/// The branch name format is not checked in this case, since renaming the
/// branch is not the fix.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error(
    "direct commits to `{branch}` are not allowed\n  create a branch and open a pull request instead"
)]
pub struct Protected {
    pub branch: String,
}

#[cfg(test)]
#[path = "protected_tests.rs"]
mod tests;
