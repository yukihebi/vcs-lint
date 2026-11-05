use std::process::ExitCode;

mod branch;
mod commit;
mod rules;
mod violations;

pub fn run(args: Vec<String>) -> ExitCode {
    let _ = args;
    ExitCode::FAILURE
}
