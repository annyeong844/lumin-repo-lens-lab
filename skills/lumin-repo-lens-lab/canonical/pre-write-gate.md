# canonical/pre-write-gate.md

> **Role:** protocol for P1's pre-write mode. When Claude is about to write new code, this mode finds what already exists and surfaces it before the write happens.
> **Owner:** this file.

---

## 1. Purpose

Close the gap between "Claude about to write X" and "Claude knowing whether X exists". This is the skill's top-priority feature per `canonical/invariants.md` §6.

## 2. Entry

- Dispatched from `canonical/mode-contract.md` §2.1 (trigger rules).
- Must return **before** Claude writes source code for the request. If pre-write times out or errors, Claude labels the situation `[확인 불가, pre-write failed]` and proceeds with an elevated caution posture; pre-write does not block.

## 3. Protocol

### Step 1 — Ground state lookup

Read, in order, the first available:

1. `canonical/` directory if present → authoritative declared invariants.
2. Most recent `<output>/<fact-artifact>.json` set if present and not stale (see `canonical/fact-model.md` §4 for confidence-downgrade + staleness rules).
3. Fresh targeted audit otherwise (run the minimum subset of scripts needed — see §4 below).

Absence of canonical/ is NOT absence of evidence. The fact model from the skill's existing scripts already grounds most of the pre-write needs.

### Step 2 — Extract intent

From Claude's own upcoming action (what it is about to create), extract:

- **Names** Claude plans to introduce (e.g., `formatTimestamp`, `UserProfile`).
- **Shapes** Claude plans to introduce (field sets for new interfaces, parameter shapes for new functions).
- **Files** Claude plans to touch or create.
- **Dependencies** Claude plans to add or reuse from package declarations
  (package specifiers such as `date-fns` or `@scope/pkg`; internal modules
  belong in `files` or `names`).
- **Planned type escapes** — any intentional `any`, `as any`, `as unknown as T`, JSDoc `{any}`, `@ts-ignore`, or `no-explicit-any` disable Claude plans to write. Each declared up front with the reason (third-party lib shape unknown, migration scaffold, etc.). Declaration here is the intent-side half of the three-stage defense in `canonical/any-contamination.md` §6 Stage 1; post-write compares planned-vs-observed to catch silently-introduced escapes.
- **Planned files** — repo-relative files Claude expects to add or materially touch when that can be inferred. Post-write compares this list against files that appeared after pre-write and records scanned files outside the list as `fileDelta.unexpectedNew`.

Intent extraction is explicit — Claude states these five items before pre-write proceeds when it can. This forces self-declaration, which is itself a useful artifact. The JSON transport normalizes all five top-level keys: `names`, `shapes`, `files`, `dependencies`, and `plannedTypeEscapes` (see `references/pre-write-intent-shape.md`). Missing top-level arrays are defaulted to `[]` with `intentWarnings`; present-but-wrong types remain schema errors. `names` / `dependencies` may be terse strings or structured self-declarations with `why`; lookups normalize to strings and preserve the declaration metadata in the advisory JSON. Structured `names` may also carry `ownerFile`, or the `file` / `targetFile` aliases, so locality-sensitive pre-write policies can calibrate through the normal CLI route.

### Step 3 — Lookup per intent item

For each item in the intent list:

| Intent               | Lookup                                                                                                                                                                       | Output shape                                                                                                                                                                    |
| -------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Name candidate       | symbol graph by identity (see `identity-and-alias.md`)                                                                                                                       | "exists at `<owner>::<name>`" / "not observed in scan range"                                                                                                                    |
| Shape candidate      | shape hash index (fact-model.md) + any-contamination check (any-contamination.md §3 "Contamination definitions" + the "Pre-write gate interaction" block in §6 Stage 1 / §9) | "matches shape of `<identity>`" / "matches but candidate is any-contaminated" / "no structural match"                                                                           |
| File candidate       | topology + symbol parse-error facts                                                                                                                                          | "file exists; N LOC" / "new file"; may also emit domain-cluster Watch-for evidence when sibling files share a prefix or repeated basename token. Boundary evaluation remains unavailable until intent carries planned edge endpoints. |
| Dependency candidate | package.json + existing package-import consumer graph                                                                                                                        | "already imported from `<file>`" / "declared package with no observed static package imports" / "new package; requires install"                                                 |

