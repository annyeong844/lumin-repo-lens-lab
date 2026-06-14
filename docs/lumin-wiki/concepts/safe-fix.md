# SAFE_FIX

`SAFE_FIX` is an action label, not a confidence vibe. It should require concrete
proof that the proposed edit preserves runtime behavior, type behavior, public
surface constraints, and local references.

## Boundaries

- Demoting an unused export can be safe when the declaration remains and local
  references are preserved.
- Deleting a declaration is stronger and needs stronger proof.
- Review evidence such as unreachable SCCs or suppressed pre-write candidates is
  not automatically SAFE_FIX evidence.
- Public API, generated consumers, framework sentinels, and resolver blind zones
  can block or degrade promotion.

## Test Implication

SAFE_FIX tests should assert both the proposed edit and the blockers for
stronger edits. A good test says "demote is allowed, delete is blocked" or
"review evidence exists, but no SAFE_FIX entry was created."
