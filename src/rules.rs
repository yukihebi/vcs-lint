//! What each lint accepts, resolved from the command line before any input is
//! inspected.

pub mod list;
pub mod protected;
pub mod scopes;
pub mod types;

use protected::ProtectedRule;
use scopes::ScopeRule;
use types::TypeRule;

/// Rules a branch name is checked against.
#[derive(Debug, PartialEq, Eq)]
pub struct BranchRules {
    pub types: TypeRule,
    pub protected: ProtectedRule,
}

/// Rules a commit header is checked against.
#[derive(Debug, PartialEq, Eq)]
pub struct CommitRules {
    pub types: TypeRule,
    pub scopes: ScopeRule,
}
