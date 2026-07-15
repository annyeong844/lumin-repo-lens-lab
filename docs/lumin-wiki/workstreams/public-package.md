# Public Package Workstream

Lumin ships as an installable skill/plugin package. Public package work ensures
that behavior verified in the development tree also appears in the installed
package.

## Current Themes

- Version labels must agree across plugin metadata and skill package metadata.
- Public install verification is required before user-visible tracker items
  become `DONE`.
- Private CI may be scarce, so public repo/package verification is part of the
  release workflow.
- Maintainer-only docs must not leak into the public skill surface without an
  explicit packaging spec.

## Test Inventory

| Suite Or Evidence                                 | Risk Type                   | Protected Invariant                                                                                                                                        | Edge Case Or Negative Guard                                                                                           |
| ------------------------------------------------- | --------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| `tests/test-plugin-package.mjs`                   | plugin package contract     | Claude Code plugin package metadata, slash commands, generated skill surfaces, and Codex wrapper opt-in remain coherent.                                   | Generated package must not include lab/private payload or stale metadata.                                             |
| `tests/test-skill-package.mjs`                    | deployable skill package    | Deployable skill package includes plugin wrapper, public scripts, commands, `_engine` internals, canonical/templates/references, and excludes lab payload. | Public skill surface remains small; maintainer docs do not leak unless explicitly packaged.                           |
| `tests/test-skill-surface.mjs`                    | public surface contract     | Shared audit engine and skill surfaces keep stable validation modes and internal-vs-public doc split.                                                      | User-facing prompts must not expose hidden engine modes or internal-only docs.                                        |
| `tests/test-publish-public-plugin.mjs`            | publish pipeline            | Public plugin publisher uses generated package allowlist, changelog prepend, dry-run, and push flow safely.                                                | Dry-run must catch package drift before push; allowlist prevents accidental private file publication.                 |
| `tests/test-github-actions-ci-policy.mjs`         | public CI policy            | GitHub Actions CI policy skips runner jobs for draft PRs while ready/manual/push still run.                                                                | Public CI must avoid burning private runner budget for draft validation while still supporting real release checks.   |
| `tests/test-hook-doctor.mjs`                      | hook runtime manifest       | Hook doctor reports the installed hook manifest, active events, workspace root, audit root, preimage store, and event store.                               | Missing or stale host hook event wiring must fail before runtime tests assume hooks are installed.                    |
| `tests/test-hook-runner-scripts.mjs`              | hook runtime scripts        | Host hook runners connect preimage capture, post-tool reminders, stop ACKs, and prompt drains without noisy malformed-stdin output.                        | Malformed host JSON, missing preimages, or ACK routing drift must remain visible.                                     |
| `tests/test-hook-path-safety.mjs`                 | hook path safety            | Hook tool target paths resolve inside the workspace and reject unsafe syntax.                                                                              | Parent traversal, absolute paths, and backslash paths must not touch disk or escape the repo.                         |
| `tests/test-hook-id-safety.mjs`                   | hook id safety              | Session and tool ids stay compact, deterministic, and content-safe.                                                                                        | Fallback ids must not include raw edited source text or unsafe host metadata.                                         |
| `tests/test-hook-event-store.mjs`                 | hook event store            | Event ledger append, dedupe, acknowledgement, delivery, cleanup, and lock handling remain durable.                                                         | Malformed ledgers, unsafe ids, duplicate events, and stale locks must degrade safely.                                 |
| `tests/test-hook-event-drain-renderer.mjs`        | hook reminder rendering     | Due event drains emit bounded ACK reminders and update delivery metadata.                                                                                  | Aggregate events must not become exact line claims; budget truncation must omit whole event blocks.                   |
| `tests/test-hook-preimage-store.mjs`              | hook preimage privacy       | Preimage capture stores fingerprints and type-escape facts without raw source text.                                                                        | Unsafe ids, malformed records, stale cleanup, and absolute-path spoofing must not leak or delete unrelated records.   |
| `tests/test-hook-ack-observer.mjs`                | hook ACK observer           | Stop-message ACK parsing marks valid events while ignoring code blocks, inline code, invalid intents, and unsafe sessions.                                 | ACK-like text in code blocks or malformed intents must not acknowledge events.                                        |
| `tests/test-hook-post-write-lite.mjs`             | hook lightweight post-write | Post-write-lite compares preimage fingerprints and emits silent-new reminders without running full audit.                                                  | Missing preimages must over-warn rather than report clean success; non-mutating batches must not create event stores. |
| `tests/test-maintainer-scripts.mjs`               | maintainer script hardening | Maintainer scripts surface child-process spawn errors and optional public package reads safely.                                                            | Spawn failures and missing optional package files must not be silently treated as successful publish checks.          |
| `tests/test-audit-manifest-export-surface.mjs`    | public/internal boundary    | Audit manifest exposes stable summary surfaces without living-audit internals.                                                                             | Maintainer-only evidence must not become public package contract by accident.                                         |
| `tests/test-definition-id-export.mjs`             | public/internal boundary    | Definition-id export surface hides raw id builder internals.                                                                                               | Internal identity builders should not leak into public API.                                                           |
| `tests/test-file-delta-export.mjs`                | public/internal boundary    | Post-write file-delta export surface hides path normalizer internals.                                                                                      | Internal path normalizer details should not become package API.                                                       |
| `tests/test-classify-policies-export-surface.mjs` | public/internal boundary    | Classify-policies export surface stays limited to active policy APIs.                                                                                      | Deprecated or internal policy helpers must not become public API.                                                     |
| `tests/test-threshold-policies.mjs`               | policy metadata             | Threshold policy ids, versions, hashes, and compact artifact summaries remain explicit.                                                                    | Magic-number thresholds must not affect public output without named policy metadata.                                  |
| `tests/test-threshold-policy-drift-guard.mjs`     | drift guard                 | Numeric threshold changes require explicit snapshot review.                                                                                                | Threshold tuning must not silently change ranking/rendering behavior.                                                 |
| `tests/test-update-test-doc.mjs`                  | generated docs drift guard  | `tests/README.md` is generated from actual suites and changelog and omits assertion-count authority.                                                       | Hand-edited or stale test docs must fail check mode.                                                                  |
| `tests/test-behavior-corpus-verifier.mjs`         | behavior corpus             | Saved-answer verifier protects no-jargon, caveat, and summary-order behavior.                                                                              | Prompt/user-facing behavior checks stay separate from engine correctness tests.                                       |
| `docs/lab/*public*verification*.md`               | installed package evidence  | Public/package install verification records real installed version behavior.                                                                               | Dev-tree parity is not assumed; installed package evidence is required before `DONE`.                                 |

## Reform Direction

Public-package tests should separate:

- local development engine parity
- package metadata/version drift
- generated package allowlist
- installed plugin behavior
- public CI and draft PR policy

## Reform Targets

- Separate package build tests from installed-package verification notes.
- Keep generated package allowlist checks close to publish tests.
- Keep public/internal boundary tests focused on export surface, not behavior.
- Treat public CI policy as budget and routing evidence, not analyzer
  correctness.
- Do not mark user-visible tracker items `DONE` from dev-tree tests alone when
  package install verification is required.
