# Blind Zones

A blind zone is a known limit in the constructed evidence. It does not mean the
code is wrong. It means Lumin cannot make a stronger absence claim without more
support.

## Common Families

- generated artifact missing or excluded
- output-to-source layout unsupported
- dynamic module surface unsupported
- framework/resource surface outside ordinary imports
- unresolved internal import relevant to a candidate

## Scoping Rule

Blind zones should be scoped to affected packages, files, exports, or candidate
surfaces whenever possible. They must not become repo-global blockers unless the
resolver cannot determine ownership or internal/external status.

## Test Implication

Blind-zone tests should verify:

- family and reason code
- affected surface metadata
- blocked candidate hint when relevant
- no fake resolved edge
- no unrelated global blocking
