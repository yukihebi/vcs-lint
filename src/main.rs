use std::{env, process::ExitCode};

fn main() -> ExitCode {
    vcs_lint::run(env::args().skip(1).collect())
}
