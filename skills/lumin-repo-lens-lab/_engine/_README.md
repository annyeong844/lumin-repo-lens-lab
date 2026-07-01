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

A package built with only one platform binary is platform-scoped, not
a cross-platform binary bundle. Runtime override variables
`LUMIN_AUDIT_CORE_BIN_<PLATFORM>_<ARCH>` and `LUMIN_AUDIT_CORE_BIN`
can point to an external audit-core binary when this package does not
include one for the current platform.
