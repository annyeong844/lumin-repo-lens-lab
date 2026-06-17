use std::env;
use std::path::{Path, PathBuf};
use std::process;

use anyhow::{bail, Context, Result};
use lumin_rust_cargo_oracle::{run_oracle, OracleOptions};

fn main() {
    match parse_args().and_then(run_oracle) {
        Ok(artifact) => {
            if let Some(output) = artifact
                .get("meta")
                .and_then(|meta| meta.get("output"))
                .and_then(|output| output.as_str())
            {
                println!("[rust-cargo-oracle] wrote {output}");
            } else {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&artifact)
                        .unwrap_or_else(|_| artifact.to_string())
                );
            }
        }
        Err(error) => {
            eprintln!("{error:#}");
            process::exit(if is_usage_error(&error) { 2 } else { 1 });
        }
    }
}

fn parse_args() -> Result<OracleOptions> {
    let mut root: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut cargo_bin = "cargo".to_string();
    let mut timeout_ms = 60_000_u64;
    let mut features: Option<String> = None;
    let mut package_name: Option<String> = None;
    let mut repo_root: Option<PathBuf> = None;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--root" => root = Some(value_path(&mut args, "--root")?),
            "--output" => output = Some(value_path(&mut args, "--output")?),
            "--cargo-bin" => cargo_bin = value_string(&mut args, "--cargo-bin")?,
            "--timeout-ms" => {
                let value = value_string(&mut args, "--timeout-ms")?;
                timeout_ms = value
                    .parse::<u64>()
                    .with_context(|| format!("invalid --timeout-ms value: {value}"))?;
            }
            "--features" => features = Some(value_string(&mut args, "--features")?),
            "--package" => package_name = Some(value_string(&mut args, "--package")?),
            "--repo-root" => repo_root = Some(value_path(&mut args, "--repo-root")?),
            "--help" | "-h" => {
                print_usage();
                process::exit(0);
            }
            unknown => bail!("unknown argument: {unknown}"),
        }
    }

    let root = root.unwrap_or(env::current_dir().context("failed to read current directory")?);
    let output = output.unwrap_or_else(|| root.join("semantic-health.json"));
    let repo_root = match repo_root {
        Some(path) => path,
        None => find_repo_root(&root).unwrap_or_else(|| PathBuf::from(".")),
    };

    Ok(OracleOptions {
        root,
        output: Some(output),
        cargo_bin,
        timeout_ms,
        features,
        package_name,
        repo_root,
    })
}

fn value_string(args: &mut impl Iterator<Item = String>, name: &str) -> Result<String> {
    args.next()
        .filter(|value| !value.trim().is_empty())
        .with_context(|| format!("{name} requires a value"))
}

fn value_path(args: &mut impl Iterator<Item = String>, name: &str) -> Result<PathBuf> {
    Ok(PathBuf::from(value_string(args, name)?))
}

fn find_repo_root(start: &Path) -> Option<PathBuf> {
    let mut cursor = if start.is_file() {
        start.parent()?.to_path_buf()
    } else {
        start.to_path_buf()
    };
    loop {
        if cursor
            .join("canonical")
            .join("oracle-registry.json")
            .is_file()
        {
            return Some(cursor);
        }
        if !cursor.pop() {
            return None;
        }
    }
}

fn is_usage_error(error: &anyhow::Error) -> bool {
    let message = error.to_string();
    message.starts_with("unknown argument:")
        || message.contains(" requires a value")
        || message.starts_with("invalid --timeout-ms")
        || message.starts_with("--package currently supports")
}

fn print_usage() {
    eprintln!(
        "Usage: lumin-rust-cargo-oracle --root <path> [--output <path>] [--cargo-bin <path>] [--timeout-ms <ms>] [--features <csv>] [--package <name>] [--repo-root <path>]"
    );
}
