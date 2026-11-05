use super::list::List;

/// Whether commit headers carry a scope, and which scopes they may use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeRule {
    /// Headers must not have a scope.
    Forbidden,
    /// Headers must have one of the listed scopes.
    Required(List),
}

impl ScopeRule {
    /// Requires one of `scopes`, or forbids scopes when it is empty.
    pub fn new(scopes: List) -> Self {
        if scopes.is_empty() {
            Self::Forbidden
        } else {
            Self::Required(scopes)
        }
    }

    pub fn check(&self, found: Option<&str>) -> Result<(), ScopeError> {
        match (self, found) {
            (Self::Forbidden, None) => Ok(()),
            (Self::Forbidden, Some(found)) => Err(ScopeError::Unexpected {
                found: found.to_owned(),
            }),
            (Self::Required(allowed), None) => Err(ScopeError::Missing {
                allowed: allowed.clone(),
            }),
            (Self::Required(allowed), Some(found)) if allowed.contains(found) => Ok(()),
            (Self::Required(allowed), Some(found)) => Err(ScopeError::Unknown {
                found: found.to_owned(),
                allowed: allowed.clone(),
            }),
        }
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ScopeError {
    #[error("unexpected scope `{found}`\n  scopes are not used; remove `({found})`")]
    Unexpected { found: String },
    #[error("missing scope\n  allowed scopes: {allowed}")]
    Missing { allowed: List },
    #[error("unknown scope `{found}`\n  allowed scopes: {allowed}")]
    Unknown { found: String, allowed: List },
}

#[cfg(test)]
#[path = "scopes_tests.rs"]
mod tests;
