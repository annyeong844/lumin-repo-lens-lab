use std::process;

use lumin_rust_cargo_oracle::{parse_args, protocol::SemanticHealthArtifact, run_oracle};
use lumin_rust_common::{is_usage_error, CliAction};

fn main() {
    match parse_args() {
        Ok(CliAction::Run(options)) => match run_oracle(options) {
            Ok(artifact) => print_artifact(artifact),
            Err(error) => exit_with_error(error),
        },
        Ok(CliAction::Help) => {}
        Err(error) => exit_with_error(error),
    }
}

fn print_artifact(artifact: SemanticHealthArtifact) {
    if let Some(output) = artifact.meta.output.as_deref() {
        println!("[rust-cargo-oracle] wrote {output}");
    } else {
        match serde_json::to_string_pretty(&artifact) {
            Ok(json) => println!("{json}"),
            Err(error) => {
                eprintln!("failed to serialize semantic health artifact: {error:#}");
                process::exit(1);
            }
        }
    }
}

fn exit_with_error(error: anyhow::Error) -> ! {
    eprintln!("{error:#}");
    process::exit(if is_usage_error(&error) { 2 } else { 1 });
}
