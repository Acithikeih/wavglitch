use std::process::ExitCode;
use wavglitch::cli::{Cli, Parser};
use wavglitch::run;
use yansi::{Condition, Paint};

fn main() -> ExitCode {
    yansi::whenever(Condition::TTY_AND_COLOR);
    match run::run(Cli::parse()) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{} {e:#}", "An error occured:".bold().bright().red());
            ExitCode::FAILURE
        }
    }
}
