# M5 Rust Topology Prefer Closeout - 2026-06-16

## Scope

This note closes the M5 lab-only Rust topology `prefer` package pass. It records
evidence only; it does not approve default Rust, stable
`/lumin-repo-lens:*`, or broad Rust replacement claims.

## Commits

- Source-side implementation commit: `90f271e` (`Harden Rust topology prefer gate`)
- Public package PR: `annyeong844/lumin-repo-lens-lab#2`
- Public package head commit: `6864890`
- Public package merge commit: `bdd5c3f`

## Public Package CI

- Workflow: `Public Package CI`
- Run: `27593629018`
- Head SHA: `bdd5c3fe2cb7cd68411c3b6f2c471b3e8ff17640`
- Result: success
- Jobs:
  - `Package Smoke (Node 20.x)`: success
  - `Package Smoke (Node 22.x)`: success

## Local Cache Verification

Cache refreshed from `public-lab/main`:

```text
C:\Users\endof\.claude\plugins\cache\annyeong844-lumin-lab-marketplace\lumin-repo-lens-lab\0.0.0-lab.0
```

Verified package runtime files match `public-lab/main` by Git blob hash:

- `skills/lumin-repo-lens-lab/_engine/lib/rust-topology-prefer.mjs`
- `skills/lumin-repo-lens-lab/_engine/lib/rust-topology-prefer-gate.mjs`
- `skills/lumin-repo-lens-lab/_engine/lib/rust-topology-quorum.mjs`
- `skills/lumin-repo-lens-lab/_engine/producers/measure-topology.mjs`

Local checks:

- `node --check` passed for the four package runtime files above.
- `node scripts/smoke-test.mjs` passed from the cached skill directory.
- `node skills/lumin-repo-lens-lab/scripts/audit-repo.mjs --help` passed.
- Hook runner smoke passed for `pre-tool-use`, `post-tool-batch`, and `stop`.

Local note: `npm run smoke` failed in this Codex shell because the npm script
shell could not resolve `node` from PATH. Direct `node scripts/smoke-test.mjs`
passed, and Public Package CI proved `npm run smoke` on Node 20 and Node 22.

## Current Contract

- `prefer` remains explicit opt-in only.
- `prefer` is run-level only.
- `off` and `compare` stay available as rollback paths.
- Blocked `prefer` writes diagnostic `topology.json` and exits non-zero.
- Quorum evidence is fail-closed for schema, policy, source commit, binary SHA,
  latest-three history, clean-source diagnostics, and full scanner coverage.
- Packaged skill `prefer` should receive an explicit
  `--rust-topology-prefer-quorum` path; package-local baseline files are not an
  implicit approval stamp.

## Accepted Follow-Ups

Implemented after the package closeout:

- malformed quorum evidence becomes visible blocked metadata instead of an
  artifact-less hard crash;
- sidecar failure statuses retain their failure classification even when failure
  metadata lacks `policyVersion`;
- packaged skill quorum behavior is documented as explicit-path driven.
