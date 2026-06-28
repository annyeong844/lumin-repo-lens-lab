#!/usr/bin/env bash
set -euo pipefail

die() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

usage() {
  cat >&2 <<'EOF'
usage: use-basepack.sh <basepack-dir-or-tarball> <check|test|clippy|all> [cargo scope args...]

examples:
  use-basepack.sh /opt/lumin-rust-1.95-offline check --workspace
  use-basepack.sh ./lumin-rust-1.95-offline.tar.zst test -p lumin-rust-source-health
  use-basepack.sh ./lumin-rust-1.95-offline.tar.zst all --workspace

env:
  OFFLINE_RUST_WORKSPACE_DIR   Cargo workspace to run, default: <repo>/experiments
  OFFLINE_RUST_BASEPACK_HOME   extraction directory for tarballs
  OFFLINE_CARGO_TARGET_DIR     target dir, default: <basepack>/target/lumin-repo-lens-lab
EOF
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "required command not found: $1"
}

resolve_dir() {
  [ -d "$1" ] || die "directory not found: $1"
  (cd "$1" && pwd -P)
}

extract_archive() {
  archive="$1"
  dest="${OFFLINE_RUST_BASEPACK_HOME:-$script_dir/.work/extracted}"
  rm -rf "$dest"
  mkdir -p "$dest"

  case "$archive" in
    *.tar.zst | *.tzst)
      if tar --help 2>/dev/null | grep -q -- '--zstd'; then
        tar --zstd -xf "$archive" -C "$dest"
      else
        require_cmd zstd
        zstd -dc "$archive" | tar -xf - -C "$dest"
      fi
      ;;
    *.tar.gz | *.tgz)
      tar -xzf "$archive" -C "$dest"
      ;;
    *.tar)
      tar -xf "$archive" -C "$dest"
      ;;
    *)
      die "unsupported basepack archive: $archive"
      ;;
  esac

  first_dir="$(find "$dest" -mindepth 1 -maxdepth 1 -type d | sort | head -n 1)"
  [ -n "$first_dir" ] || die "archive did not contain a basepack directory"
  resolve_dir "$first_dir"
}

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
repo_root="$(cd -- "$script_dir/../.." && pwd -P)"

if [ "$#" -lt 2 ]; then
  usage
  exit 2
fi

basepack_input="$1"
command_name="$2"
shift 2

case "$command_name" in
  check | test | clippy | all) ;;
  *)
    usage
    die "unknown command: $command_name"
    ;;
esac

require_cmd tar

if [ -d "$basepack_input" ]; then
  basepack_dir="$(resolve_dir "$basepack_input")"
elif [ -f "$basepack_input" ]; then
  basepack_dir="$(extract_archive "$basepack_input")"
else
  die "basepack not found: $basepack_input"
fi

[ -d "$basepack_dir/rustup" ] || die "basepack is missing rustup/: $basepack_dir"
[ -d "$basepack_dir/cargo/bin" ] || die "basepack is missing cargo/bin/: $basepack_dir"
[ -d "$basepack_dir/vendor" ] || die "basepack is missing vendor/: $basepack_dir"

export RUSTUP_HOME="$basepack_dir/rustup"
export CARGO_HOME="$basepack_dir/cargo"
export PATH="$CARGO_HOME/bin:$PATH"
export CARGO_NET_OFFLINE=true
export CARGO_TARGET_DIR="${OFFLINE_CARGO_TARGET_DIR:-$basepack_dir/target/lumin-repo-lens-lab}"

vendor_dir="$(resolve_dir "$basepack_dir/vendor")"
mkdir -p "$CARGO_HOME"
cat > "$CARGO_HOME/config.toml" <<EOF
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "$vendor_dir"

[net]
offline = true
EOF

workspace_dir="${OFFLINE_RUST_WORKSPACE_DIR:-$repo_root/experiments}"
[ -f "$workspace_dir/Cargo.toml" ] || die "Cargo.toml not found in workspace: $workspace_dir"

cargo_scope=("$@")
if [ "${#cargo_scope[@]}" -eq 0 ]; then
  cargo_scope=(--workspace)
fi

run_check() {
  cargo check --offline --locked "${cargo_scope[@]}"
}

run_test() {
  cargo test --offline --locked "${cargo_scope[@]}"
}

run_clippy() {
  cargo clippy --offline --locked --all-targets "${cargo_scope[@]}" -- -D warnings
}

cd "$workspace_dir"
case "$command_name" in
  check) run_check ;;
  test) run_test ;;
  clippy) run_clippy ;;
  all)
    run_check
    run_test
    run_clippy
    ;;
esac
