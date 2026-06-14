# FP-41 Sentinel — partner-export graph sub-spec

> **Role:** sub-spec for FP-41 follow-up items §4.3 (framework sentinel) + §4.4 (allowlist / partner export) from `docs/history/FP-41-regression.md`. The core FP-41 JSX-identifier counter fix already landed (see memory `project_grounded_audit.md` "FP-41 fix LANDED" section). This spec covers the REMAINING cases the counter fix does NOT address.
> **Status:** design draft, v1 (spec only — implementation deferred pending reviewer approval + dogfood evidence).
> **Last updated:** 2026-04-20

---

## 1. Boot

| File | Why |
|---|---|
| `docs/history/FP-41-regression.md` | Parent regression report. §4.3–§4.4 define the sub-spec boundary. §2 mechanism. §6 explicitly states "do NOT add framework sentinel as the primary fix" — the counter must be accurate first (it is, post-fix). |
| `_lib/classify-facts.mjs::countFileReferencesAst` | Current reference counter. JSX-aware after FP-41 fix. This sub-spec does NOT modify the counter — sentinel layer runs AFTER counting. |
| `classify-dead-exports.mjs:193` | Tier mapping (`occ === 0 → C`). Sentinel adjusts tier AFTER initial classification, does NOT change the mapping itself. |
| `_lib/vocab.mjs` | `TAINT` / `EVIDENCE` / `provenanceFields` constants. Sentinel emits new provenance tag (`'framework-sentinel-partner'`) — added here when implementation lands. |
| `_lib/ranking.mjs::tierForFinding` | Per-finding tier decision. Sentinel provenance consulted at this stage. |

Not required for this sub-spec: `canonical/*.md` (sentinel is an implementation heuristic, not a canonical rule), `docs/history/phases/p2/*.md` (sentinel is independent of post-write delta).

## 2. Scope

### 2.1 Problem this sub-spec closes

After the FP-41 counter fix (`classify-facts.mjs` walker now counts `JSXIdentifier` + `JSXMemberExpression`), **same-file compound patterns classify correctly**: `<AlertDialogTrigger>` used inside `AlertDialog`'s render produces `fileInternalRefs.valueRefs === 1` → Tier A. That case is closed.

Remaining over-escalation surface:

1. **Cross-file compound components.** shadcn/ui's common pattern splits compound exports across sibling files — `dropdown-menu.tsx` exports `DropdownMenu`, `dropdown-menu-trigger.tsx` exports `DropdownMenuTrigger`, `dropdown-menu-content.tsx` exports `DropdownMenuContent`. Each file has zero file-internal JSX use; each import is through a barrel. The counter sees external fan-in via the barrel re-export, but if barrel detection drops the finding (pre-R-8) or if any one component lacks an external import outside the barrel's self-closure, it falls to Tier C despite being live-by-association.
2. **Higher-order wrapper relationships.** `export const Memoized = memo(InnerComponent)` — `InnerComponent` has a file-internal AST identifier reference (good, counter catches it), but if the HOC wrapper is the only liveness link, the inner has exactly 1 ref → Tier A — which may be correct, but the policy intent ("can I delete the export?") gets ambiguous: the wrapper requires the inner's function identity; renaming or unexporting the inner breaks the wrapper's source semantics but not its runtime.
3. **Type-only partner exports.** `export type AlertDialogProps = ComponentProps<typeof AlertDialog>`. The type reference to `AlertDialog` is file-internal; the type export itself is what downstream imports. If `AlertDialogProps` has no external consumer AND `AlertDialog` is the only thing referring to it internally, the current classifier may escalate incorrectly.
4. **forwardRef / displayName patterns.** `export const Foo = forwardRef<T, P>((props, ref) => ...)`. The wrapped inner function may be anonymous, leaving the outer binding's only internal reference at the displayName side-effect statement `Foo.displayName = 'Foo'`. Counter sees 1 ref via the assignment LHS (skip position) and 1 ref via the RHS name-match. Edge case — pin via test.

### 2.2 In scope (this sub-spec)

