# WT-23 Beta 48 Service-Operation Policy Verification

Date: 2026-05-13

## Scope

Public install verification for the WT-23 P1 `serviceOperationSiblingPolicy`
slice.

## Install Evidence

- `installed_plugins.json`: `0.9.0-beta.48`
- Install `plugin.json`, `marketplace.json`, and skill `package.json`:
  `0.9.0-beta.48`
- Install/dev engine parity: content equivalent after line-ending
  normalization.
- Maintainer commit under test: `ce2a42b` (`Add WT-23 service operation
  sibling policy`)

## Fixture Results

| Intent | Candidate | Result |
|---|---|---|
| `searchUser` | `fetchUser` | PASS: `serviceOperationSiblingPolicy.promoted[]` includes `fetchUser` with `operationFamily: "read-query"` and locality evidence; `nearNames`, `semanticHints`, and cue cards remain empty. |
| `createUser` | `fetchUser` | PASS: no promotion; `muted[]` records `service-sibling-operation-family-mismatch`. |
| `searchPost` | `fetchUser` | PASS as noise-floor behavior: no promotion and no evaluated policy candidate when no suppressed evidence admits `fetchUser` into the policy input set. |

## Interpretation

The P1 evaluator consumes existing suppressed evidence; it does not perform a
second broad scan for every function-like name. Therefore domain-mismatch muted
entries are expected only for candidates that already appear in
`suppressedNearNames[]` or `suppressedSemanticHints[]`.

If `searchPost` and `fetchUser` share no supported name/domain signal, `fetchUser`
may remain absent from both `promoted[]` and `muted[]`. That is the intended
noise-floor behavior for P1, not a failed mute.

## Verdict

WT-23 P1 public install verification passed for beta.48. Cue-card rendering,
renderer wording, mutation-family policy, signature compatibility, and broader
corpus calibration remain follow-up work.
