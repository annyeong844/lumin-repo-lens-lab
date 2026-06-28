# Auto Hook Design — Vibe Coder Safety Belt

**Date:** 2026-04-28
**Topic:** Auto hooks (`UserPromptSubmit`, `PostToolBatch`, `PreToolUse`, `Stop`) for `auditing-repo-structure`
**Status:** v10 — ninth maintainer review applied. Phase 1 implementation-ready.

## Problem

Today the skill's `pre-write` and `post-write` flows only fire when the
user (or assistant Marie) explicitly invokes a slash command. Vibe-coder
users do not type slash commands — they describe what they want and let
the assistant code. As a result the most valuable behaviors of this
skill (`silent-new` regression detection, `helper-registry` reuse hints,
`canonical drift` warnings) almost never fire in real sessions.

Goal: bring the substance of `pre-write` and `post-write` to vibe coders
without asking them to learn the tool. The assistant runs them
automatically as a "safety belt" the user does not have to fasten.

## Non-goals

- Replace the manual slash commands.
- Auto-block the assistant or the user. Behavior is *Warn*, not *Block*.
- Start new analysis on non-coding prompts. (Pending due safety events
  may still surface on `UserPromptSubmit` by design.)
- Trigger on non-JS/TS repos.
- Cover behavior changes the assistant did not cause (`git pull`,
  `npm install`, user-side editor edits).
- Phase 1 does not handle `NotebookEdit` — deferred to Phase 1.5.
- Phase 1 does not allow same-key regeneration after acknowledgement.
  An acked `until_ack` identity stays suppressed forever (via tombstone
  lookup). Phase 2 may permit `ack_source: "fixed"` to open a fresh
  generation under explicit policy.

## Mental Model — Six Lines

```
cursor                = new-event discovery only
ack-ledger            = delivery + acknowledgement state, drives redelivery
                        (active entries) AND identity suppression for
                        acked until_ack (tombstone entries)
dedupe_key            = issue identity (opaque string; never split).
                        Phase 1: same key = same event id forever, even
                        across acked-cleanup tombstones for until_ack.
                        once-policy tombstones release identity for fresh
                        emission.
dedupe window         = delivery throttle (when, not whether, to redeliver)
render source         = ALWAYS the ledger; never events.jsonl directly
unacked until_ack     = NEVER becomes invisible. Either due, throttled by
                        next_redeliver_at, or explicitly acknowledged.
                        Never expired while unacked. Never starved by
                        invalid neighbors. Never gated by file size. Never
                        resurrected by recoverOrphans after ack. Never
                        falsely acknowledged by line-only checks.
```

## Decisions Locked

