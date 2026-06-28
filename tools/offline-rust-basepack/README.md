# Offline Rust Basepack

This builds a portable Linux Rust basepack for offline validation of the Rust migration workspace.

The current `experiments/Cargo.lock` requires Rust 1.95 because `ra_ap_syntax = 0.0.337`
declares `rust-version = "1.95"`. Use `RUST_VERSION=1.85.0` only for an older lockfile
whose dependencies still support 1.85.

The pack contains:

- Rustup toolchain state for `x86_64-unknown-linux-gnu`
- Cargo/rustc/rustfmt/clippy shims
- A vendored dependency tree produced from the workspace `Cargo.lock`
- A Cargo source-replacement config template

It does not contain `target/` build output.

## Build On An Online Linux Host

Run this from the repo root on a Linux `x86_64-unknown-linux-gnu` host with network access:

```bash
RUST_VERSION=1.95.0 bash tools/offline-rust-basepack/build-basepack.sh
```

The default output is:

```text
tools/offline-rust-basepack/out/lumin-rust-1.95.0-x86_64-unknown-linux-gnu-offline.tar.zst
```

Useful knobs:

```bash
RUST_VERSION=1.95.0 \
RUST_TARGET=x86_64-unknown-linux-gnu \
WORKSPACE_DIR=/path/to/lumin-repo-lens-lab/experiments \
COMPRESSION=zst \
bash tools/offline-rust-basepack/build-basepack.sh
```

`WORKSPACE_DIR` is the Cargo workspace whose `Cargo.lock` drives `cargo vendor --locked`.
For a different project, build a separate pack from that project's lockfile.

## Use In The Offline Sandbox

Copy the tarball into the sandbox, then run:

```bash
bash tools/offline-rust-basepack/use-basepack.sh /path/to/lumin-rust-1.95.0-x86_64-unknown-linux-gnu-offline.tar.zst check --workspace
bash tools/offline-rust-basepack/use-basepack.sh /path/to/lumin-rust-1.95.0-x86_64-unknown-linux-gnu-offline.tar.zst test --workspace
bash tools/offline-rust-basepack/use-basepack.sh /path/to/lumin-rust-1.95.0-x86_64-unknown-linux-gnu-offline.tar.zst clippy --workspace
```

Or run all three:

```bash
bash tools/offline-rust-basepack/use-basepack.sh /path/to/lumin-rust-1.95.0-x86_64-unknown-linux-gnu-offline.tar.zst all --workspace
```

Package-scoped examples:

```bash
bash tools/offline-rust-basepack/use-basepack.sh /path/to/basepack.tar.zst check -p lumin-rust-source-health
bash tools/offline-rust-basepack/use-basepack.sh /path/to/basepack.tar.zst test -p lumin-rust-analyzer
```

The script sets:

```bash
export RUSTUP_HOME=/path/to/basepack/rustup
export CARGO_HOME=/path/to/basepack/cargo
export PATH="$CARGO_HOME/bin:$PATH"
export CARGO_NET_OFFLINE=true
export CARGO_TARGET_DIR=/path/to/basepack/target/lumin-repo-lens-lab
```

If the extracted basepack is read-only, set:

```bash
export OFFLINE_CARGO_TARGET_DIR=/writable/cache/lumin-target
```

## Cargo Config

The sandbox script writes this config into the basepack `CARGO_HOME` with an absolute vendor path:

```toml
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "/absolute/path/to/basepack/vendor"

[net]
offline = true
```

This keeps the repo's own `.cargo/config.toml` untouched.

## Boundaries

- The pack is lockfile-specific. If `Cargo.lock` changes, rebuild the pack.
- The pack is native Linux, not Wasm.
- The pack intentionally uses vendored sources instead of a duplicated Cargo registry cache.
- The pack should not be committed. `out/` and `.work/` are ignored locally in this tool directory.
