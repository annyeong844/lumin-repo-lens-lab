# M6 Rust Source Health Local Smoke

This directory records local lab smoke evidence for the M6 Rust source health
vertical slice.

Generated command:

```powershell
cargo run --manifest-path experiments/rust-sidecar/rust-source-health/Cargo.toml --release -- `
  --root . `
  --output baselines/m6-rust-source-health-local/rust-health.json `
  --source-commit dc0ff0378804b0a1f8437b4120493f3cc6e938ea `
  --threads 2 `
  --worker-stack-bytes 16777216
```

Observed summary:

```text
files=12
skippedFiles=0
parseErrorFiles=0
parseErrors=0
signals=16
```

Notes:

- This is local lab evidence, not stable plugin behavior.
- The Rust binary owns file discovery, path policy, hashing, UTF-8 handling,
  final metadata, summary recompute, and artifact writes.
- The same binary still supports request JSON on stdin for compatibility.
- Rust-main mode does not traverse excluded `**/target/**` or `**/vendor/**`
  directories.
