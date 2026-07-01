# Rust Canon-Draft Lifecycle Execution

## Goal

Move the active `audit-repo.mjs --canon-draft` lifecycle child orchestration
from `_lib/audit-canon-draft.mjs` into `lumin-audit-core` without moving canon
draft content generation, final manifest writing, or any JS/TS producer
semantics.

## Checked JS Behavior

The migrated owner mirrors `_lib/audit-canon-draft.mjs`:

- missing `--sources` means all canon draft sources:
  `type-ownership`, `helper-registry`, `topology`, `naming`
- `all` expands to that same source list
- duplicate requested sources are deduped while preserving first occurrence
- unknown source values return a raw block with `requested: true`,
  `ran: false`, a reason of `unknown --sources values: ...`, `exitCode = 1`,
  and `forceExitCode = true`
- each requested source spawns `generate-canon-draft.mjs` with `--root`,
  `--output`, `--source`, forwarded scan args, and `--canon-output`
- child exit `0` records `ran: true`, `exitCode: 0`, and a draft path from the
  child stderr line when present, otherwise `<canon-output>/<source>.md`
- child exit `2` records
  `required producer artifact absent (see stderr of child process)`
- other non-zero exits record `generate-canon-draft.mjs exited <code>`
- if every requested source fails, the raw block has
  `reason: all requested sources failed`, `exitCode = 1`, and
  `forceExitCode = false`

## Rust Owner

`experiments/rust-main/lumin-audit-core/src/canon_draft_lifecycle.rs` owns only
the raw `manifest.canonDraft` execution block and `execute-canon-draft` JSON
CLI. The block includes `executionOwner: "lumin-audit-core"` so the owner change
is artifact-visible.

## Non-Goals

- Do not port `generate-canon-draft.mjs`.
- Do not port `check-canon` in this slice; it reads and aggregates
  `canon-drift.json` and needs a separate parity plan.
- Do not port pre-write/post-write; their advisory and delta semantics are
  separate product lanes.
- Do not move final `manifest.json` writing.
- Do not add timeouts, repository-size caps, or elapsed-time limits.

## Verification

Rust tests must prove the product behavior above with fake child executables:
successful source execution with fallback draft paths, unknown-source hard
failure, all-failed advisory exit semantics, and malformed CLI request
hard-stop behavior.
