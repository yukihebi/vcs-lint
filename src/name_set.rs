use derive_more::Display;

/// Names in the order they were first given, each kept once.
#[derive(Debug, Clone, Default, PartialEq, Eq, Display)]
#[display("{}", _0.join(", "))]
pub struct NameSet(Vec<String>);

impl NameSet {
    /// Whether `name` is one of the names, compared exactly.
    pub fn contains(&self, name: &str) -> bool {
        self.0.iter().any(|n| n == name)
    }
}

/// A repeated name keeps its first position.
impl<S: Into<String>> FromIterator<S> for NameSet {
    fn from_iter<I: IntoIterator<Item = S>>(names: I) -> Self {
        let mut set = Vec::new();
        for name in names.into_iter().map(Into::into) {
            if !set.contains(&name) {
                set.push(name);
            }
        }
        Self(set)
    }
}

#[cfg(test)]
#[path = "name_set_tests.rs"]
mod tests;
