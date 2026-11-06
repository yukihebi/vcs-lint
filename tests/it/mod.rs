//! Runs the built binary and compares everything it reports.

mod branch;
mod cli;
mod commit;

use std::{
    ffi::OsStr,
    fmt::{self, Display, Formatter},
    io::Write,
    path::Path,
    process::Command,
};

use tempfile::NamedTempFile;

/// The exit code and output of one run, rendered as a whole for snapshots.
struct Run {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

impl Run {
    fn new(args: &[impl AsRef<OsStr>]) -> Self {
        Self::in_dir(Path::new(env!("CARGO_MANIFEST_DIR")), args)
    }

    fn in_dir(dir: &Path, args: &[impl AsRef<OsStr>]) -> Self {
        let output = Command::new(env!("CARGO_BIN_EXE_vcs-lint"))
            .args(args)
            .current_dir(dir)
            .output()
            .unwrap();
        Self {
            code: output.status.code(),
            stdout: String::from_utf8(output.stdout).unwrap(),
            stderr: String::from_utf8(output.stderr).unwrap(),
        }
    }

    /// Runs `vcs-lint commit` on a file holding `message`.
    fn commit(message: &str, options: &[&str]) -> Self {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(message.as_bytes()).unwrap();
        let path = file.path().to_str().unwrap();
        Self::new(&[&["commit", path], options].concat())
    }
}

impl Display for Run {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.code {
            Some(code) => writeln!(f, "exit: {code}")?,
            None => writeln!(f, "exit: signal")?,
        }
        write!(f, "--- stdout\n{}--- stderr\n{}", self.stdout, self.stderr)
    }
}
