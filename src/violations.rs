use std::fmt::{self, Display, Formatter};

/// Every violation found in one input, reported together.
///
/// Never empty when built by [`Violations::collect`].
#[derive(Debug, PartialEq, Eq)]
pub struct Violations<V>(pub Vec<V>);

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
        let text = self
            .0
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        let mut lines = text.lines();
        if let Some(first) = lines.next() {
            write!(f, "  {first}")?;
        }
        for line in lines {
            write!(f, "\n  {line}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "violations_tests.rs"]
mod tests;
