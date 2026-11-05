/// What a commit message file amounts to once the lines git strips before
/// committing are set aside.
#[derive(Debug, PartialEq, Eq)]
pub enum Message<'a> {
    /// The first line of the message, to be linted.
    Header(&'a str),
    /// A message git creates for merges or autosquash, accepted as is.
    Exempt,
    /// The message has no line left.
    Empty,
}

impl<'a> Message<'a> {
    /// Marks the start of the diff added by `git commit --verbose`.
    const SCISSORS: &str = "# ------------------------ >8 ------------------------";

    const EXEMPT_PREFIXES: [&str; 4] = ["Merge ", "fixup! ", "squash! ", "amend! "];

    pub fn parse(text: &'a str) -> Self {
        let header = text
            .lines()
            .take_while(|line| *line != Self::SCISSORS)
            .find(|line| !line.starts_with('#') && !line.trim().is_empty());
        match header {
            None => Self::Empty,
            Some(header) if Self::is_exempt(header) => Self::Exempt,
            Some(header) => Self::Header(header),
        }
    }

    fn is_exempt(header: &str) -> bool {
        Self::EXEMPT_PREFIXES
            .iter()
            .any(|prefix| header.starts_with(prefix))
    }
}

#[cfg(test)]
#[path = "message_tests.rs"]
mod tests;