Every lookup returns a result with a citation in the `[grounded, <artifact>.json.<field>=<value>]` form (invariants.md Iron Law).

**Any-contamination reuse caution rule** (from `canonical/any-contamination.md` §6 Stage 1 "Pre-write" + §9 "Pre-write gate interaction", annotation shape in §4, tier definitions in §3): caution is label-specific, NOT blanket. The `anyContamination` annotation may carry any combination of `has-any`, `any-contaminated`, `severely-any-contaminated`, and `unknown-surface` in its `labels` array. Each tier gets a different advisory rendering.

Epistemic labels are measurement-only here: `[grounded]`, `[degraded]`, and `[확인 불가]` describe whether the tool measured the contamination fact. Semantic reuse caution is separate and uses `recommendation: warn-on-reuse`.

Current capability note: if `symbols.meta.supports.anyContamination !==
true`, pre-write emits a single capability note and renders semantic
contamination state as `[확인 불가, reason: producer did not emit
anyContamination capability]`. It must not call a candidate clean merely
because the annotation is absent.

| `labels` includes                                 | Lookup rendering                                                                                                                                                                                                                           |
| ------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| none (annotation omitted, capability true)        | `[grounded, ...]` — clean reuse candidate.                                                                                                                                                                                                 |
| `unknown-surface` only                            | `[grounded structural, semantic caution: unknown-surface]` — `unknown` is a safe boundary type, NOT contamination. Advisory notes that reuse requires narrowing via type guards; candidate is fine.                                        |
| `has-any` only (no `any-contaminated` escalation) | `[grounded structural, any signal present, semantic caution: mild `any` occurrence]` — single passing occurrence (e.g. one `Record<string, any>` field). Reuse is viable but the advisory surfaces the measurement so Claude can weigh it. |
| `any-contaminated` or `severely-any-contaminated` | `[grounded, anyContamination.label = ..., measurements = ...]` plus `[recommendation: warn-on-reuse, confidence: low, reason: <label> semantic reuse caution]` — measurement remains grounded; reuse carries real type-safety loss.        |

The advisory text MUST surface the raw measurements from `anyContamination.measurements` (ratio, counts) for every non-clean tier, not just the label — scale is the signal Claude needs (see any-contamination.md §11 "Honesty requirements").

**Never call `unknown-surface` "contaminated"** and **never apply `warn-on-reuse` to a `has-any`-only or `unknown-surface`-only candidate**. Doing so teaches the wrong type-system lesson (`unknown` is GOOD boundary design) and inflates the caution signal past the point Claude can act on it.

### Step 4 — Emit pre-write advisory

Output the advisory to Claude before it writes. Format in §5.

### Step 5 — Claude writes (or doesn't)

Claude may:

- Adopt a pre-existing item (quotes the citation in its own message).
- Proceed with a new item (cites override reason).
- Ask the user for clarification when an existing item is near-but-not-quite.

The gate is advisory; Claude retains authorial authority.

### Rust Lane Migration

`lumin-rust-analyzer pre-write` is the Rust execution surface for Rust source
intents. It consumes typed `rust-source-health` evidence in memory. It does not
run Cargo metadata or Cargo check because the current cargo/rustc oracle can
verify diagnostics and clean build scope but cannot prove that an unreferenced
repository name, shape, file, inline pattern, or dependency consumer is absent.
The public audit orchestrator enters this route through explicit Rust
selection, currently `audit-repo.mjs --pre-write --rust-pre-write`, or through
`audit-repo.mjs --pre-write --pre-write-engine auto` when the intent transport
explicitly declares `language: "rust"`. Filename, dependency, and repository
shape inference are not part of this contract. `intent.language` is an
orchestrator route selector, not a Rust analyzer schema field; the audit
orchestrator strips it before invoking `lumin-rust-analyzer pre-write` so Rust
keeps its typed `deny_unknown_fields` boundary. JS audit scan-scope flags
`--production` / `--exclude-tests` and repeated `--exclude <pattern>` are
forwarded to `lumin-rust-analyzer pre-write`. Rust source-health owns the Rust
file enumeration semantics for that route, applies Rust test-like path policy
and normalized excludes before analysis, and records the effective scope under
the source-health input metadata (`includeTests`, `exclude`, and `pathPolicy`).

