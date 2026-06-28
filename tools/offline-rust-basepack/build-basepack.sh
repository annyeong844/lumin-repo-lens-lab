#!/usr/bin/env bash
set -euo pipefail

die() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "required command not found: $1"
}

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
repo_root="$(cd -- "$script_dir/../.." && pwd -P)"

rust_version="${RUST_VERSION:-1.95.0}"
target="${RUST_TARGET:-x86_64-unknown-linux-gnu}"
rustup_profile="${RUSTUP_PROFILE:-minimal}"
workspace_dir="${WORKSPACE_DIR:-$repo_root/experiments}"
out_dir="${OUT_DIR:-$script_dir/out}"
work_dir="${WORK_DIR:-$script_dir/.work}"
compression="${COMPRESSION:-zst}"
pack_name="${PACK_NAME:-lumin-rust-${rust_version}-${target}-offline}"

[ -f "$workspace_dir/Cargo.toml" ] || die "Cargo.toml not found in WORKSPACE_DIR: $workspace_dir"
[ -f "$workspace_dir/Cargo.lock" ] || die "Cargo.lock not found in WORKSPACE_DIR: $workspace_dir"
workspace_dir="$(cd -- "$workspace_dir" && pwd -P)"
mkdir -p "$out_dir" "$work_dir"
out_dir="$(cd -- "$out_dir" && pwd -P)"
work_dir="$(cd -- "$work_dir" && pwd -P)"
basepack_dir="$work_dir/$pack_name"

require_cmd curl
require_cmd tar
require_cmd sha256sum

case "$compression" in
  zst) require_cmd zstd ;;
  gz | none) ;;
  *) die "unsupported COMPRESSION '$compression' (use zst, gz, or none)" ;;
esac

rm -rf "$basepack_dir"
mkdir -p "$basepack_dir/rustup" "$basepack_dir/cargo" "$basepack_dir/vendor"

export RUSTUP_HOME="$basepack_dir/rustup"
export CARGO_HOME="$basepack_dir/cargo"
export PATH="$CARGO_HOME/bin:$PATH"

if [ ! -x "$CARGO_HOME/bin/rustup" ]; then
  rustup_init="$work_dir/rustup-init.sh"
  mkdir -p "$work_dir"
  curl --proto '=https' --tlsv1.2 -fsSL https://sh.rustup.rs -o "$rustup_init"
  sh "$rustup_init" -y --no-modify-path \
    --profile "$rustup_profile" \
    --default-host "$target" \
    --default-toolchain none
fi

rustup toolchain install "${rust_version}-${target}" \
  --profile "$rustup_profile" \
  --component rustfmt \
  --component clippy
rustup default "${rust_version}-${target}"

cargo --version
rustc --version
rustfmt --version
cargo clippy --version

(
  cd "$workspace_dir"
  cargo vendor --locked "$basepack_dir/vendor" > "$basepack_dir/cargo-vendor.config.toml"
)

# The vendor directory is the dependency source of truth. Cargo's registry/git
# caches duplicate those bytes and make the portable pack much larger.
rm -rf "$CARGO_HOME/registry" "$CARGO_HOME/git"

cat > "$basepack_dir/cargo-config.template.toml" <<'EOF'
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "__ABSOLUTE_VENDOR_DIR__"

[net]
offline = true
EOF

repo_head="unknown"
if command -v git >/dev/null 2>&1; then
  repo_head="$(git -C "$repo_root" rev-parse --short HEAD 2>/dev/null || printf 'unknown')"
fi

cat > "$basepack_dir/BASEPACK-MANIFEST.txt" <<EOF
name=$pack_name
rust_version=$rust_version
rust_target=$target
workspace_dir=$workspace_dir
repo_head=$repo_head
cargo_lock_sha256=$(sha256sum "$workspace_dir/Cargo.lock" | awk '{print $1}')
created_at_utc=$(date -u '+%Y-%m-%dT%H:%M:%SZ')
contents=rustup,cargo-shims,vendor,cargo-config-template
EOF

case "$compression" in
  zst)
    archive="$out_dir/$pack_name.tar.zst"
    archive_tmp="$archive.tmp.$$"
    rm -f "$archive" "$archive_tmp"
    tar -C "$work_dir" -cf - "$pack_name" | zstd -T0 -6 -o "$archive_tmp" >/dev/null
    mv "$archive_tmp" "$archive"
    ;;
  gz)
    archive="$out_dir/$pack_name.tar.gz"
    archive_tmp="$archive.tmp.$$"
    rm -f "$archive" "$archive_tmp"
    tar -C "$work_dir" -czf "$archive_tmp" "$pack_name"
    mv "$archive_tmp" "$archive"
    ;;
  none)
    archive="$out_dir/$pack_name.tar"
    archive_tmp="$archive.tmp.$$"
    rm -f "$archive" "$archive_tmp"
    tar -C "$work_dir" -cf "$archive_tmp" "$pack_name"
    mv "$archive_tmp" "$archive"
    ;;
esac

printf 'created %s\n' "$archive"
