use std::{env, process::ExitCode};

fn main() -> ExitCode {
    vcs_lint::run(env::args_os().skip(1).collect(), env::vars_os()).report()
}
