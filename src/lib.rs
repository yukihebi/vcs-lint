use std::process::ExitCode;

mod rules;

pub fn run(args: Vec<String>) -> ExitCode {
    let _ = args;
    ExitCode::FAILURE
}
