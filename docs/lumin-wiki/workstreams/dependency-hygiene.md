# Dependency Hygiene Workstream

Dependency hygiene work compares package manifest declarations against observed
evidence. It must stay review-first: a declaration with no observed consumer is
not automatically removable.

## Current Themes

- `unused-deps.json` is the first dependency hygiene artifact.
- The artifact is review-only and must not create package edits, fix-plan
  entries, SARIF warnings, or `SAFE_FIX` claims.
- Runtime package-script entry evidence is a prerequisite. Tools such as `tsx`
  can explain dependencies that never appear as imports.
- Monorepo package ownership matters. A root package must not claim imports from
  child workspace packages unless the root is the nearest package root.
- Unsupported or missing dependency-consumer evidence must produce
  `status: "unavailable"` or confidence-limited review evidence, not false
  absence.
- Public skill installs are lazy. The plugin cache is not expected to contain
  `node_modules` until first run; the dependency guard installs runtime parser
  packages in `skills/lumin-repo-lens-lab` or, when auto-install is disabled,
  prints the exact setup command.

## Test Inventory

| Suite Or Evidence                                                                               | Risk Type                         | Protected Invariant                                                                                                        | Edge Case Or Negative Guard                                                                                                                                                 |
| ----------------------------------------------------------------------------------------------- | --------------------------------- | -------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `tests/test-unused-deps-producer.mjs`                                                           | review-only dependency artifact   | Declared dependencies classify as `used`, `muted`, `review-unused`, or unavailable without action claims.                  | Package script tools, `@types/*`, workspace internals, unsupported symbols lanes, and nearest package ownership must not become removal proof.                              |
| `tests/unused-deps-producer.test.mjs`                                                           | Vitest mirror                     | The focused mirror preserves the same helper and audit pipeline contract as the Node suite.                                | The mirror must keep Node entrypoint parity and must not weaken the review-only boundary.                                                                                   |
| beta.57 public install verification                                                             | installed package evidence        | Installed package emits `unused-deps.json` with `unused-deps.v1` and `unused-deps-review-policy-v1`.                       | `fix-plan.json`, `export-action-safety.json`, SARIF, summary Markdown, and review-pack Markdown must not surface deletion or `SAFE_FIX` claims.                             |
| beta.58 public install verification                                                             | installed review-surface evidence | Installed package surfaces dependency hygiene counts in `manifest.json`, summary Markdown, and review-pack Markdown.       | Package names stay out of Markdown, strong package-edit wording is absent, and no dependency hygiene evidence leaks into fix-plan, action-safety, SARIF, or safe cue lanes. |
| beta.56 runtime package script verification                                                     | prerequisite entry evidence       | Package script runtime tools such as `tsx src/server.ts` enter entry-surface evidence.                                     | Later positional arguments are script argv, not additional runtime entry files.                                                                                             |
| `docs/spec/unused-deps-producer.md`                                                             | design contract                   | Dependency hygiene starts as artifact-only review evidence.                                                                | P2 summary/review-pack surfacing requires a separate wording and leakage review.                                                                                            |
| `docs/spec/unused-deps-review-surface.md`                                                       | review-surface design contract    | P2/P3 manifest, summary, and review-pack surfacing stays review-only.                                                      | Markdown wording, manifest mirrors, and review-pack lines must not leak into package edits, fix-plan, SARIF, action-safety, or safe cue lanes.                              |
| `tests/test-audit-manifest-export-surface.mjs` / `tests/audit-manifest-export-surface.test.mjs` | manifest mirror guard             | `manifest.json.unusedDependencies` mirrors shallow `unused-deps.json` status, counts, reasons, and capped review examples. | Unavailable dependency hygiene evidence must stay explicit and must not become a false zero-candidate claim.                                                                |
| `tests/test-audit-repo.mjs` / `tests/audit-repo-artifact-brief.test.mjs`                        | Markdown review surface           | `audit-summary.latest.md` and `audit-review-pack.latest.md` surface dependency hygiene as review-only counts.              | Markdown must point to `manifest.json.unusedDependencies` and `unused-deps.json` without package-name examples or package-edit action wording.                              |

## Reform Direction

Dependency hygiene tests should separate these claims:

- manifest declaration collection
- package identity normalization
- package-script tool evidence
- workspace package ownership
- unsupported or missing consumer evidence
- review-only output boundaries

Do not combine dependency hygiene with package edits or automated removal. The
safe first question is "what evidence did we observe?", not "what should the
package manager remove?"

## Reform Targets

- Keep `unused-deps.json` artifact tests focused on classification and
  provenance.
- Keep `manifest.json.unusedDependencies` shallow: counts and capped examples
  are navigation evidence, not ranking or package-edit instructions.
- Keep summary/review-pack wording weak: counts and artifact paths only, with
  package names staying in JSON evidence unless a later slice pins example
  wording.
- Add configuration or allowlist support only after corpus data shows the
  review artifact is too noisy without it.
- Keep public install verification required for user-visible dependency hygiene
  behavior.
- Treat missing install-cache `node_modules` as expected lazy-install state, not
  as a publish failure, unless the installed wrapper cannot either install
  runtime dependencies or emit the manual setup command.