| Axis | Decision |
|---|---|
| Behavior on `silent-new` | **Warn** — never block, surface a one-line note and ask Marie to ACK |
| Phase 1 detection | Foreground sync `PostToolBatch` (`post-write-lite`) reads disk postimage and diffs against PreToolUse preimage fingerprints |
| Phase 1 mutating tools | `Edit`, `Write`, `MultiEdit` only. `NotebookEdit` deferred to Phase 1.5 |
| Same-file batch handling | `post-write-lite` groups mutating calls by `repoRel`. Per group: **strict first-call preimage** (in stable batch order). If first call's preimage is missing → empty baseline + `preimage_incomplete=true` (conservative over-warn). NEVER use a later preimage as the baseline. Read disk postimage ONCE, diff ONCE, append events ONCE per identity, cleanup ALL group preimages |
| Preimage scope | All Phase 1 mutating tools capture preimage at PreToolUse |
| Fingerprint shape | Multiset `Map<diffKey, {count, examples[]}>`; examples carry `{kind, line, snippet, matchedLineText, ctxHint, enclosingSymbol}` |
| Identity vs diff keys | `diff_key` (kind + normalized matched line + enclosing symbol; **no adjacent line, no line number**) drives pre/post diff. `dedupe_key` (kind + repoRel + normalized matched line hash + symbol context hash; **no raw line number**) drives event identity. Both are **opaque strings** — never `split(":")` to extract fields |
| Aggregation rule | Phase 1 emits one event per distinct escape identity. Identical occurrences in the same identity aggregate via `occurrence_count`; the ledger applies `+= max(1, occurrence_delta ?? 1)` on merge or initialises from the same delta on first append |
| Identity lookup (append) | `findIdentityForAppend(sid, dedupe_key, policy)` — policy-aware. Active match → merge. Active acked → suppress (Phase 1). **Acked `until_ack` tombstone → suppress** (Phase 1; mental model invariant on identity persistence). **`once`-policy tombstone → release** (allow fresh event id). No match → allow new |
| Workspace vs package roots | `resolveWorkspaceRoot(cwd)`: try `git rev-parse --show-toplevel` first; on failure, walk ancestors accepting `.git` directory OR `.git` file (worktree/submodule); then workspace markers (`pnpm-workspace.yaml`, `package.json` with `workspaces`); finally nearest `package.json`. `resolvePackageRoot(cwd)`: nearest package.json (per-package eligibility). **Audit root uses workspace root**; eligibility may use either |
| Audit root anchoring | `resolveAuditRoot(cwd) = path.join(resolveWorkspaceRoot(cwd), ".audit")`. Each hook computes once; passed to all event-store and preimage APIs |
| Path safety — tool input | `safeRepoPathForToolInput(cwd, file_path, opts)` resolves relative to cwd; full guards. Returns `{ok, repoRoot, absolute, repoRel, ext, sizeBytes, kind, exists}` |
| Path safety — ledger read | `safeRepoRelForRead(repoRoot, repoRel)` resolves repo-relative paths against `repoRoot` (NOT cwd). Returns `{ok, absolute, exists, sizeBytes?}` |
| Path safety — render | `safeRepoPathSyntactic(repoRel)` is string-only. NO `fs.statSync`, NO `fs.realpathSync`, NO `fs.existsSync` |
| File existence | All path APIs include an explicit `exists` boolean. `sizeBytes === 0` is NOT used to mean "absent" |
| Tool path extraction | `getToolTargetPath(tool_name, tool_input)` |
| ID safety | `safeSessionId(payload)` and `safeToolUseId(raw, fallbackParts)` use `typeof raw === "string"` guard before regex. Fallback is deterministic across PreToolUse and PostToolBatch (no `hook_event_name`, no `call_index`) |
| Tool-use fallback ID material | `canonical_tool_input_subset` uses `sha256(content) + byteLength(content)` for content-bearing fields (`old_string`, `new_string`, `content`); never raw content. Keeps fallback hash small AND avoids surfacing sensitive raw text |
| State distinction | `hasSessionDir` vs `hasEventState`. Drainer + ack-observer use `hasEventState` |
| Empty-drain artifact rule | A prompt with no `hasEventState` AND no mutating tool call leaves zero event-store files |
| Phase 2 reuse hints | `UserPromptSubmit` event with one merged sync runner plus `async: true` slow path |
| Drainer placement | `UserPromptSubmit` AND `PreToolUse`, both foreground, both render from ledger eligibility only |
| ACK observer | `Stop` — prefers `payload.last_assistant_message`; transcript fallback only when missing |
| ACK token format | Line-based sentinel `AUDIT_ACK <event-id> <intentional\|fixed\|noted>`, after stripping fences (closed + unclosed-to-EOF), inline backticks, indented 4-space code blocks, leading-space blockquotes |
| Implicit code-fixed | **Identity-based, never line-only.** Re-fingerprint the file (under `safeRepoRelForRead`) and require absence of a current example whose `dedupe_key` matches the ledger entry's `dedupe_key`. A line shift alone NEVER triggers code-fixed |
| Render source | Ledger-only via `claimDueDeliveriesAndAdvanceCursor(...)` |
| Atomic claim | One per-session lock; **mark-before-emit**; **validate-while-selecting**; **fresh reload inside the lock**; cooperative self-deadline (no `process.exit` while holding lock) |
| Quarantine | Invalid render-time entries get `render_suppressed_at`/`_count`; stay in ledger; re-checked after `suppressionTtl` (1 h) |
| Tombstones | Expired or acked-cleanup entries become inactive ledger entries (`active: false` + `archived_at` + `archive_reason`); not deleted; `recoverOrphans` skips ids present in the ledger regardless of `active`. Identity lookup is policy-aware (see findIdentityForAppend) |
| Activation | Plugin install + `isHookEligibleRepo(cwd)` + per-lane `featureCapability(cwd)` |
| Significance kinds, Phase 1 | `silent-new` only |
| Significance kinds, Phase 2 | adds `parse-error-new`, `helper-registry-hit`, `canonical-drift`, `public-api-touch`, `domain-cluster-collision` |
| TTL semantics | `until_ack` + `acknowledged === false` → never expires |
| Resume / replay | Reminder wording: "observed in this/previous tool batch"; "If acknowledged, ignore this transcript context" disclaimer |
| Ledger render data | `data` carries `{file, line, escape_kind, snippet, enclosing_symbol, matched_line_text}` so renderer can use softer wording without re-parsing the source. All sanitized (length cap, control chars stripped, ANSI/backticks stripped) |
| Display line | `data.line` is best-effort metadata. When `occurrence_count > 1`, renderer uses "near `<enclosing_symbol>`" or "one of N matching escapes" rather than asserting the displayed line is authoritative |
| Worker stdout (Phase 2 async) | Success path writes nothing to stdout/stderr |
| Per-session state | `<auditRoot>/sessions/<safeSessionId>/`; cache at `<auditRoot>/cache/` (Phase 2) |
| Concurrency | `<auditRoot>/cache/.lock` (cache); `<auditRoot>/sessions/<sid>/.event-store.lock` (event-store atomicity); cooperative deadline never aborts inside a held lock |
| Reminder size budget | `additionalContext` ≤ 2 KB rendered |
| Source extensions allowed | `.ts`, `.tsx`, `.mts`, `.cts`, `.js`, `.jsx`, `.mjs`, `.cjs` |
| Source size cap | Skip preimage capture and postimage read if file > 512 KB; never affects render-side ledger validation |
| Self-deadline | Cooperative; ~75 % of platform timeout; exits 0 cleanly after lock release |
| Hook output JSON | Every emit is `{"hookSpecificOutput": {"hookEventName": "<event>", "additionalContext": "<text>"}}` |

## Architecture

(Carries v9 architecture diagram and three invariants. State paths
unchanged. Diagram updated note: PostToolBatch step now reads
"first-call preimage strict; empty baseline if missing.")

