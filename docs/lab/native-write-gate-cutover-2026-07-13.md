# Native Write-Gate Cutover Verification (2026-07-13)

## Revision And Runtime

- Cutover commit: `2056c6c870ced318597dea1aa26a16a2fed7bebd`
- Packaged wrapper: `skills/lumin-repo-lens-lab/_engine/producers/audit-repo.mjs`
- Windows helper: `skills/lumin-repo-lens-lab/_engine/bin/win32-x64/lumin-audit-core.exe`
- Windows SHA-256: `F55133EC12A60F454909CCAC60AACC6E46AD4CEED991B719739CA040755F3DFD`
- Linux helper: `skills/lumin-repo-lens-lab/_engine/bin/linux-x64/lumin-audit-core`
- Linux SHA-256: `F857534510A1269C96B266C12BC107648969096262AED45A5E2CC284EE24B39C`
- Runtime contract: `audit-core-js-runtime-bridge.v51`
- Required native feature: `nativeJsTsPreWriteLifecycle: true`

The Windows release helper was rebuilt from the cutover commit and passed both
the runtime-contract matcher and the centralized executable fixture probe. The
Linux musl helper was rebuilt after the Linux-specific output cleanup fix and
passed the same probe from its final packaged `/mnt` path.

## Lifecycle Dogfood

The packaged wrapper was run against
`tests/fixtures/canon-drift-helpers-clean` with a fresh output directory and an
invocation-specific advisory handoff.

| Run | Pre-write | Post-write | Result |
| --- | ---: | ---: | --- |
| first valid cold process pair | 2,753 ms | 2,731 ms | exit 0 / exit 0 |
| final rebuilt Windows helper | 634 ms | 594 ms | exit 0 / exit 0 |

Both manifest lifecycle blocks reported `executionOwner: "lumin-audit-core"`.
The pair used `audit-repo.mjs --pre-write` followed by
`audit-repo.mjs --post-write`; no standalone pre-write/post-write script or JS
classification fallback was present.

Dogfood also found and closed two fail-closed contract issues before this
record was written:

- `refactorSources` documentation now states the checked object shape.
- generated post-write invocation IDs now replace both ISO timestamp colons and
  the millisecond dot with filename-safe hyphens.

## Verification

- `cargo lumin-fmt`
- `cargo lumin-clippy`
- `cargo lumin-test`
- packaged source fallback `cargo check --locked -p lumin-audit-core`
- source and packaged Node entrypoint syntax checks
- centralized executable contract fixture for the rebuilt Windows helper
- centralized executable contract fixture for the rebuilt Linux musl helper
- `git diff --check`

Node/Vitest suites were intentionally not run; the removed JS write-gate
implementation is retired rather than authoritative.

## Linux Platform Check

The WSL control plane was restarted after stale interactive sessions stopped
responding. The final helper is a stripped x86-64 static PIE, retains Git mode
`100755`, and passed the centralized contract and executable fixture probes
from `skills/lumin-repo-lens-lab/_engine/bin/linux-x64/lumin-audit-core`.