The Rust-native lookup artifact is not the lifecycle advisory. The analyzer
writes `rust-pre-write-artifact.<invocationId>.json`; the audit orchestrator
wraps that result in the standard `pre-write-advisory.<invocationId>.json`
shape so post-write can reuse the checked invocation, scan-range,
file-inventory, and planned-file delta contract. Post-write must receive the
standard advisory path, not the native Rust artifact path.

The wrapper is fail-closed at this boundary. Before invoking the analyzer it
removes stale current-run native/advisory targets. A successful child is not
enough: the native artifact must declare `rust-pre-write.v1`,
`prewrite-token-policy-v1`, producer `lumin-rust-analyzer`, complete typed
intent-lane coverage, and every required evidence array. Missing, malformed,
or contract-mismatched output produces a non-zero lifecycle block and no
replacement advisory or native `latest` file.

The JS/TS pre-write owner remains the execution surface for JS/TS source
intents. The Rust command replaces the corresponding lanes only for Rust source
input:

- name lookup;
- file lookup;
- exact shape-hash lookup;
- function-signature lookup;
- Cargo dependency declaration/import lookup;
- inline extraction pattern lookup;
- planned type-escape declaration preservation.

The Rust file lane follows the JS/TS P1-2 file lookup semantics with
Rust-owned evidence:

- `rust-source-health.files[path]` present -> `FILE_EXISTS`;
- safe repo-relative `.rs` path absent from `files` and `skippedFiles`, under
  the source-health path policy -> `NEW_FILE`;
- skipped files, non-Rust paths, excluded `target` / `vendor` paths, and unsafe
  path text -> `FILE_STATUS_UNKNOWN`;
- boundary status is always `NOT_EVALUATED` because Rust pre-write intent does
  not carry planned `from -> to` edges.

The Rust command accepts the checked pre-write intent transport. Missing arrays
default with warnings and malformed present fields hard-stop. `taskId` is the
only typed transport extension. Unknown extra fields hard-stop instead of
reopening a dynamic `Value` map at the Rust boundary. Planned type escapes are
validated and preserved as intent evidence; Rust does not invent a TypeScript
`any` equivalent or post-write type-escape extractor.

Rust shape intent lookup follows the JS/TS P4 discipline with Rust-owned facts:
exact `shape.hash` matches `rust-source-health.files[*].ast.shapeHashes[]`, and
function-signature hashes match
`rust-source-health.files[*].ast.functionSignatures[]`. Field-only shapes and
TypeScript `typeLiteral` strings remain unavailable rather than guessed.
Unmatched exact hashes are grounded `NOT_OBSERVED` only when the Rust syntax
input is complete: no parse errors, no skipped files, and no review-visible
opaque surfaces.

Rust dependency intent lookup uses `Cargo.toml` declarations and
`rust-source-health` Rust path evidence. Cargo declarations are package/member
scoped. A dependency declared by one workspace member does not make another
member's consumer `DEPENDENCY_AVAILABLE` unless that member declares or inherits
the dependency itself. Missing or malformed root Cargo manifests hard-stop this
lane because there is no grounded declaration source.

Rust inline extraction lookup consumes
`rust-source-health.files[*].ast.inlinePatterns[]`. Explicit
`refactorSources[]` declarations can produce `INLINE_PATTERN_MATCH` or
`NO_INLINE_PATTERN_MATCH` only when the requested source files are parsed; if a
source is missing, skipped, or parse-failed, the lane emits unavailable
evidence instead of an absence claim.

## 4. Minimum script subset for fresh audit

