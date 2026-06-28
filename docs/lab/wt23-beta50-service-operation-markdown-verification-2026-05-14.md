# WT-23 Beta 50 Service-Operation Markdown Verification

Date: 2026-05-14

## Scope

Public install verification for the WT-23 P2b
`service-operation-sibling` Markdown renderer slice.

P2b sits on top of the P2a JSON cue-tier adapter. It changes the default
Markdown wording for already-promoted service-operation sibling cue cards; it
does not change the policy evaluator or the cue-tier adapter.

## Install Evidence

- Installed package version: `0.9.0-beta.50`
- Private PR: `#253`
- Private merge commit: `5bc8320`
- Implementation commit: `e637507`
- Public package commit: `02edb2ba3f84a8efee44823c94d55f7ac8e742cc`
- Public CI: success,
  `https://github.com/annyeong844/lumin-repo-lens-lab/actions/runs/25808009189`
- Installed renderer path:
  `0.9.0-beta.50/skills/lumin-repo-lens-lab/_engine/lib/pre-write-render.mjs`

Install/dev renderer parity was content-equivalent for execution code. The
only observed diff between maintainer `_lib/pre-write-render.mjs` and the
installed skill mirror was in comment-only maintainer-history wording. The
cue-tier adapter was unchanged from P2a.

## Fixture Output

The installed beta.50 renderer produced the expected service-operation review
cue:

```md
### Agent review cues

- Review related service operation: `fetchUser` in `src/services/user.ts`.
  [heuristic-review, pre-write-advisory.json / lookups[].serviceOperationSiblingPolicy.promoted; cueTier=AGENT_REVIEW_CUE]
  policy prewrite-service-operation-sibling-cue-v1
  shared domain tokens: `user`; operation family: `read-query`; locality: sameDir.
  supporting suppressed reasons: `near-distance-exceeded`, `single-non-weak-token-only`.
  action: inspect this related operation before creating parallel service code.
```

## Verification Matrix

| Requirement | Result |
|---|---|
| Markdown starts with `Review related service operation: <name> in <ownerFile>.` | PASS |
| Shared domain tokens, operation family, and locality are rendered | PASS |
| Supporting suppressed reasons are rendered | PASS |
| `policyVersion` is rendered | PASS |
| Evidence path cites `pre-write-advisory.json / lookups[].serviceOperationSiblingPolicy.promoted` | PASS |
| Confidence remains `heuristic-review` and cue tier remains `AGENT_REVIEW_CUE` | PASS |
| Muted service-operation policy entries stay hidden in default Markdown | PASS |
| Service-operation cue body avoids reuse/equivalence/safety/action-forcing wording | PASS |
| P2a JSON cue-card behavior remains unchanged | PASS |

## Test Evidence

`node tests/test-pre-write-render.mjs` passed with the P2b assertions:

- `P2b. service operation cue renders explicit review wording`
- `P2b. service operation cue cites the policy evidence path`
- `P2b. muted service operation details remain hidden by default`
- `P2b. service operation renderer avoids strong action wording`

The full suite result was `98 passed, 0 failed`.

The P2a cue-tier adapter regression suite was also re-run and passed:
`24 passed, 0 failed`.

## Wording Boundary

The disallowed wording guard applies to the service-operation cue body. A first
live check matched the legacy `### Already exists (reuse candidates)` section
header, but that header is outside the P2b cue text and predates this policy.

Changing advisory-wide reuse wording is a separate product wording slice. P2b
only verifies that the service-operation cue itself stays review-only and does
not imply reuse, equivalence, safety, existence, required calls, or blocking
failure.

## Verdict

WT-23 P2b public install verification passed for beta.50. The service-operation
sibling Markdown rendering slice may be treated as public-verification
complete.

Remaining WT-23 work is corpus calibration, mutation-family decisions,
signature-based compatibility, and any broader advisory-wide wording cleanup.
