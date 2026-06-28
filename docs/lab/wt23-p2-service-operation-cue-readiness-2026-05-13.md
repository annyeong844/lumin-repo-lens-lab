# WT-23 P2 Service Operation Cue Readiness - 2026-05-13

## Purpose

WT-23 P1 added `serviceOperationSiblingPolicy` as a pure evidence object. P2
should render selected `promoted[]` entries as review cues only after the
fixture and corpus gates below pass.

This note is a pre-implementation checklist. It does not change analyzer
behavior and does not justify a package bump by itself.

## Current Baseline

- beta.47 verified suppressed near-name and semantic diagnostics.
- beta.48 verified the P1 service-operation sibling policy object.
- `searchUser` -> `fetchUser` can be promoted inside
  `serviceOperationSiblingPolicy.promoted[]`.
- Formal `nearNames[]`, `semanticHints[]`, and `cueCards[]` remain unchanged in
  P1.
- `searchPost` -> `fetchUser` is noise-floor behavior when no suppressed
  evidence admits `fetchUser` into the policy input set.

## Fixture Matrix

| Fixture | Required P2 Result | Guard |
|---|---|---|
| `searchUser` -> `fetchUser`, same service file or directory | exactly one `AGENT_REVIEW_CUE` in `service-operation-sibling` | no `SAFE_CUE`, `EXISTS`, or `SAFE_FIX` |
| `lookupUser` -> `findUser`, same service file or directory | exactly one `AGENT_REVIEW_CUE` in `service-operation-sibling` | no threshold relaxation |
| `createUser` -> `fetchUser` | muted only | no cue card |
| `deleteUser` -> `removeUser` | muted in v1 | no mutation-family cue |
| `searchPost` -> `fetchUser`, no suppressed input evidence | absent from policy | no synthetic mute entry |
| `searchPost` -> `fetchUser`, synthetic supporting signal admits candidate | muted with `service-sibling-domain-mismatch` | no cue card |
| generated, bundled, vendor, scaffold, or framework-resource candidate | muted | no cue card |
| `classMethodIndex` candidate | handled by `class-method-name` lane | no service-operation sibling cue |

## Renderer Contract

Allowed default wording:

```text
Review related service operation: fetchUser in src/services/user.ts.
```

Disallowed default wording:

```text
reuse
equivalent
safe
exists
should call
blocking failure
```

The rendered cue must reference `pre-write-advisory.json`,
`lookups[].serviceOperationSiblingPolicy.promoted`, `policyId`,
`policyVersion`, candidate identity, operation family, shared domain tokens,
locality, and supporting suppressed reasons.

The cue-tier adapter must copy the policy result. It must not recompute
operation families, domain tokens, locality, or signature compatibility.

## Corpus Gate

Before P2 is enabled by default, run at least:

- one service-heavy app corpus;
- one library or noise-heavy corpus.

Record:

- promoted count;
- muted count;
- suppressed count;
- top promoted candidate names;
- generated/vendor/framework suppressed count;
- reviewed false positives;
- any missing corpus and why it was unavailable.

If the corpus gate is not complete, P2 implementation may exist behind a
review-only development path, but it should not become the default public
pre-write rendering behavior.

## Decision

Proceed next with fixture-first P2 implementation only if this matrix is added
as tests before renderer behavior changes. Keep mutation-family cues and
signature-weighted promotion for later calibration.
