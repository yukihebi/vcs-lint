//! Runs each command line both through the library, to see how the run ends,
//! and through the built binary, to compare everything it reports.

/// Asserts that the library ended `run` as `pattern`.
macro_rules! assert_status {
    ($run:expr, $pattern:pat) => {
        assert!(matches!($run.status, $pattern), "{:?}", $run.status)
    };
}

mod branch;
mod cli;
mod commit;
mod config_file;
mod history;
mod list_dir;
mod required_version;
mod severity;
mod workdir;

use std::{
    env,
    ffi::{OsStr, OsString},
    fmt::{self, Display, Formatter},
    io::Write,
    process::Command,
};

use tempfile::NamedTempFile;
use vcs_lint::RunStatus;

/// How one run ends, through the library and through the binary.
struct Run {
    status: RunStatus,
    output: BinaryOutput,
}

impl Run {
    /// Runs `args` in the current directory, where relative paths resolve the
    /// same way for the library and the binary.
    fn new(args: &[impl AsRef<OsStr>]) -> Self {
        Self::with_vars(args, &[])
    }

    /// Runs `args` as [`Run::new`] does, with `vars` as the only environment
    /// variables of vcs-lint set.
    fn with_vars(args: &[impl AsRef<OsStr>], vars: &[(&str, &str)]) -> Self {
        let mut command = Command::new(BinaryOutput::PROGRAM);
        for (name, _) in env::vars_os() {
            if name.to_string_lossy().starts_with("VCS_LINT_") {
                command.env_remove(name);
            }
        }
        let output = BinaryOutput::of(command.args(args).envs(vars.iter().copied()));
        Self {
            status: vcs_lint::run(
                args.iter().map(|arg| arg.as_ref().to_owned()).collect(),
                |name| {
                    vars.iter()
                        .find(|&&(set, _)| set == name)
                        .map(|&(_, value)| OsString::from(value))
                },
            ),
            output,
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
        self.output.fmt(f)
    }
}

/// The exit code and output of the binary, rendered as a whole for snapshots.
struct BinaryOutput {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

impl BinaryOutput {
    const PROGRAM: &str = env!("CARGO_BIN_EXE_vcs-lint");

    /// Runs `command`, which runs the binary, to completion.
    fn of(command: &mut Command) -> Self {
        let output = command.output().unwrap();
        Self {
            code: output.status.code(),
            stdout: String::from_utf8(output.stdout).unwrap(),
            stderr: String::from_utf8(output.stderr).unwrap(),
        }
    }
}

impl Display for BinaryOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.code {
            Some(code) => writeln!(f, "exit: {code}")?,
            None => writeln!(f, "exit: signal")?,
        }
        write!(f, "--- stdout\n{}--- stderr\n{}", self.stdout, self.stderr)
    }
}
