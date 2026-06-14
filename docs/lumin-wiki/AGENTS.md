# Lumin Wiki Rules

This directory is a maintainer-only synthesis wiki for Lumin Repo Lens. It is
not a public skill surface and should not be copied into generated package
artifacts unless a future packaging spec explicitly says so.

## Source Of Truth

Wiki pages summarize and connect evidence. They do not replace:

- `docs/spec/` design specs
- `docs/lab/` verification notes and corpus evidence
- `tests/test-*.mjs` regression suites
- `tests/README.md` generated test inventory
- engine code under root scripts, `_lib/`, and `skills/lumin-repo-lens-lab/_engine/`

When a wiki page makes a concrete claim, link to the source document, test, or
work tracker entry that supports it.

## Edit Rules

- Keep pages short enough to scan.
- Prefer links over pasted history.
- Record material wiki changes in `log.md`.
- Update `index.md` when adding, renaming, or removing a page.
- Mark uncertainty as uncertainty. Do not turn a lab note into a product claim.
- Do not use wiki text as a substitute for regression tests.

## Test Reform Rule

When documenting tests, name the protected invariant and the failure mode. A
test that only proves a helper exists is weak. A test that would catch a known
edge case is strong.
