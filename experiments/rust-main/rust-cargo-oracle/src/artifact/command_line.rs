pub(super) fn command_args(cargo_bin: &str, cargo_args: &[String]) -> Vec<String> {
    let mut out = vec![cargo_bin.to_string()];
    out.extend(cargo_args.iter().cloned());
    out
}
