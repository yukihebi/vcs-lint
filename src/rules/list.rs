use derive_more::Display;

/// An ordered set of values given through a list option.
///
/// Values are trimmed of surrounding whitespace, empty values are dropped, and
/// a repeated value keeps its first position, so the list reads the same way
/// it was written.
#[derive(Debug, Clone, Default, PartialEq, Eq, Display)]
#[display("{}", _0.join(", "))]
pub struct List(Vec<String>);

impl List {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn contains(&self, value: &str) -> bool {
        self.0.iter().any(|v| v == value)
    }
}

impl<S: Into<String>> FromIterator<S> for List {
    fn from_iter<I: IntoIterator<Item = S>>(values: I) -> Self {
        let mut list = Vec::new();
        for value in values.into_iter().map(Into::into) {
            let value = value.trim();
            if !value.is_empty() && !list.iter().any(|v| v == value) {
                list.push(value.to_owned());
            }
        }
        Self(list)
    }
}

#[cfg(test)]
#[path = "list_tests.rs"]
mod tests;
