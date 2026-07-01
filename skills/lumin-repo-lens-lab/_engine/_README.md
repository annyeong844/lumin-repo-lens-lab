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
a cross-platform binary bundle. Install or build a package for the
runtime platform, or set a runtime override variable:

- `LUMIN_AUDIT_CORE_BIN_<PLATFORM>_<ARCH>` for one platform
- `LUMIN_AUDIT_CORE_BIN` as a generic external binary override

Those overrides must point to a real audit-core binary for the current
runtime platform. They are the only supported fallback when this
package does not include `_engine/bin/<platform>-<arch>/` for the
current platform.