The JS/TS name, file, and dependency lanes use the Rust-owned
`js-ts-pre-write-evidence` command. For normal repository runs Rust discovers
the checked scan-range file list with the `scan_scope.rs` mirror, then reads and
parses those files once with OXC and returns the compact
`symbols` and `topology` evidence needed by pre-write plus the occurrence-level
`any-inventory.pre.<invocationId>.json` baseline required by post-write. The
command does not write or require repository-sized `symbols.json` or
`topology.json` artifacts, and those Rust-owned lanes do not load the Node
`oxc-parser` binding. Dependency consumer projection is restricted to the normalized package
roots requested by the intent so unresolved aliases cannot masquerade as
package imports. Failure is a pre-write failure; JS must not run a fallback
extractor or classifier. Per-file parse failures remain explicit incomplete
evidence and prevent absence claims; unreadable required files hard-stop.
Explicit path-backed file requests remain available for contract probes and
focused callers, but JS must not walk the repository before a normal fresh
pre-write. This Rust-only discovery exception avoids repeated Node/DrvFS
directory-entry crossings on WSL while preserving the checked JS scan policy.
The shared Rust pass may skip OXC parsing for files whose exact current content
and extraction context match its strict per-file cache. It still discovers the
current scoped file set and rebuilds the complete before or after evidence
artifact every time. Every scoped file is read from the current worktree and
identified by SHA-256 of those exact bytes. Cache misses parse that same byte
buffer. Git index/blob bytes are never used as source identity or parser input
because clean/smudge filters, Git LFS, and working-tree encodings can make them
different from the file being reviewed.
For the same mounted-checkout case, the JS audit-core bridge may execute the
packaged `win32-x64/lumin-audit-core.exe` for `js-ts-pre-write-evidence` so
repository discovery and source reads use native NTFS instead of DrvFS. This
route is limited to x64 WSL, a root and cache directory that both translate
losslessly from `/mnt/<drive>/...` when incremental reuse is enabled, and a
Windows helper that reports the exact current runtime contract. With
incremental reuse disabled, the cache root is not a host-route prerequisite;
result transport uses a shared Windows temp directory. Explicit Linux/generic
audit-core overrides keep their documented precedence and disable host routing.

The bridge translates only absolute transport paths (`root`, `cacheRoot`, and
the temporary result path). Repository-relative file/evidence identities and
artifact semantics remain Rust-owned and unchanged. Returned root/cache paths
are translated back to the caller's WSL spelling before validation. Candidate
absence or contract mismatch selects the normal Linux helper before execution;
after the Windows command starts, non-zero exit, missing result output, or
null/malformed evidence is a hard failure and must not retry another analyzer.
Packaged Windows helpers retain executable mode for WSL installs on Linux
filesystems. Explicit path-backed source files are canonicalized and must stay
within the canonical request root before any source content is read.
Added, deleted, and renamed paths change the source-set fingerprint and
invalidate cached relative-resolution facts. Cache corruption or identity
mismatch reparses current source and is reported as a miss; it never produces
empty evidence or selects the legacy JS extractor. Consequently the first run
still pays full discovery/read/parse cost, while a fresh post-write snapshot and
later pre-write runs may reuse only proven unchanged per-file parse facts.
The public CLI skips Node analysis dependency setup for pre-write-only and
post-write-only invocations. Normal fresh post-write obtains its after type-
escape inventory and file list from the same Rust command; it does not load the
Node parser package. An explicit base profile, Rust analysis, SARIF, or canon
request keeps the normal dependency guard because those paths still run
JS-owned producers.

Run only what the requested JS/TS intent items require:

- Name lookup → compact Rust pre-write evidence.
- Shape lookup → `build-shape-index.mjs` when the intent includes shape entries. Grounded matches still require an exact `shape.hash` or supported `shape.typeLiteral`. If the intent has only loose field names, or if no validated `shape-index.json` is available, emit `[확인 불가]` rather than a fuzzy match.
- File lookup → compact Rust pre-write evidence. Do not run
  `triage-repo.mjs`: the current intent shape has no planned edge endpoints,
  so triage cannot change the required `NOT_EVALUATED` boundary result.
