# WT-15 beta.42 Public Class Method Pre-Write Verification

Maintainer evidence for `0.9.0-beta.42` public install verification. This note
records the class-heavy fixture check that closed the public verification gap
for WT-15. It is lab evidence, not a user-facing contract.

## Scope

- Feature: WT-15 pre-write class method surface.
- Package: public installed `lumin-repo-lens-lab` beta.42.
- Fixture focus: class-heavy code where the intended reuse candidate is a class
  method, not a top-level function.
- Intent shape: `handleBulkDelete` / delete-domain pre-write lookup.

## Results

| Check | Result | Evidence |
|---|---:|---|
| Installed version is `0.9.0-beta.42` | PASS | `installed_plugins.json` reports `version: "0.9.0-beta.42"` |
| `symbols.meta.supports.classMethodIndex === true` | PASS | Packaged `symbol-graph-artifact.mjs` emits `classMethodIndex: true` |
| `symbols.classMethodIndex` records class methods | PASS | `handleDelete` appears with `className: "TaskControlEventDispatcher"` and `identity: "src/event-dispatcher.ts::TaskControlEventDispatcher#handleDelete"` |
| Pre-write surfaces the class method cue | PASS | `nearNames` matched through `matchedField === "classMethodIndex"` |
| Cue stays review-only | PASS | The method did not enter `defIndex` / dead-export evidence, and the rendered cue stayed in `AGENT_REVIEW_CUE` / `class-method-name` |

## Install / Dev Parity

The verification compared the installed beta.42 engine files against the
maintainer checkout skill mirror for:

- `pre-write-cue-tiers.mjs`
- `pre-write-lookup-name.mjs`
- `symbol-graph-artifact.mjs`

The compared files were byte-identical, so the public install path and the
maintainer test path used the same implementation for the verified surface.

## Conclusion

WT-15 may be treated as `MVP + public verification complete`. It should not be
marked `DONE` yet because the standalone member-index artifact, unavailable
evidence lane, signature-level method facts, broader policy calibration, and
phase timing are still intentionally deferred.
