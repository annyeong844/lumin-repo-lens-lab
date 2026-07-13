# Native Write-Gate Cutover Verification (2026-07-13)

## Revision And Runtime

- Cutover commit: `2056c6c870ced318597dea1aa26a16a2fed7bebd`
- Packaged wrapper: `skills/lumin-repo-lens-lab/_engine/producers/audit-repo.mjs`
- Windows helper: `skills/lumin-repo-lens-lab/_engine/bin/win32-x64/lumin-audit-core.exe`
- Windows SHA-256: `F55133EC12A60F454909CCAC60AACC6E46AD4CEED991B719739CA040755F3DFD`
- Linux helper: `skills/lumin-repo-lens-lab/_engine/bin/linux-x64/lumin-audit-core`
- Linux SHA-256: `78F431DCF6C717AC2212F2929CA3CA464D059429C8A46D590E8E74DB4E8E654E`
- Runtime contract: `audit-core-js-runtime-bridge.v51`
- Required native feature: `nativeJsTsPreWriteLifecycle: true`

The Windows release helper was rebuilt from the cutover commit and passed both
the runtime-contract matcher and the centralized executable fixture probe. The
Linux musl helper was rebuilt for v51 and passed the same probe before the final
clippy-only source cleanup.

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
- `git diff --check`

Node/Vitest suites were intentionally not run; the removed JS write-gate
implementation is retired rather than authoritative.

## Linux Follow-up Boundary

The local WSL control plane stopped responding even to `printf` after the
successful Linux v51 build and fixture probe. Existing unrelated interactive
WSL sessions were not force-terminated. Therefore the committed Linux helper's
hash and prior v51 fixture result are recorded here, while the public Linux CI
run remains the final platform check for this revision.
