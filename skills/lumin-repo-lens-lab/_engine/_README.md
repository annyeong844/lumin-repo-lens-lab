# Internal Engine

This directory is packaged with the skill because the public
`scripts/*.mjs` wrappers need it at runtime.

Files under `_engine/` are internal implementation details. They
are not a stable user-facing API; use `scripts/audit-repo.mjs` or
the other public wrappers instead.

`_engine/bin/<platform>-<arch>/` contains the packaged audit-core
binary for each platform supplied at package build time. The current
build platform is rebuilt before packaging so stale CLI commands are
not copied. Additional platform binaries can be supplied with
`LUMIN_AUDIT_CORE_BIN_<PLATFORM>_<ARCH>`.

The package also carries a minimal `_engine/rust` Cargo workspace for
`lumin-audit-core`. If no matching packaged/env/PATH binary exists and
Cargo is available, the runtime wrapper builds that helper for the
current platform before invoking it.

If Cargo is not available, set a runtime override variable:

- `LUMIN_AUDIT_CORE_BIN_<PLATFORM>_<ARCH>` for one platform
- `LUMIN_AUDIT_CORE_BIN` as a generic external binary override
- `lumin-audit-core` / `lumin-audit-core.exe` on `PATH`

Override binaries must match the current runtime platform. They
are supported when this package does not include
`_engine/bin/<platform>-<arch>/` for the current platform.

When the wrapper is running from a source checkout that still has
`experiments/Cargo.toml`, it can also build the current-platform helper
from that checkout if no matching packaged/env/PATH/package-source
binary exists. Set
`LUMIN_AUDIT_CORE_NO_AUTO_BUILD=1` to disable that source-checkout
fallback and fail fast instead.
