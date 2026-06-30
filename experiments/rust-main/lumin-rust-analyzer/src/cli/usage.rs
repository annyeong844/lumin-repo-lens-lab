pub(super) fn print_analyze() {
    eprintln!(
        "Usage: lumin-rust-analyzer --root <path> --source-commit <sha> [--output <path>] [--cargo-bin <path>] [--features <csv>] [--package <name>] [--repo-root <path>] [--semantic-mode metadata-only|cargo-check|targeted-cargo-check] [--cargo-target-dir-mode isolated-temp|reusable-temp] [--calibration-adjudication <path>] [--source-health-profile compact|full] [--cache-root <path>] [--no-incremental] [--clear-incremental-cache] [--cargo-check] [--targeted-cargo-check] [--threads <n>] [--worker-stack-bytes <bytes>]"
    );
}

pub(super) fn print_pre_write() {
    eprintln!(
        "Usage: lumin-rust-analyzer pre-write --root <path> --source-commit <sha> --intent <path> [--output <path>] [--threads <n>] [--worker-stack-bytes <bytes>]"
    );
}
