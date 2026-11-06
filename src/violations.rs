use std::fmt::{self, Display, Formatter};

/// Every violation found in one input, reported together. Never empty.
#[derive(Debug, PartialEq, Eq)]
pub struct Violations<V>(Vec<V>);

impl<V> Violations<V> {
    /// Gathers the failures among `results`, failing if there is any.
    pub fn collect(results: impl IntoIterator<Item = Result<(), V>>) -> Result<(), Self> {
        let violations: Vec<V> = results.into_iter().filter_map(Result::err).collect();
        if violations.is_empty() {
            Ok(())
        } else {
            Err(Self(violations))
        }
    }
}

impl<V> From<V> for Violations<V> {
    fn from(violation: V) -> Self {
        Self(vec![violation])
    }
}

/// Lists each violation on its own lines, indented under the message that
/// names the input.
impl<V: Display> Display for Violations<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut separator = "";
        for violation in &self.0 {
            for line in violation.to_string().lines() {
                write!(f, "{separator}  {line}")?;
                separator = "\n";
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "violations_tests.rs"]
mod tests;