- Dependency lookup → package.json direct read + compact Rust pre-write evidence
  for observed static package-import consumer counts. This lane is for
  package specifiers, not relative/internal module paths. If the symbol
  evidence is unavailable or lacks `meta.supports.dependencyImportConsumers`,
  report the import count as unavailable; never render it as `0 observed
consumers`.

For Rust pre-write, do not build `symbols.json`, `shape-index.json`, or
`function-clones.json` to answer Rust source intents. Run
`lumin-rust-analyzer pre-write`; it obtains Rust name, file, shape, signature,
dependency, and inline evidence from typed Rust source-health facts in memory.
The outer JS audit orchestrator may still own lifecycle dispatch and advisory
packaging until the public command route is migrated, but it must not treat the
JS/TS artifacts above as Rust absence evidence.

Never run the full pipeline for a pre-write. Latency budget: < 5 seconds total in warm-cache case, < 30 seconds in cold-cache.

## 5. Output format

Pre-write output is consumed by Claude, not by the user. Structured for Claude to cite directly.

The invocation-specific `pre-write-advisory.<invocationId>.json` is the complete
current-run advisory and the authoritative cue/lookup surface. Production
result-file lifecycle commands emit only a constant-shape terminal handoff:
the invocation-specific path plus complete counts for cue cards, suppressed
cues, lookups, unavailable evidence, drift, and planned type escapes. They must
not stream one terminal row per candidate or lookup. Repository-sized terminal rendering can
fill a caller pipe after the advisory files are complete and prevent the
result-file caller from recovering the finished run. Agents read the
invocation-specific JSON selectively when the handoff counts are non-zero.

The expanded example below describes advisory content. It is not a requirement
that the result-file command duplicate every row on stdout.

