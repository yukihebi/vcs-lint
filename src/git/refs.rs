//! Refs as git lists them with the objects they point to.

use super::object_id::GitObjectId;

/// A ref listed as `<object>\t<refname>`, the form of `git ls-remote` and of
/// `git for-each-ref --format='%(objectname)%09%(refname)'`.
pub struct GitListedRef<'o> {
    id: GitObjectId,
    name: &'o [u8],
}

impl<'o> GitListedRef<'o> {
    /// Finds the object the ref named exactly `name` points to among the refs
    /// listed in `stdout`, giving `Some(None)` when it is not listed.
    ///
    /// Given a name, both commands list more than the ref of that name:
    /// `git ls-remote` every ref whose name ends with it, and
    /// `git for-each-ref` every ref under it. Those other refs are passed over.
    ///
    /// Gives `None` for output not in the expected form.
    pub fn find(stdout: &'o [u8], name: &str) -> Option<Option<GitObjectId>> {
        if stdout.is_empty() {
            return Some(None);
        }
        let records = stdout.strip_suffix(b"\n")?;
        let mut found = None;
        for record in records.split(|&byte| byte == b'\n') {
            let listed = Self::parse(record)?;
            if listed.name == name.as_bytes() {
                found = Some(listed.id);
            }
        }
        Some(found)
    }

    /// Reads `<object>\t<refname>`, giving `None` for a record not in this
    /// form.
    fn parse(record: &'o [u8]) -> Option<Self> {
        let tab = record.iter().position(|&byte| byte == b'\t')?;
        let (id, name) = (&record[..tab], &record[tab + 1..]);
        let id = GitObjectId::parse(id)?;
        (!name.is_empty()).then_some(Self { id, name })
    }
}

#[cfg(test)]
#[path = "refs_tests.rs"]
mod tests;
