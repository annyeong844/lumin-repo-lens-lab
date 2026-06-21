use std::io::{self, Read};
use std::process;

use anyhow::{Context, Result};

use crate::protocol::HealthRequest;
use crate::{is_usage_error, usage_error, wrapper};

use super::analysis::analyze_request;
use super::validation::validate_request;

pub fn main_entry() {
    if let Err(error) = run_from_args(std::env::args().skip(1).collect()) {
        eprintln!("{error:#}");
        process::exit(if is_usage_error(&error) { 2 } else { 1 });
    }
}

pub fn run_from_args(args: Vec<String>) -> Result<()> {
    if !args.is_empty() {
        return wrapper::run_cli(args);
    }

    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .context("failed to read stdin")?;
    if input.trim().is_empty() {
        return Err(usage_error("stdin request JSON is required"));
    }

    let request: HealthRequest = serde_json::from_str(&input)
        .map_err(|error| usage_error(format!("failed to parse request JSON: {error}")))?;
    validate_request(&request)?;
    let response = analyze_request(request, Vec::new(), None)?;
    println!("{}", serde_json::to_string(&response)?);
    Ok(())
}