- **Cross-file partner-graph detection** — build a lightweight "partner set" from same-directory + same-prefix + mutual-JSX-use signals. See §4.
- **Tier-adjustment provenance** — new `taintedBy: 'framework-sentinel-partner'` signal (or a positive `supportedBy: 'partner-of-live-sibling'` — §7 debates both). Sentinel does NOT write to `fileInternalUses` or `fileInternalRefs`; it annotates provenance only. Raw counts stay ground-truth.
- **Opt-in policy** — sentinel is **advisory by default**: adds provenance, does NOT automatically flip tiers. A follow-up flag `--apply-framework-sentinel` (or equivalent) enables the tier flip. Rationale: heuristics can hide real dead code; sentinel should expose its reasoning before acting on it.
- **Corpus cases** — two new `CASE-FP41-CROSS-FILE` + `CASE-FP41-HOC` pinning the detection without requiring the tier flip (apply-flag off by default).

### 2.3 Out of scope

- **Modifying the counter.** Counter stays as-is (JSX-aware post-FP-41-fix). Sentinel runs after counting, annotates provenance.
- **Resolving HOC wrappers to their inner.** `memo(X)` does NOT mean "X's tier inherits from memo(X)" — that's value-flow analysis beyond sentinel scope.
- **Type-only partner detection.** Type graph is a separate analysis (shape-hash P4 is the right home for cross-type partnerships, not sentinel).
- **Auto-applying tier flips by default.** Explicit flag required.
- **Renaming or reorganizing existing tier labels.** A / B / C stay unchanged; sentinel adds provenance inside them.
- **Cross-package / monorepo partnership detection.** Single-package scope; `partner-graph` edges cross FILES but not PACKAGES.

## 3. Dependencies

### 3.1 What sentinel reads

- `symbols.json.defIndex` — enumerated exports per file.
- `symbols.json.uses[]` — cross-file use edges (value + type).
- `symbols.json.fanInByIdentity` — external fan-in per identity.
- `dead-classify.json.entries[]` — initial tier assignment per finding (before sentinel runs).
- `repoMode.workspaceDirs` — workspace boundaries (sentinel does not cross them).

### 3.2 What sentinel writes

New fields on each `dead-classify.json.entries[]` row that the sentinel matches:

```ts
{
  // existing fields unchanged: symbol, file, tier, fileInternalUses, ...
  sentinel: {
    kind: 'framework-partner',
    evidence: Array<{
      kind: 'naming-prefix' | 'file-colocation' | 'mutual-jsx-use' | 'barrel-coexport',
      detail: string,   // e.g. "prefix 'AlertDialog' shared with 3 siblings"
    }>,
    livePartner: string | null,  // e.g. "AlertDialog" — the live export this finding is partnered with
    wouldReclassifyTo: 'A-remove-export' | null,   // what the sentinel would suggest IF --apply-framework-sentinel
  } | null
}
```

`sentinel === null` when no partnership detected. `sentinel` present but `wouldReclassifyTo === null` means partnership detected but the sentinel declined to recommend a flip (e.g., evidence count too low).

### 3.3 No changes

- `_lib/classify-facts.mjs` — counter is fixed and complete for JSX. No edits.
- `_lib/ranking.mjs` — tier-assignment rules unchanged unless `--apply-framework-sentinel` (and even then, the rule only consults `sentinel.wouldReclassifyTo`).
- `classify-dead-exports.mjs:193` tier boundaries — unchanged.
- `canonical/*.md` — sentinel is implementation heuristic, not canonical rule.

## 4. Partner-graph construction

The sentinel builds a "partner graph" during post-classify phase, AFTER `dead-classify.json` is computed. Edges run between exported symbols; weights reflect evidence strength.

### 4.1 Edge signals

1. **Naming prefix.** `AlertDialog` / `AlertDialogTrigger` / `AlertDialogContent` share the prefix `AlertDialog`. Weight: `1 / (1 + prefix_rank)` where `prefix_rank` is the symbol's position in the sorted prefix group (shortest name = root = rank 0, tiebreak by file+line).
   - Minimum shared prefix length: 4 characters (avoids `A` + `AB`-style noise).
   - Prefix must be a valid identifier start (starts with uppercase for components; lowercase for hooks like `useForm` / `useFormField`).

