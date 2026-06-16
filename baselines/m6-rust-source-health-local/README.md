# M6 Rust Source Health Local Smoke

This directory records local lab smoke evidence for the M6 Rust source health
vertical slice.

Generated command:

```powershell
node scripts/run-rust-source-health.mjs `
  --root . `
  --output baselines/m6-rust-source-health-local/rust-health.json `
  --rust-source-health-bin experiments/rust-sidecar/rust-source-health/target/release/lumin-rust-source-health.exe `
  --sidecar-source-commit 4a2392f74cd4acf33c7923ec83bb962db2e09b39 `
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
- The M6 wrapper owns file discovery, path policy, hashing, UTF-8 handling,
  final validation, and artifact writes.
- The Rust sidecar receives request JSON on stdin and emits artifact JSON on
  stdout only.
- The wrapper does not traverse excluded `target/**` or `vendor/**` directories.