### Three invariants

1. **All hooks exit 0.** Cooperative deadline NEVER calls `process.exit`
   while holding any lock.
2. **State is split by lifetime AND by category.** Repo-global persistent:
   `<auditRoot>/cache/` (Phase 2). Session ephemeral:
   `<auditRoot>/sessions/<sid>/`. Within a session, **preimage state ≠
   event state**.
3. **Render is ledger-driven, atomically, with fresh-reload inside the
   lock and validation while selecting.** Identity persistence in
   Phase 1 holds across acked-cleanup tombstones for `until_ack`.

### Assumptions to verify in implementation

- `hooks/hooks.json` shape and `additionalContext` plumbing match
  [code.claude.com/docs/en/hooks](https://code.claude.com/docs/en/hooks)
  and [code.claude.com/docs/en/plugins-reference](https://code.claude.com/docs/en/plugins-reference).
- `async: true` hooks: output reaches Claude on the next turn.
- `UserPromptSubmit`, `PostToolBatch`, `Stop` always fire; matchers ignored.
- `Stop` payload includes `last_assistant_message`.
- `additionalContext` ≤ 10 KB cap; we design for ≤ 2 KB.
- `PreToolUse` per tool call (concurrent in parallel batches).
- Layer 2 fixtures pin `tool_input` shapes for `Edit`, `Write`, `MultiEdit`.
- `git rev-parse --show-toplevel` is available where Claude Code runs;
  if `git` is missing, fall back to ancestor `.git` file/dir scan.

## Phasing — committed

**Phase 1**: `silent-new` ACK loop. Mutating tools = Edit, Write, MultiEdit.
Acked `until_ack` identity stays suppressed forever via tombstone
lookup; no same-key regeneration.

**Phase 1.5**: `NotebookEdit` (cell-scoped); events.jsonl rotation;
`generic-any` lane; pre/post occurrence matching for accurate display
lines on aggregated events.

**Phase 2**: pre-write reuse hints + full audit + helper-registry /
canonical-drift / public-api-touch / domain-cluster lanes + warm-cache
hot path on `UserPromptSubmit` + parse-error-new lane. Optional
generation policy: `ack_source === "fixed"` may permit a fresh event id
when the same identity reappears, behind an explicit
`generationPolicy.allowAfterFixed` config flag.

## Identity & Path Safety

### Workspace, package, and audit roots

```text
resolveWorkspaceRoot(cwd) → string | null
  Order:
    1. Try `git rev-parse --show-toplevel` from cwd.
       Accept stdout if exit code 0 and result is a real directory.
    2. Else walk ancestors:
         - if `.git` is a directory in this ancestor → return ancestor
         - if `.git` is a file (worktree, submodule pointer)
           in this ancestor → return ancestor
    3. Else look for explicit workspace markers in ancestors:
         - pnpm-workspace.yaml
         - package.json containing "workspaces" (npm/yarn/bun)
    4. Else nearest package.json ancestor (single-package repo).
    5. Else null.

resolvePackageRoot(cwd) → string | null
  Walk up from cwd to the nearest package.json. Used for per-package
  eligibility checks; may differ from workspaceRoot.

resolveAuditRoot(cwd) → string | null
  workspace = resolveWorkspaceRoot(cwd)
  return workspace ? path.join(workspace, ".audit") : null

isHookEligibleRepo(cwd) → boolean
  Eligible when resolvePackageRoot OR resolveWorkspaceRoot exists AND
  the bundled or repo-local audit-repo.mjs resolves successfully.
```

### Three path-safety functions

(Unchanged from v9 in shape. `safeRepoPathForToolInput` returns
`{ok, repoRoot, absolute, repoRel, ext, sizeBytes, kind, exists}`;
`safeRepoRelForRead` returns `{ok, absolute, exists, sizeBytes?}`;
`safeRepoPathSyntactic` is string-only. Sensitive deny list unchanged.
Extension allow list: `.ts, .tsx, .mts, .cts, .js, .jsx, .mjs, .cjs`
plus `.ipynb` only when `opts.allowNotebook`.)

### Tool-input target extraction

```text
getToolTargetPath(tool_name, tool_input) → string | null
  Edit       → tool_input.file_path
  Write      → tool_input.file_path
  MultiEdit  → tool_input.file_path
  default    → null
```

### ID safety

```text
SAFE_ID_RE = /^[A-Za-z0-9_-]{1,128}$/

isSafeId(raw) → boolean
  return typeof raw === "string" && SAFE_ID_RE.test(raw)

safeSessionId(payload) → string
  raw = payload.session_id
  if isSafeId(raw): return raw
  if typeof payload.transcript_path === "string" && payload.transcript_path:
    return "sid_" + sha256(payload.transcript_path).slice(0, 16)
  return "default-session"

safeToolUseId(raw, fallbackParts) → string
  if isSafeId(raw): return raw
  // Fallback MUST NOT include hook_event_name or call_index.
  // Content-bearing fields hashed, not raw.
  parts = [
    typeof raw === "string" ? raw : "",
    fallbackParts.sid ?? "",
    fallbackParts.tool_name ?? "",
    fallbackParts.target_repo_rel ?? "",
    sha256(canonicalize(fallbackParts.tool_input_subset ?? {})).slice(0, 16),
    fallbackParts.transcript_path ?? ""
  ]
  return "tool_" + sha256(parts.join("\0")).slice(0, 16)

canonicalize(obj):
  Stable JSON serialization: keys sorted recursively, no whitespace.

canonical_tool_input_subset (P0 update — content fields hashed):

  Edit:
    {
      file_path,
      old_hash:  sha256(old_string ?? ""),
      old_len:   byteLength(old_string ?? ""),
      new_hash:  sha256(new_string ?? ""),
      new_len:   byteLength(new_string ?? ""),
      replace_all
    }

  Write:
    {
      file_path,
      content_hash: sha256(content ?? ""),
      content_len:  byteLength(content ?? "")
    }

  MultiEdit:
    {
      file_path,
      edits: edits.map(e => ({
        old_hash:    sha256(e.old_string ?? ""),
        old_len:     byteLength(e.old_string ?? ""),
        new_hash:    sha256(e.new_string ?? ""),
        new_len:     byteLength(e.new_string ?? ""),
        replace_all: e.replace_all
      }))
    }

Principle: safeToolUseId fallback uses stable hashes of content-bearing
fields, never raw content. PreToolUse and PostToolBatch derive the same
fallback id for the same tool call without copying potentially huge
strings into the ID material.
```

## Components

### Files

(Same as v9.)

### `hooks/hooks.json` — Phase 1 shape

(Full Phase 1 JSON — same as v9. UserPromptSubmit + PostToolBatch +
PreToolUse + Stop. No matcher on UserPromptSubmit/PostToolBatch/Stop;
PreToolUse uses `matcher: "*"`. Timeouts 2/3/2/5 seconds.)

### `escape-detect.mjs`

(Same as v9 — patterns, `diff_key`, `dedupe_key`, multiset fingerprint
API, example shape with `kind, line, snippet, matchedLineText, ctxHint,
enclosingSymbol`. Both keys are **opaque strings**; downstream code
never `split` them to extract fields.)

### `preimage.mjs`

(Same as v9. Uses `safeReadResult.exists` to choose `absent: true`
versus capturing fingerprint of an existing file, including empty.)

### `event-store.mjs`

#### State distinction and open modes

(Same as v9. `openReadOnly` returns only `hasEventState` boolean;
`openWritable` creates dirs only when about to write.)

#### Ledger entry schema

```json
{
  "id": "evt_20260428_070123_a1b2",
  "active": true,
  "session_id": "<safeSessionId>",
  "ts": "2026-04-28T07:01:23.456Z",
  "kind": "silent-new",
  "severity": "warn",
  "ack_required": true,
  "delivery_policy": "until_ack",
  "diff_key": "<opaque>",
  "dedupe_key": "<opaque>",
  "data": {
    "file": "_lib/parse.mjs",
    "line": 142,
    "escape_kind": "as any",
    "snippet": "value as any",
    "enclosing_symbol": "parse",
    "matched_line_text": "const value = raw as any"
  },
  "created_at": "...",
  "first_seen_at": "...",
  "delivered_count": 1,
  "delivered_at": "...",
  "next_redeliver_at": "...",
  "last_seen_at": "...",
  "occurrence_count": 1,
  "acknowledged": false,
  "acknowledged_at": null,
  "ack_source": null,
  "expires_at": null,
  "render_suppressed_at": null,
  "render_suppressed_count": 0,
  "render_suppressed_reason": null,
  "archived_at": null,
  "archive_reason": null
}
```

`data.snippet` and `data.matched_line_text` are sanitized at append
time: length ≤ 160, single-line, control characters stripped, ANSI
escapes stripped, backticks stripped. `data.enclosing_symbol` is
sanitized to ≤ 64 characters, alphanumeric + underscore + dot.

#### Identity lookup — policy-aware

```text
findIdentityForAppend(sid, dedupe_key, delivery_policy) →
  | { kind: "active-merge",        entry }   // existing active unacked
  | { kind: "active-acked-suppress", entry } // existing active acked
  | { kind: "tombstone-suppress",  entry }   // until_ack tombstone (acked-cleanup)
  | { kind: "tombstone-released",  entry }   // once-policy tombstone
  | { kind: "allow-new" }                    // no ledger entry at all

Lookup logic:
  active = find active ledger entry by dedupe_key (any policy)
  if active:
    if !active.acknowledged: return { kind: "active-merge", entry: active }
    else: return { kind: "active-acked-suppress", entry: active }

  tombstone = find tombstone (active === false) by dedupe_key
  if tombstone:
    if tombstone.delivery_policy === "until_ack":
      // Phase 1 mental-model invariant: same key never regenerates
      // a new event id after acknowledgement.
      return { kind: "tombstone-suppress", entry: tombstone }
    if tombstone.delivery_policy === "once":
      // Phase 2 user-visible: once-policy identity reset.
      return { kind: "tombstone-released", entry: tombstone }

  return { kind: "allow-new" }

Phase 2 may add: if delivery_policy === "until_ack" AND
generationPolicy.allowAfterFixed AND tombstone.ack_source === "fixed",
return { kind: "tombstone-released", entry: tombstone } so that fixed-then-reintroduced
identities can produce a new generation. Out of scope for Phase 1.
```

#### Public API

- `cleanupExpiredEntries(now)` — internal; tombstones, never deletes.

- `appendEventIfNotDeduped(auditRoot, sid, event) → {appended, eventId}`

  ```text
  delta = max(1, event.occurrence_delta ?? 1)

  Under lock:
    cleanupExpiredEntries(now)
    lookup = findIdentityForAppend(sid, event.dedupe_key,
                                    event.delivery_policy)

    case "active-merge":
      lookup.entry.last_seen_at = now
      lookup.entry.occurrence_count += delta
      lookup.entry.data = sanitize(event.data)
      return { appended: false, eventId: lookup.entry.id }

    case "active-acked-suppress":
    case "tombstone-suppress":
      // Phase 1 simplification: skip append; identity considered settled.
      return { appended: false, eventId: lookup.entry.id }

    case "tombstone-released":
    case "allow-new":
      append events.jsonl line; create active ledger entry with
        created_at = first_seen_at = now,
        delivered_count = 0,
        occurrence_count = delta,
        expires_at = per delivery_policy
      return { appended: true, eventId: <new> }
  ```

- `discoverNewEvents`, `dueDeliveries`, `markDelivered`,
  `markAcknowledged`, `markRenderSuppressed`, `recoverOrphans`,
  `advanceCursor`, `validateLedgerEntryForRender` — same as v9.

- `claimDueDeliveriesAndAdvanceCursor(auditRoot, sid, opts)` — same
  fresh-reload-in-lock + validate-while-selecting algorithm as v9.

#### Significant kinds

(Same table as v9.)

### `deadline.mjs`

(Same as v9. Cooperative; no `process.exit` while holding locks.)

### `event-drain.mjs` contract

(Same as v9. PreToolUse captures preimage FIRST, then drains. Drain
opens read-only first; if `hasEventState`, calls
`claimDueDeliveriesAndAdvanceCursor`.)

### Reminder wording

Renderer uses ledger `data` directly; no source re-read at render time.

```text
[audit · observed in this/previous tool batch]
<file>:<line> — <observational sentence using kind, snippet, enclosing_symbol>
Event id <evt_xyz>.
(occurrence_count > 1: "<N> matching escapes near `<enclosing_symbol>`")

If this event was already acknowledged, ignore this transcript context.
The live ledger controls future reminders.

To acknowledge, place a single line in your reply by itself
(NOT inside a code fence) of the form:
  AUDIT_ACK <event id above> <intentional|fixed|noted>
```

`<enclosing_symbol>` is taken from `data.enclosing_symbol`. When
`occurrence_count > 1`, the line shown is best-effort; the wording uses
"near" / "one of N" rather than asserting exact location.

### `post-write-lite.mjs` contract — strict first-call preimage

```text
event   : PostToolBatch
input   : Claude Code hook payload on stdin
output  : exit 0 within 3s; optionally hookSpecificOutput JSON
budget  : cooperative self-deadline 2500ms; platform 3000ms

behavior:
  d         = Deadline.after(2500)
  sid       = safeSessionId(payload)
  auditRoot = resolveAuditRoot(payload.cwd)
  if auditRoot null → exit 0.

  1. Gate: isHookEligibleRepo(payload.cwd) &&
           featureCapability(payload.cwd) ⊇ {silent-new}
       false → exit 0
  2. Filter:
       MUTATING = {Edit, Write, MultiEdit}     // Phase 1
       mutating = tool_calls.filter(c => MUTATING.has(c.tool_name))
       if mutating.length === 0 → exit 0

  3. Resolve and validate ALL mutating calls; group by safe.repoRel.
     Preserve the batch order per group:
       For each mutating call in batch order:
         path = getToolTargetPath(c.tool_name, c.tool_input)
         safe = safeRepoPathForToolInput(payload.cwd, path,
                  { toolName: c.tool_name })
         if !safe.ok → log, drop this call from grouping
         tid = safeToolUseId(c.tool_use_id, {
           sid, tool_name: c.tool_name,
           target_repo_rel: safe.repoRel,
           tool_input_subset: extractCanonicalSubset(c.tool_name, c.tool_input),
           transcript_path: payload.transcript_path
         })
         group[safe.repoRel] ||= { safe, calls: [] }
         group[safe.repoRel].calls.push({ call: c, tid })
       (calls within each group remain in original batch order)

  4. For each (repoRel, group) — diff ONCE per file:
       if d.tooClose(300) → break

       // STRICT FIRST-CALL PREIMAGE.
       firstCall = group.calls[0]    // first in batch order
       firstPreimage = preimage.read(auditRoot, sid, firstCall.tid)

       if firstPreimage:
         pre_fp = firstPreimage.absent
                    ? empty Map
                    : deserialize(firstPreimage.fingerprint)
         preimageIncomplete = false
       else:
         // First call's preimage is missing. We must NOT use a later
         // preimage as the baseline (it would already include the
         // first call's mutation, hiding the new escape).
         pre_fp = empty Map
         preimageIncomplete = true
         // log: "preimage_incomplete: missing first preimage for
         // <repoRel> in batch; treating baseline as empty"

       postimage_text = read disk at group.safe.absolute
       if read fails → log, cleanup all group preimages, continue
       postimage_fp = escape-detect.fingerprint(postimage_text)

       diffs = escape-detect.diff(pre_fp, postimage_fp)
       for each {diff_key, addedCount, examples} in diffs:
         representative = examples.at(-1)
         dk = escape-detect.buildDedupeKey(
           group.safe.repoRel, representative.kind,
           representative.matchedLineText, representative.enclosingSymbol)
         event = {
           id: generated, session_id: sid, ts: now,
           kind: "silent-new", severity: "warn",
           ack_required: true, delivery_policy: "until_ack",
           diff_key, dedupe_key: dk,
           data: {
             file: group.safe.repoRel, line: representative.line,
             escape_kind: representative.kind,
             snippet: sanitize(representative.snippet),
             enclosing_symbol: sanitize(representative.enclosingSymbol),
             matched_line_text: sanitize(representative.matchedLineText)
           },
           occurrence_delta: addedCount,
           debug: preimageIncomplete ? { preimage_incomplete: true } : undefined
         }
         event-store.appendEventIfNotDeduped(auditRoot, sid, event)

       // Cleanup ALL preimages in the group, not just the earliest.
       for each {tid} in group.calls:
         preimage.cleanup(auditRoot, sid, tid)

  5. Claim and render same turn:
       if d.expired() → exit 0
       view = event-store.openReadOnly(auditRoot, sid)
       if view.hasEventState:
         {snapshots, ackHints} = event-store.claimDueDeliveriesAndAdvanceCursor(
           auditRoot, sid, { limit: 5, dedupeWindowSec: 60, deadline: d })
         if snapshots.length > 0:
           additionalContext = render(snapshots, ackHints, budget=2048)
           emit:
             {"hookSpecificOutput": {
                "hookEventName": "PostToolBatch",
                "additionalContext": additionalContext
              }}

  6. Cleanup orphan preimages older than 1 hour.
  7. exit 0 always.
```

The `preimage_incomplete` debug field is internal; never surfaced in
`additionalContext`. It gives `hook-doctor.mjs` a way to count how
often Phase 1 falls back to over-warn.

### `ack-observer.mjs` contract

(Same as v9 — identity-based code-fixed. Uses `safeRepoRelForRead`,
`exists` boolean. Last-seen-at 5s grace window. Missing file does NOT
trigger code-fixed.)

### `repo-mode.mjs` extensions

(Carries v9 contract. `featureCapability` per-lane logic unchanged.)

### Phase 2 only

(Same as v9. Note that Phase 2 may add `generationPolicy.allowAfterFixed`
to permit `findIdentityForAppend` to release `until_ack` tombstones
when `ack_source === "fixed"`. Phase 1 hardcodes Phase 1 behaviour.)

## Data Flow

(Scenarios A–R from v9 carry over. Updates / additions:)

### Scenario M (updated)

(Monorepo subdir cwd. `resolveWorkspaceRoot` now also handles `.git`
file via `git rev-parse --show-toplevel`. Audit root at workspace root.)

### Scenario S (new) — acked silent-new tombstone suppresses regeneration

```
T0:  silent-new evt_xyz raised at _lib/parse.mjs (`as any` in `parse`).
T1:  Marie acks: evt_xyz active, acknowledged=true, expires_at=T1+24h.
T2 (T1+24h): cleanupExpiredEntries marks evt_xyz active=false,
     archived_at=T2, archive_reason="acked-cleanup".
T3:  Marie's new Edit reintroduces the same `as any` at the same
     identity (same file + line text + enclosing symbol).
T4:  PostToolBatch → post-write-lite → appendEventIfNotDeduped:
       cleanupExpiredEntries (no-op for evt_xyz; already tombstoned)
       findIdentityForAppend:
         active match? no
         tombstone match by dedupe_key? yes → evt_xyz, until_ack
         → return { kind: "tombstone-suppress" }
       → skip append; no new event id; no reminder
```

This preserves the mental-model invariant: in Phase 1, an acked
identity stays settled. Phase 2 may relax this with
`generationPolicy.allowAfterFixed`.

### Scenario T (new) — same-file batch with missing first preimage

```
Batch:
  Edit A on _lib/x.ts: introduces `as any`
  Edit B on _lib/x.ts: unrelated change

PreToolUse(A): preimage.capture FAILS (e.g., race on a freshly created
               file, or transient I/O error). Logged; proceed.
Edit A applied to _lib/x.ts (now contains `as any`).
PreToolUse(B): preimage.capture succeeds; the captured snapshot
               already includes the `as any` from Edit A.

PostToolBatch → post-write-lite:
  group _lib/x.ts: calls = [A, B]
  firstCall = A
  firstPreimage = preimage.read(A.tid) → null (capture had failed)
  pre_fp = empty Map
  preimageIncomplete = true     // log diagnostic

  postimage_text = read disk after both edits
  postimage_fp = fingerprint includes the `as any`
  diff(empty, postimage_fp) → reports the `as any` as new
  appendEventIfNotDeduped → silent-new evt for the `as any`

Result: detector OVER-WARNS rather than missing the regression.
        If the `as any` was actually pre-existing (impossible in this
        scenario but possible if the batch baseline is wrong for other
        reasons), Marie can ACK with intentional/noted; the identity
        is then suppressed forever per Scenario S.
```

This is the safer failure mode. Using B's preimage as baseline would
have hidden A's regression entirely, violating the unacked-invariant
spirit (no false negatives on real new escapes).

## Concurrency Notes

(Carries v9 list. No additions for v10.)

## Failure Modes

(Carries v9 table. Updates / additions:)

| Category | Scenario | Behavior |
|---|---|---|
| Workspace | git worktree (`.git` is a file) or submodule | `resolveWorkspaceRoot` tries `git rev-parse --show-toplevel` first; if git CLI unavailable, ancestor walk accepts `.git` file as well as directory |
| Workspace | pnpm monorepo without `package.json#workspaces` | Falls back to `pnpm-workspace.yaml` ancestor |
| Identity | acked silent-new whose original identity reappears 24h+ later | `findIdentityForAppend` returns `tombstone-suppress` for `until_ack`; no new event; reminder does not re-surface |
| Identity | once-policy entry expired and re-detected | `findIdentityForAppend` returns `tombstone-released`; new event id issued |
| Batch | first call's preimage missing in same-file group | Use empty baseline + `preimage_incomplete=true` debug. Detector OVER-WARNS; Marie can ACK as intentional/noted. NEVER use a later preimage as baseline |
| Tool-use ID | huge `Write.content` or `new_string` | `canonical_tool_input_subset` uses sha256 + length only; raw content never enters `safeToolUseId` material; constant-time fallback hash regardless of payload size |

(All other v9 entries unchanged.)

## Testing Strategy

(Carries v9 layers. Adds these required tests for v10.)

### Layer 1 — Unit tests (additions)

| Module | Phase | Added cases |
|---|---|---|
| `safe-repo-path.mjs` | 1 | `resolveWorkspaceRoot` via `git rev-parse --show-toplevel` (mock); fallback to `.git` file (worktree); fallback to `.git` directory; fallback to `pnpm-workspace.yaml`; fallback to `package.json#workspaces`; fallback to nearest package.json |
| `safe-repo-path.mjs (tool-use ID)` | 1 | **`safeToolUseId` fallback hashes content fields**: a 5 MB Write produces an ID whose computation is bounded by sha256(content) cost (no JSON serialization of 5 MB); two PreToolUse/PostToolBatch contexts produce the same ID for identical Write.content |
| `event-store.mjs (findIdentityForAppend)` | 1 | active unacked match → "active-merge"; active acked match → "active-acked-suppress"; **until_ack tombstone → "tombstone-suppress"** (BLOCKING); **once tombstone → "tombstone-released"** (BLOCKING); no match → "allow-new" |
| `event-store.mjs (data sanitization)` | 1 | `data.snippet`, `data.matched_line_text`, `data.enclosing_symbol` are sanitized: length capped, control chars stripped, ANSI/backticks stripped |
| `post-write-lite.mjs` | 1 | **strict first-call preimage**: missing first preimage → empty baseline + `preimage_incomplete=true`, NEVER falls back to later preimage (BLOCKING); group call order preserved |

### Layer 3 — Integration tests (additions)

- **BLOCKING (acked identity suppression):** ack a silent-new; advance
  clock past 24 h cleanup; in a NEW PostToolBatch, reintroduce the
  same identity (same file + line text + enclosing symbol). Expected:
  no new event id; no reminder; ledger has tombstone entry with
  `findIdentityForAppend` returning `tombstone-suppress`.
- **BLOCKING (once identity reset, Phase 2 surface):** simulate a
  once-policy entry tombstoned; new occurrence with same `dedupe_key`
  → fresh event id appended. (Phase 1 covers via event-store unit
  test; Phase 2 makes this user-visible via helper-registry-hit.)
- **BLOCKING (worktree workspace root):** create a fixture with a
  git worktree (`.git` is a file pointing to gitdir); cwd inside the
  worktree → `resolveWorkspaceRoot` returns the worktree root; audit
  state lives at `<worktreeRoot>/.audit/`.
- **BLOCKING (same-file missing first preimage):** simulate the
  failure injection where `preimage.capture` fails for the first call
  in a same-file group; second call's preimage is captured normally;
  PostToolBatch must emit a `silent-new` for the actual new escape
  (over-warn path), NOT use the second preimage as baseline.
- Tool-use ID large content: PreToolUse with a 5 MB Write; missing
  `tool_use_id`; assert PostToolBatch derives the same fallback id and
  finds the preimage; assert ID derivation latency is dominated by
  sha256, not by JSON serialization.

### Layer 4 — Failure-mode regression tests (additions)

- pnpm monorepo without `.git`: `pnpm-workspace.yaml` at workspace root
  + nested `package.json` in subpackages → audit anchored at workspace.
- `git rev-parse` unavailable (PATH stripped): fallback walk accepts
  `.git` file/dir.

### Layer 5 — Performance benchmarks (additions)

| Metric | Target |
|---|---|
| `resolveWorkspaceRoot` via `git rev-parse` (cached child process) | < 30 ms |
| `resolveWorkspaceRoot` ancestor walk fallback | < 10 ms |
| `safeToolUseId` fallback for 5 MB Write (sha256 dominant) | < 60 ms |
| `findIdentityForAppend` over a 1000-entry ledger | < 5 ms |

### Layer 6 — Manual end-to-end (additions)

- In a git worktree of the maintainer repo (created via
  `git worktree add ../audit-wt branchname`), trigger silent-new and
  ACK; confirm state at `<worktreeRoot>/.audit/`.
- Acked-identity test: introduce a silent-new, ACK, wait
  (or use a manual tombstone fixture); reintroduce the same escape;
  confirm reminder does NOT re-surface.

### Coverage gates

(All v9 BLOCKING tests still required, plus:)
- BLOCKING: acked identity tombstone suppression
- BLOCKING: once tombstone identity reset
- BLOCKING: worktree workspace root
- BLOCKING: same-file missing first preimage over-warns

## Open Questions Deferred to Implementation

- Drainer reminder language (Korean vs English) — auto-detect; default English.
- Lock implementation: `proper-lockfile` (npm) preferred for cross-platform.
- `CLAUDE_PLUGIN_DATA` use: out of scope for P0.
- events.jsonl rotation (Phase 1.5).
- Phase 1.5: NotebookEdit cell-scoped preimage; `generic-any` lane;
  pre/post occurrence matching for accurate display lines on aggregated
  events.
- Phase 2 `generationPolicy.allowAfterFixed`: explicit config to allow
  `until_ack` tombstone identity reset when `ack_source === "fixed"`.
- Implicit code-fixed `last_seen_at` grace window (5 s proposed) tuning.
- `enclosing_symbol` extraction sophistication.
- `suppressionTtl` default (1 h) tuning.
- Ledger growth in long-running sessions: tombstones never deleted in
  Phase 1; Phase 1.5 may add compaction.
- `git rev-parse` invocation cost: cache result per (auditRoot, hook
  invocation) tuple to avoid spawning per-event.

## Acceptance Criteria

### Phase 1

1. Edit/Write/MultiEdit introducing an unannounced escape produces a
   `silent-new` reminder either same turn (PostToolBatch) or on the
   next drainer fire.
2. Pre-existing escapes do NOT produce reminders.
3. Per-identity emission with aggregation: N distinct identities → N
   event ids; identical occurrences in one identity aggregate via
   `occurrence_count`.
4. Failed `Edit` produces NO event.
5. Multiset growth via delta: `occurrence_count += event.occurrence_delta`.
6. Adjacent-line stability.
7. Symbol-boundary detection.
8. Path safety.
9. ID safety (typeof guard).
10. Tool-use id determinism: PreToolUse and PostToolBatch derive the
    same `safeToolUseId` for the same logical tool call.
11. **Tool-use id uses content hashes, not raw content.** A 5 MB
    `Write.content` does not appear verbatim in any safeToolUseId
    material; ID derivation latency is dominated by `sha256(content)`.
12. **Workspace-anchored audit root.** Works with `.git` directory,
    `.git` file (worktree/submodule), `pnpm-workspace.yaml`, and
    `package.json#workspaces`.
13. Empty no-artifact.
14. Preimage-only no event-state.
15. AUDIT_ACK on its own line acknowledges.
16. AUDIT_ACK in any code structure does NOT acknowledge.
17. Ignored reminder re-surfaces after `next_redeliver_at`.
18. Unacked `until_ack` events do NOT expire.
19. Render is not size-gated.
20. **Once-policy identity reset (tombstone-released).**
21. **Acked `until_ack` identity stays suppressed forever via
    tombstone-suppress.** A new occurrence with the same `dedupe_key`
    after acked-cleanup tombstoning produces NO new event id and NO
    reminder. (BLOCKING.)
22. Acked event never resurrects (recoverOrphans tombstone-aware).
23. Atomic claim with fresh reload.
24. Validate-while-selecting.
25. **Same-file multi-call grouping with strict first-call preimage.**
    Missing first preimage → empty baseline + over-warn; later preimage
    NEVER used as baseline. `occurrence_count` reflects net diff, not
    per-call doubling.
26. Identity-based code-fixed (line-shift safe).
27. Same identity introduced twice in two batches produces only one
    event id.
28. Read-only batch produces zero events and zero `additionalContext`
    (with empty ledger).
29. Two concurrent Claude Code sessions in the same repo do not share events.
30. Hostile ledger defense.
31. Cooperative deadline lock cleanup.
32. Resume / replay tolerance.
33. Subdir cwd correctness.
34. Hook output JSON shape.
35. Existing empty file: preimage `absent: false` with empty fingerprint.
36. **Ledger `data` carries `enclosing_symbol` and `matched_line_text`**;
    renderer uses `enclosing_symbol` for "near `<symbol>`" wording when
    `occurrence_count > 1`.
37. **`dedupe_key` is opaque.** No code path splits it on `:` or any
    delimiter; comparisons are full-string equality.
38. `scripts/hook-doctor.mjs` reports green.
39. All Layer 1–4 tests pass; Layer 5 benchmarks within budget; manual
    Layer 6 checklist signed off; ALL BLOCKING tests pass.

### Phase 2 (additive)

40. Warm-cache `helper-registry-hit` reminder same turn (best-effort).
41. Cold-cache reminder by next turn.
42. Merged `UserPromptSubmit` sync runner emits exactly one
    `additionalContext` per fire.
43. `parse-error-new` lane works.
44. After 24 h without ACK, `helper-registry-hit` for the same helper
    emits as a fresh event id (once-policy `tombstone-released`).
45. `generationPolicy.allowAfterFixed` (when enabled) permits
    `until_ack` tombstone-released after `ack_source === "fixed"`.