2. **File co-location.** Partners live in the same directory, OR in sibling files under a shared parent with correlated filenames:
   ```
   src/ui/alert-dialog.tsx                (AlertDialog)
   src/ui/alert-dialog-trigger.tsx        (AlertDialogTrigger)
   src/ui/alert-dialog-content.tsx        (AlertDialogContent)
   ```
   Weight: `1.0` for same-dir + kebab-case filename prefix match; `0.5` for same-dir without filename correlation; `0` otherwise.

3. **Mutual JSX use.** `AlertDialog` renders `<AlertDialogTrigger>` (cross-file JSX reference exists in `symbols.uses`). This is the strongest signal — the counter sees it, but for CROSS-FILE references, the counter counts it as external fan-in on the trigger (which is good) but doesn't know about the partnership semantics.
   - Weight: `2.0` per mutual-use edge.

4. **Barrel co-export.** `index.ts` re-exports all of them together via `export * from './alert-dialog'` + `export * from './alert-dialog-trigger'` etc. Weight: `1.5`.

### 4.2 Edge aggregation

Per candidate finding (symbol marked Tier C with `fileInternalUses === 0` AND external fan-in within-workspace only):

```
partnerScore(finding) = sum of weights for every edge between `finding.symbol`
                        and ANY live export (Tier != C) in the repo.
```

Threshold: `partnerScore >= 3.0` → mark as partner; record the highest-scoring `livePartner`.

Rationale for 3.0: one signal alone (e.g., naming prefix) is too weak; two signals (naming + co-location) OR one strong signal (mutual-use) + any other should trigger. Threshold tunable; pin at 3.0 for v1, reassess after dogfood corpus.

### 4.3 What "live export" means for partnership

An export is "live" in the partnership sense if:
- `tier !== 'C-completely-dead'` in the INITIAL classification (pre-sentinel), AND
- `externalFanIn > 0` (i.e., at least one cross-file consumer outside the workspace-internal reference graph), OR
- `fileInternalUses > 0` AND within a file that has a live external consumer transitively.

Sentinel does NOT bootstrap its own liveness determination — it consumes the classifier's output. This keeps the sentinel a pure post-processor with no circular dependency on itself.

## 5. Detection rules

### 5.1 Pipeline stage

`rank-fixes.mjs` currently runs after `classify-dead-exports.mjs`. Sentinel runs as a new `_lib/framework-sentinel.mjs` module invoked:

- After `classify-dead-exports.mjs` emits `dead-classify.json`.
- Before `rank-fixes.mjs` consumes it.
- Standalone invocation: `framework-sentinel.mjs --root <repo> --output <dir>` (for dogfood / test).

### 5.2 Algorithm (pseudocode)

```js
function runFrameworkSentinel({ symbols, classify, repoMode }) {
  const liveExports = new Set();
  for (const entry of classify.entries) {
    if (isLive(entry)) liveExports.add(`${entry.file}::${entry.symbol}`);
  }

  for (const entry of classify.entries) {
    if (entry.tier !== 'C-completely-dead') continue;
    if (entry.fileInternalUses > 0) continue;  // JSX fix already caught these

    const candidates = buildPartnerEdges(entry, classify, symbols, liveExports);
    const score = aggregateScore(candidates);
    if (score < SENTINEL_THRESHOLD) {
      entry.sentinel = null;
      continue;
    }

    const topPartner = candidates.sort((a, b) => b.weight - a.weight)[0];
    entry.sentinel = {
      kind: 'framework-partner',
      evidence: candidates.map(c => ({ kind: c.kind, detail: c.detail })),
      livePartner: topPartner.liveSymbol,
      wouldReclassifyTo: 'A-remove-export',
    };
  }

  return classify;
}
```

### 5.3 CLI flag gating

```
classify-dead-exports.mjs [no new flag; always emits sentinel provenance]
rank-fixes.mjs --apply-framework-sentinel   # NEW — off by default
```

