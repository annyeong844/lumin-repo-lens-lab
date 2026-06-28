# Test Reform

The test suite should become easier to navigate and harder to accidentally
weaken. The direction is risk-based tests, not conversation-shaped tests.

## Risk Categories

- Component contract: focused helper behavior and edge cases.
- Artifact shape: schema fields, support flags, mirrors, and Markdown guidance.
- Resolver blind zone: unsupported/candidate output and absence of fake edges.
- Deadness/ranking: graph evidence, blockers, and action safety.
- Regression edge case: a known bug class captured as a minimal fixture.
- Public install/corpus verification: installed package behavior and dogfood
  evidence.
- Performance measurement: counters, cache ratios, and measured timing deltas.

## Strong TDD

A strong red test fails because current behavior is wrong on a concrete edge
case. Examples:

- prototype-named class methods such as `toString` and `constructor`
- namespace re-export siblings that should remain dead
- unreachable SCCs that should be review evidence but not SAFE_FIX
- `import.meta.glob` surfaces that should be unsupported diagnostics
- exact tiny clones that have identical exact-body hashes

## Weak TDD

A weak red test fails only because a new helper or file does not exist. That is
allowed only when the same test also contains the edge-case fixture that the new
unit must protect.

## Migration Rules

- Do not move suites before inventorying the protected invariant.
- Do not merge fixtures if the original bug becomes harder to see.
- Prefer one fixture shape reused across suites only when the shared shape keeps
  the same failure mode.
- Keep generated `tests/README.md` as inventory; do not hand-edit it.
- When a test is refactored, preserve a negative assertion that would fail if
  Lumin overclaimed evidence.
- Use the [Structure Review Charter](review-charter.md) before extracting
  shared shapes, functions, or helpers.
- Keep professional-runner pilots explicitly scoped to reviewed test files.
  Corpora, generated repo fixtures, audit outputs, and lab payloads are test
  data, not runner-discoverable test suites.
