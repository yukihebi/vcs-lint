//! The directories that the index holds entries in.

use std::path::Path;

/// A directory of the work tree, whose subdirectories are read from the
/// entries of the index under it.
pub struct GitIndexDir<'a> {
    /// The components of its path from the top of the work tree, none for the
    /// top itself.
    components: Vec<&'a [u8]>,
}

impl<'a> GitIndexDir<'a> {
    /// The mode of a gitlink, the entry standing for a submodule.
    const GITLINK_MODE: &[u8] = b"160000";

    /// The directory at `path`, relative to the top of the work tree.
    pub fn new(path: &'a Path) -> Self {
        Self {
            components: path
                .components()
                .map(|component| component.as_os_str().as_encoded_bytes())
                .collect(),
        }
    }

    /// Reads the entries under the directory, as `git ls-files -z -s
    /// --full-name` lists them, giving the names of the directories directly
    /// under it that hold an entry, each once and ordered by their bytes. A
    /// submodule directly under it counts as such a directory.
    ///
    /// Gives `None` for output not in the expected form, including an entry
    /// outside the directory.
    pub fn parse_subdirs<'o>(&self, stdout: &'o [u8]) -> Option<Vec<&'o [u8]>> {
        let records = stdout.strip_suffix(b"\0")?;
        let mut names = Vec::new();
        for record in records.split(|&byte| byte == b'\0') {
            if let Some(name) = self.subdir_of(GitIndexEntry::parse(record)?)? {
                names.push(name);
            }
        }
        names.sort_unstable();
        names.dedup();
        Some(names)
    }

    /// The name of the directory directly under the directory that holds
    /// `entry`, `Some(None)` for an entry not in such a directory, or `None`
    /// for an entry outside the directory.
    fn subdir_of<'o>(&self, entry: GitIndexEntry<'o>) -> Option<Option<&'o [u8]>> {
        let mut rest = entry.path.split(|&byte| byte == b'/');
        for component in &self.components {
            if rest.next()? != *component {
                return None;
            }
        }
        let rest: Vec<_> = rest.collect();
        Some(match rest.as_slice() {
            [name, _, ..] => Some(*name),
            [name] if entry.mode == Self::GITLINK_MODE => Some(*name),
            _ => None,
        })
    }
}

/// An entry of the index, as `git ls-files -s` lists it.
struct GitIndexEntry<'o> {
    mode: &'o [u8],
    /// Its path from the top of the work tree, with components separated by
    /// `/`.
    path: &'o [u8],
}

impl<'o> GitIndexEntry<'o> {
    /// Reads `<mode> <object> <stage>\t<path>`, giving `None` for a record not
    /// in this form.
    fn parse(record: &'o [u8]) -> Option<Self> {
        let tab = record.iter().position(|&byte| byte == b'\t')?;
        let (meta, path) = (&record[..tab], &record[tab + 1..]);
        let mode = meta.split(|&byte| byte == b' ').next()?;
        (!mode.is_empty() && !path.is_empty()).then_some(Self { mode, path })
    }
}

#[cfg(test)]
#[path = "index_tests.rs"]
mod tests;