When `--apply-framework-sentinel` is set, `tierForFinding` in `_lib/ranking.mjs` consults `finding.sentinel?.wouldReclassifyTo` and returns that instead of the original tier. When unset, `sentinel` provenance is emitted but tier stays at whatever classify-dead-exports assigned.

## 6. Sequence (test-first, 4 steps)

### 6.1 Step 1 — `_lib/framework-sentinel.mjs` pure module

**Test first:** `tests/test-framework-sentinel.mjs`. Cases:

- Empty input → empty output.
- Single Tier C finding with no siblings → `sentinel === null`.
- Cross-file partner fixture (AlertDialog + AlertDialogTrigger in sibling files, mutual JSX use) → sentinel emitted on Trigger with `livePartner === 'AlertDialog'`.
- Naming-only partner (prefix match but no JSX use) → `partnerScore < 3.0` → `sentinel === null` (just naming isn't enough).
- Barrel re-export fixture (`index.ts` re-exports both) → weighted edge contributes.
- HOC wrapper fixture (§2.1 case 2) — record behavior but don't pin a flip (scope excludes).

Impl: pure function `buildSentinelProvenance({symbols, classify, repoMode})`. No I/O. Returns the same `classify` object with `.sentinel` fields populated.

**Exit:** ~15 assertions, sentinel provenance emitted deterministically.

### 6.2 Step 2 — orchestrator integration

Wire the sentinel into the existing pipeline between `classify-dead-exports.mjs` and `rank-fixes.mjs`.

Option A: new standalone `framework-sentinel.mjs` CLI script that reads `dead-classify.json`, calls the lib, and writes `dead-classify.json` in place (atomic temp+rename).

Option B: inline into `classify-dead-exports.mjs` as a final pass before write.

**Recommendation: A.** Keeps `classify-dead-exports.mjs` responsible for its core job; sentinel is a clearly-labeled post-processor with its own test surface and its own CLI for dogfood. Matches the existing pipeline philosophy (each concern = one script).

**Test first:** CLI smoke test + `audit-repo.mjs` integration (sentinel runs as step 8.5 between classify and rank-fixes).

**Exit:** ~8 assertions + `audit-repo.mjs` step list includes the new producer.

### 6.3 Step 3 — `--apply-framework-sentinel` flag

Extend `rank-fixes.mjs`:
- Flag parsed via `parseCliArgs`.
- When set, `tierForFinding` consults `finding.sentinel?.wouldReclassifyTo`.
- When unset, sentinel provenance is informational only.

**Test first:** two paired test cases — same fixture, one WITH the flag (Trigger classifies as Tier A), one WITHOUT (Trigger stays Tier C). Pins the gating.

**Exit:** ~6 assertions.

### 6.4 Step 4 — corpus case + dogfood

- New `CASE-FP41-CROSS-FILE` in `tests/test-corpus.mjs`:
  - AlertDialog + AlertDialogTrigger in sibling files.
  - App imports only AlertDialog.
  - PRE-SENTINEL: Trigger is Tier C.
  - POST-SENTINEL (info only, no flag): Trigger has `sentinel.livePartner === 'AlertDialog'`, `wouldReclassifyTo === 'A-remove-export'`, tier STILL C.
  - WITH flag: Trigger is Tier A.
  - `FP_BUDGET = 0` unchanged.
- Dogfood: run `framework-sentinel.mjs` on duyet (or this skill) + report how many findings the sentinel proposes to flip. Do NOT apply the flip without human review for dogfood round 1.

**Exit:** corpus green, dogfood findings reviewed (expected ~0 on this skill's own repo; ~20+ on duyet based on FP-41 regression report).

## 7. Known risks

- **False positives — masking legitimate dead code.** A partnered-looking finding might actually be stale. Example: `AlertDialog` is live + `AlertDialogLegacy` (orphaned predecessor) shares the naming prefix. The sentinel's naming signal would weigh toward partnership. Mitigation: threshold of 3.0 requires a second signal (co-location + mutual use); pure-naming partnership (weight ≤ 1.0) never triggers alone.
- **Weight tuning drift.** The edge weights (1.0 / 0.5 / 2.0 / 1.5) and threshold 3.0 are chosen by design-time reasoning, not dogfood-tuned. First corpus + dogfood round WILL surface adjustments. Pin v1 weights in a module-level constant with a comment linking to this spec; raise weight changes as explicit PR topics.
- **Evidence explosion in large repos.** Naming-prefix edges are O(n²) naively. Implementation must bucket-by-prefix first (`Map<prefix, Symbol[]>`) before pair-checking. Pin algorithmic complexity in the test.
- **Sentinel advisory by default.** `--apply-framework-sentinel` off is the safe default. Risk: users miss the flag and keep seeing "dead" findings the sentinel thinks are partnered. Mitigation: rank-fixes.mjs console output lists "N findings have sentinel partnership evidence; pass --apply-framework-sentinel to reclassify" as a hint. Documented in SKILL.md.
- **Overlap with shape-hash (P4).** Shape-hash identifies structurally-duplicate exports (content-level). Sentinel identifies semantically-partnered exports (relationship-level). They're orthogonal but a future unification might collapse them. Document the orthogonality in this spec and in the P4 spec when written.
- **HOC + forwardRef NOT in scope.** §2.1 cases 2, 4 are noted but deferred. Risk: those cases remain Tier C when they should be Tier A. Mitigation: JSX counter fix already covers the dominant case (compound components). HOC patterns are rarer and can be addressed in a separate sub-spec if dogfood shows real-world prevalence.
- **Type-only partners NOT in scope.** §2.1 case 3 deferred to P4 (shape-hash).

## 8. Reviewer checklist when this sub-spec implementation closes

- `_lib/framework-sentinel.mjs` exists, pure, no I/O. Source grep: no `readFileSync` / `writeFileSync` inside the module.
- `dead-classify.json.entries[]` rows carry `sentinel` field (possibly null).
- `sentinel.evidence[]` carries at least one edge per matched finding.
- `sentinel.livePartner` is always a symbol::file that exists in `symbols.defIndex`.
- `sentinel.wouldReclassifyTo === 'A-remove-export'` for all matched findings (v1 scope — no `C → B` flips).
- Naming-alone partnership (weight ≤ 1.0) does NOT trigger.
- Threshold is a module-level constant with doc comment linking to this spec.
- `--apply-framework-sentinel` off by default. When off, tier assignment unchanged vs pre-sentinel run.
- When on, tier consults `sentinel.wouldReclassifyTo` and the pinning test verifies C → A flip.
- Corpus case `CASE-FP41-CROSS-FILE` covers both "flag off" and "flag on" branches.
- `FP_BUDGET = 0` unchanged.
- Dogfood results documented in memory — number of findings proposed to flip, spot-check accuracy.
- SKILL.md updated with the flag + philosophy ("advisory by default, opt-in to apply").
- `update-test-doc.mjs` clean after registering the new test file.

## 9. Handoff to implementation session

When this sub-spec is approved:

1. Create a session file (`fp-sentinel/fp-sentinel-1.md` or similar) following the 4-step sequence in §6.
2. Implement test-first.
3. Dogfood on this skill's own repo + one external repo (duyet if available).
4. Reviewer round.
5. Land with `--apply-framework-sentinel` OFF by default so existing CI stays untouched.

**Escalation to its own phase:** if dogfood surfaces that cross-file compound patterns are far more prevalent than anticipated, OR if HOC / forwardRef / type-partner cases consume additional design effort, consider promoting this to `p3-sentinel/session.md` with its own sub-phases (cross-file / HOC / type-partner). For v1, single session is the target.

## 10. Non-goals

- No counter modification (counter is correct post-FP-41 fix).
- No canonical/*.md edits.
- No automatic tier flipping without explicit `--apply-framework-sentinel` flag.
- No HOC / forwardRef resolution.
- No type-only partnership detection.
- No cross-package partnership.
- No value-flow analysis.
- No precision claim without dogfood evidence — v1 sentinel ships with the threshold disclosed and the evidence enumerated, not with an accuracy number.
