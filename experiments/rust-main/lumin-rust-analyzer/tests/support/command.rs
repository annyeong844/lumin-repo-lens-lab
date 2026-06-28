use std::process::Command;

pub fn unified_analyzer_command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_lumin-rust-analyzer"))
}