```
## pre-write advisory (canonical/pre-write-gate §5)

### Already exists (reuse candidates)

- `formatTimestamp` — not observed in scan range.
  [확인 불가, scan range: TS/JS production files ex tests, symbols.json absent → fresh build-symbol-graph pass, no `formatTimestamp` identity found]

- `formatDate` — EXISTS at `src/utils/date.ts::formatDate`.
  [grounded, symbols.json.defIndex['src/utils/date.ts']['formatDate'] present, fan-in = 8]
  Reuse candidate. Shape: `(Date) => string`.

### Already exists — but any-contaminated (reuse with warning)

- `UserData` — EXISTS at `src/types/User.ts::UserData`.
  [recommendation: warn-on-reuse, confidence: low, reason: severely-any-contaminated semantic reuse caution; measurement remains grounded]
  [grounded, type-owner.anyContamination = {label: "severely-any-contaminated", labels: ["has-any", "severely-any-contaminated"], measurements: {totalFields: 7, anyFields: 6, unknownFields: 0, anyFieldRatio: 0.85, indexSignatureAny: false}}]
  ⚠ Candidate is severely any-contaminated (85% any fields). Reusing it will not transfer type safety; consider writing a typed version or tightening the candidate first.

### New code candidates

- New file `src/utils/time.ts` — would be new.
  [grounded, topology.json.nodes does not contain 'src/utils/time.ts'; topology.meta.complete = true]
  Boundary rule: `src/utils/*` has zero cross-submodule inbound; safe to add.

### Watch-for

- Shape `{ year, month, day, hour }` — exact shape lookup unavailable from field names alone.
  [확인 불가, reason: shape intent lacks exact sha256 shape hash or supported typeLiteral; field names alone are not structural equality evidence]
  P4 shape lookup can emit grounded matches only from a validated `shape-index.json` and an exact hash or supported `typeLiteral` normalized by the P4 shape-hash producer.

- DOMAIN_CLUSTER_DETECTED — planned `lib/cardNewsService.js` shares prefix `lib/cardNews*` with 9 existing files.
  [grounded, topology.json.nodes matched 9 files with prefix 'lib/cardNews*']
  Recommendation: inspect the existing domain cluster before creating a parallel owner file.
- DOMAIN_CLUSTER_DETECTED — planned `_engine/lib/artifact-loader.mjs` shares domain token `artifact` with existing `*-artifact.mjs` siblings.
  [grounded, topology.json.nodes matched N files with domain key 'artifact' in '_lib']
  Recommendation: inspect the existing domain cluster before creating a parallel owner file.

### Planned type escapes (from Step 2 intent)

- 1 escape planned: `as unknown as ThirdPartyShape` at `src/vendor/wrapper.ts::adaptResponse`.
  Reason declared: upstream SDK lacks type exports; wrapper narrows at the boundary.
  [grounded, intent extracted at pre-write Step 2; will be checked against observed escapes in post-write per any-contamination.md §6 Stage 2]

Or, when none are planned:

- 0 escapes planned. Post-write will treat any observed `type-escape` (every `escapeKind` enumerated in `canonical/fact-model.md` §3.9 — `explicit-any`, `as-any`, `angle-any`, `as-unknown-as-T`, `rest-any-args`, `index-sig-any`, `generic-default-any`, `ts-ignore`, `ts-expect-error`, `no-explicit-any-disable`, `jsdoc-any`) as a silent introduction per any-contamination.md §6 Stage 2.
```

Sections:

- **Already exists** — each intent item looked up. `EXISTS` entries carry reuse hints. `not observed` entries carry `[확인 불가]` with scan range.
- **New code candidates** — intent items that would legitimately be new (`NEW_FILE` with `topology.meta.complete: true`, or `NEW_PACKAGE`). Heading is neutral — a `FORBIDDEN` boundary sub-line here is a legitimate warning, not a self-contradiction. P1-2 cannot emit `ALLOWED` / `FORBIDDEN` boundaries without planned `from → to` edges, so most P1-2 rows show `boundary: not evaluated` with `[확인 불가]`.
- **Watch-for / search hints** — shape-matches, domain clusters, boundary-adjacent cases, feature envy risks, and intent-token name hints. Advisory, not blocking. Intent-token hints are degraded search cues, not grounded reuse claims.
- **Planned type escapes** — the Step 2 declared-escape list, echoed so post-write has an anchor for the planned-vs-observed delta. Empty-list form is the default; any non-empty entry must carry a declared reason.

## 6. What pre-write does NOT do

- Does not block Claude from writing. Advisory only.
- Does not rewrite Claude's plan. Claude reads the advisory and decides.
- Does not verify the user's intent matches the advisory. That's user's judgment.
- Does not run every script. Only the minimum subset for the intent.
- Does not emit a "fix this" message to the user. The user sees Claude's final output; the advisory is internal.

## 7. Confidence and scan-range rules (inheriting from invariants.md)

Every `EXISTS` / `not observed` line MUST carry a citation.

For `EXISTS`:

- Cite the artifact field and the identity (`symbols.json.defIndex[file][name]`, shape hash, etc.).
- If confidence < high (resolver blindness > 15%, parse errors present, staleness > N days), demote the line:
  - `EXISTS` → `LIKELY EXISTS [degraded, confidence: medium]`.
  - Caller notes the degradation.

For `not observed`:

- ALWAYS `[확인 불가, scan range: ...]`.
- Never say "does not exist" — the scan is bounded, external use is not ruled out.

## 8. How pre-write interacts with canonical/ when present

If the repo has a `canonical/` directory (user or LLM-maintained):

- Read owner claims from `canonical/helper-registry.md`, `canonical/type-ownership.md`, etc.
- Use them as the **first** source of truth for "already exists".
- If canonical and current AST disagree, emit a drift warning in the advisory (not a block):
  - `CANONICAL DRIFT: canonical/type-ownership.md:L42 lists owner as X; current AST observes Y.`
- Full drift analysis is P5 (`check-canon.mjs`, separate spec); pre-write only flags obvious name/owner mismatches.

## 9. Why pre-write, and not post-write alone

Post-write catches mistakes after the fact. Pre-write prevents them. For vibe-coder repos, the cost of post-write-only is that every session leaves residue even when the error is found — Claude wrote it, got told, now has to undo. Pre-write shortens the loop.

Both are needed (P2 exists for what pre-write missed). But pre-write is the primary defense.
