# M6 Rust Source Health Local Smoke

This directory records local lab smoke evidence for the M6 Rust source health
vertical slice.

Generated command:

```powershell
node scripts/run-rust-source-health.mjs `
  --root . `
  --output baselines/m6-rust-source-health-local/rust-health.json `
  --rust-source-health-bin experiments/rust-sidecar/rust-source-health/target/release/lumin-rust-source-health.exe `
  --sidecar-source-commit a76bc31ae45ea0aa361df25b3b024e20e77af5ed `
  --threads 2 `
  --worker-stack-bytes 16777216
```

Observed summary:

```text
files=12
skippedFiles=4
parseErrorFiles=0
parseErrors=0
signals=16
```

Notes:

- This is local lab evidence, not stable plugin behavior.
- The M6 wrapper owns file discovery, path policy, hashing, UTF-8 handling,
  final validation, and artifact writes.
- The Rust sidecar receives request JSON on stdin and emits artifact JSON on
  stdout only.
- Skipped files in this smoke are excluded `target/**` Rust files produced by
  local Cargo builds.

