# FP-41 — JSX identifier blindness in `countFileReferencesAst`

> **Role:** regression report for FP-41 discovered by external reviewer during duyet re-run (v1.9.11 vs v1.10.x in-progress snapshot).
> **Severity:** HIGH. ~50% of Tier A → Tier C migrations on duyet appear to be this one bug. Dead-classification correctness regresses on any TSX/JSX file with internal-use compound components.
> **Status:** mechanism confirmed from source; 35/70 over-escalation rate sampled by reviewer; pre-fix, no corpus case pinning it.
> **Filed:** 2026-04-20.

---

## 1. Summary

v1.10.0 P0 replaced regex-text reference counting with AST identifier reference counting in `_lib/classify-facts.mjs::countFileReferencesAst`. The AST walker examines only `Identifier` nodes. **JSX identifier nodes (`JSXIdentifier`, `JSXMemberExpression`) are not handled.** Any same-file JSX usage of an exported symbol is therefore invisible to the reference counter.

On duyet, this causes:

- External fan-in = 0 (no cross-file import) AND file-internal AST identifier count = 0 (JSX use silent) → `occ === 0` → Tier C ("completely dead").
- The same symbol in v1.9.11 was Tier A ("export 제거 가능, 1~2회") because the regex counter happened to match `<SymbolName` as a word-boundary occurrence, incidentally but correctly counting JSX usage.

Regression impact sampled by reviewer: **35 of 70 A→C migrations on duyet (50%) are over-escalations** driven by this mechanism. Typical pattern is shadcn/ui-class compound components (`AlertDialog` + `AlertDialogTrigger`, `CodeBlock` + `CodeBlockContainer` + `CodeBlockContent`).

## 2. Mechanism (grounded)

### 2.1 The counter ignores JSX

`_lib/classify-facts.mjs` line 209:

```js
function walk(node, parent, key) {
  if (!node || typeof node !== 'object') return;

  if (node.type === 'Identifier' && node.name === symbolName) {
    // ... count++ path
    return;
  }

  // recurse into child keys
  // ... walk(c, node, k)
}
```

`JSXIdentifier` has `type: "JSXIdentifier"`, not `"Identifier"`. The condition is strict equality on `"Identifier"`, so JSXIdentifier nodes pass through the condition, fall into the recursion block, and are never counted. `JSXMemberExpression` nodes wrap two `JSXIdentifier` children; same fate.

The recursion block at line 221–234 does descend into JSX subtrees — `JSXElement.openingElement.name` is visited — but the leaf node carrying the symbol name has the wrong `type` string, so no `count++` fires.

### 2.2 The tier mapping

`classify-dead-exports.mjs` line 193–197:

```js
if (occ === 0)      category = 'C-completely-dead';
else if (occ <= 2)  category = 'A-remove-export';
else                category = 'B-file-internal-hub';
```

`occ` is `astResult.count` when parse succeeds. JSX-only internal usage ⇒ `count === 0` ⇒ Tier C.

### 2.3 Why regex counter got it right incidentally

v1.9.11 used:

```js
const re = new RegExp(`\\b${IDENT_ESCAPE(name)}\\b`, 'g');
```

Word-boundary on raw text. `<CodeBlockContainer ...>` contains `CodeBlockContainer` surrounded by `<` and space — both non-word characters — so the regex matches. Regex had FP classes (comments, string literals, property keys — documented in classify-facts.mjs header), but on JSX identifier usage it happened to produce the correct count.

The AST rewrite fixed the FP classes but lost the accidental JSX coverage. Net accuracy on JSX-heavy repos regressed.

### 2.4 Tests do not exercise JSX

`tests/test-classify-facts-ast.mjs` line 38:

```js
const r = countFileReferencesAst(src, '/fake/test.ts', name, line);
```

Every test case passes `/fake/test.ts` — `.ts` extension, not `.tsx`. Grep for `JSX|jsx|tsx|React.createElement` in the test file returns zero matches. JSX code path has zero test coverage. The gap was not caught because it was never tested.

The file header (lines 16–20) explicitly flags "no scope-aware shadowing" as a known gap but does not mention JSX. Reviewer discovered this by running on a real TSX corpus (duyet), not from the tests.

## 3. Impact on duyet comparison

Reviewer's comparison matrix, v1.9.11 → v1.10.x in-progress:

| Tier | v1.9.11 | v1.10.x | Δ |
|---|---:|---:|---:|
| A (export 제거 가능) | 368 | 350 | −18 |
| B (file-internal hub) | 82 | 22 | −60 |
| C (completely dead) | 220 | 298 | +78 |

Total deadInProd unchanged at 712. usesResolved unchanged at 2,628. Resolver / graph numbers identical; only classification migrated.

Sample audit of 70 A→C migrations (reviewer read source for each):

- **35 migrations correct** — symbol had no external consumer AND no file-internal usage (regex FP cases cleared up by AST counter).
- **35 migrations over-escalated** — symbol had file-internal JSX usage by a live sibling (pattern: parent component renders child sub-components). v1.9.11 Tier A classification was correct; v1.10.x Tier C is wrong.

Reviewer's concrete reproduction: `ai-elements/` Vercel AI SDK vendored library's `CodeBlockContainer` + `CodeBlockContent`. `CodeBlock` is live (imported externally from `tool.tsx`). Inside `CodeBlock`'s render, `<CodeBlockContainer>` wraps `<CodeBlockContent />`. Both are Tier C in v1.10.x; both should be Tier A.

Same shape: shadcn/ui's `AlertDialog` + `AlertDialogTrigger` pattern.

## 4. Root causes (mapped to fix roles)

Reviewer's four diagnosis lines, keyed to the mechanism:

### 4.1 Scope-aware shadowing 미완성 — *this is the FP-41 driver*

v1.10.0 P0 shipped with "no scope-aware shadowing" as a self-documented gap. FP-41 is the adjacent gap: **no JSX identifier handling.** Both are subspecies of "the AST walker is incomplete vs the actual TS/JSX AST surface". The scope-aware gap affects mixed-scope files (rare in practice); the JSX gap affects every TSX file with internal-use compound components (extremely common).

Concrete fix: extend `walk()` and the counted-node condition so that:

- `JSXIdentifier` with `name === symbolName` counts, subject to the same skip rules that apply to `Identifier`.
- `JSXMemberExpression` is traversed into its `object` and `property` slots (both may be `JSXIdentifier`).
- The top-level name of `<Foo.Bar />` — i.e., `object = JSXIdentifier("Foo")` — counts against `Foo` when `Foo` is the symbol being counted.

Classification as a type vs value reference: JSX usage is a value position (it compiles to `React.createElement(Foo, ...)`). Increment `valueRefs`, not `typeRefs`.

### 4.2 Classification policy — *policy wording to tighten*

Tier C is defined in-code as "occurrence count is zero" (`classify-dead-exports.mjs` line 195). The definition works only if the counter is complete. Policy statement to add (comment + docs): **"Tier C requires both external fan-in 0 AND file-internal reference count 0, where file-internal reference counting is JSX-aware and future-scope-aware."** Without the qualifier, "count is 0" is a mechanical statement that drifts with counter precision.

Changelog entry should note: Tier boundary definitions depend on counter completeness; any precision-improving change to the counter should be paired with corpus case re-evaluation, not treated as pure refactor.

### 4.3 Framework sentinel — *defense in depth, not the primary fix*

shadcn/ui and compound-component patterns are identifiable from AST shape: exported functions/consts in a file where at least one live sibling renders them via JSX. A framework sentinel could elevate such partner exports from C back to A. Useful as a backstop (will help even when scope-aware + JSX-aware counting still has edge cases), but **should not replace the JSX counter fix**. Sentinels are heuristics; the counter is ground truth and must be accurate first.

### 4.4 Allowlist / partner export — *architectural, not for this fix*

Same-file exports linked via JSX/call dependency trees could inherit the parent's live status via a partner graph. This is the generalization of §4.3. Worth considering for a separate spec (post-1.10.0). Not required to close FP-41 — once the JSX counter is fixed, the symptom is gone without introducing a partner inference layer.

Priority order: **4.1 is the fix. 4.2 is the policy doc. 4.3 is optional backstop. 4.4 is future architecture, not blocking.**

## 5. Proposed fix

### 5.1 `_lib/classify-facts.mjs::countFileReferencesAst` — count JSX identifiers

Current condition:

```js
if (node.type === 'Identifier' && node.name === symbolName) { ... }
```

Extend to:

```js
if (node.name === symbolName && (
      node.type === 'Identifier' ||
      node.type === 'JSXIdentifier'
    )) {
  // Declaration / skip-position check as before.
  const nodeLine = lineOf(lineStarts, node.start ?? 0);
  if (nodeLine === declLine) return;
  if (isSkipPosition(parent, key, node)) return;

  // For JSXMemberExpression top-level name (`<Foo.Bar />`), the `property`
  // slot is `Bar` and the `object` slot is `Foo`. Both are JSXIdentifier.
  // The `property` position names a sub-component, not our symbol (the
  // same semantic as MemberExpression.property on non-computed access).
  if (parent?.type === 'JSXMemberExpression' && key === 'property') return;

  count++;
  if (isTypeContext(parent)) typeRefs++;
  else valueRefs++;  // JSX usage is a value reference
  return;
}
```

Additions to `isSkipPosition(parent, key, parentNode)`:

- `JSXAttribute` with `key === 'name'` — attribute names in `<Foo bar={...} />` are JSXIdentifier but name a prop, not our symbol. Skip.
- `JSXNamespacedName` with `key === 'namespace'` — XML-style namespaces are property-key-like. Skip the namespace slot; the `name` slot still counts.

Rationale for skip rules parallels existing `MemberExpression.property` and `Property.key` handling.

### 5.2 Evidence label update

Current label `ast-ident-ref-count` remains accurate after the fix (still counts identifier references via AST; JSX identifiers ARE identifier references). No downstream consumer change required.

Do NOT rename the label to `ast-jsx-aware` — it would imply a capability boundary that should be permanent, not a fixed regression.

### 5.3 Corpus case to pin the fix (candidate for `tests/test-corpus.mjs`)

```js
// CASE-FP41 — JSX identifier reference in same file
// Guards FP-41: v1.10.0 P0 AST counter missed JSX usage of same-file
// exports, over-escalating Tier A compound components to Tier C.
{
  name: 'CASE-FP41',
  files: {
    'src/components/alert-dialog.tsx': `
      import * as React from 'react';
      export const AlertDialog = (props) => (
        <div className="root">
          <AlertDialogTrigger onClick={props.onOpen}>
            {props.children}
          </AlertDialogTrigger>
        </div>
      );
      export const AlertDialogTrigger = (props) => (
        <button onClick={props.onClick}>{props.children}</button>
      );
    `,
    'src/app.tsx': `
      import { AlertDialog } from './components/alert-dialog';
      export const App = () => <AlertDialog onOpen={() => {}}>open</AlertDialog>;
    `,
  },
  expectations: [
    // AlertDialog is live (imported by app.tsx).
    { assertion: 'AlertDialog not in dead-list',
      file: 'dead-classify.json',
      path: 'buckets.C-completely-dead[].symbol',
      absent: 'AlertDialog' },

    // AlertDialogTrigger has no external consumer but is used by
    // <AlertDialogTrigger> inside AlertDialog's render.
    // Correct classification: Tier A (file-internal use, 1 JSX occurrence).
    // Bug classification: Tier C (AST identifier count = 0, mistakenly dead).
    { assertion: 'AlertDialogTrigger is Tier A, not Tier C',
      file: 'dead-classify.json',
      path: 'buckets.A-remove-export[].symbol',
      contains: 'AlertDialogTrigger' },
    { assertion: 'AlertDialogTrigger NOT in Tier C',
      file: 'dead-classify.json',
      path: 'buckets.C-completely-dead[].symbol',
      absent: 'AlertDialogTrigger' },

    // Provenance: the file-internal ref count must report 1 JSX usage.
    { assertion: 'AlertDialogTrigger.fileInternalUses == 1',
      file: 'dead-classify.json',
      path: 'entries[symbol=AlertDialogTrigger].fileInternalUses',
      equals: 1 },
    { assertion: 'valueRefs split reports 1',
      file: 'dead-classify.json',
      path: 'entries[symbol=AlertDialogTrigger].fileInternalRefs.valueRefs',
      equals: 1 },
  ],
},
```

Shape follows CASE-AST / CASE-P1 / CASE-FP40 already in the corpus. Budget gate stays at `FP_BUDGET = 0`; the test fails pre-fix, passes post-fix.

### 5.4 Unit tests to add in `tests/test-classify-facts-ast.mjs`

Twelve minimum cases. Extension must be `.tsx`, not `.ts`, for the parser to accept JSX.

1. Single JSX element `<Foo />` — count 1, valueRefs 1.
2. Nested JSX `<Foo><Bar /></Foo>` counted for `Foo` — count 1.
3. Self-closing with props `<Foo a={1} b="x" />` — count 1.
4. JSXMemberExpression `<Foo.Bar />` counted for `Foo` only (property slot skipped) — count 1.
5. JSXMemberExpression `<Foo.Bar />` counted for `Bar` — count 0 (property slot).
6. JSXAttribute name not counted: `<div foo={x} />` — count 0 for `foo`.
7. JSXAttribute value IS counted: `<div a={foo} />` — count 1 for `foo`.
8. Children text not counted: `<div>foo bar</div>` — count 0 for `foo`.
9. Fragment `<><Foo /></>` counted — count 1.
10. JSX inside conditional `{cond ? <Foo /> : null}` — count 1.
11. Declaration line excluded: `export const Foo = () => <Foo />` with declLine pointing at the export line — inner `<Foo />` still counted? Pick a convention and pin it. (Recommendation: count it — the inner usage IS a separate reference to the binding from the body side.)
12. Spread attributes `<Foo {...props} />` counted — count 1.

Plus the existing 23 non-JSX cases regress-passing unchanged.

## 6. What the fix should NOT do

- Do NOT teach the walker to resolve JSX to `React.createElement` calls. JSX is the source form; treating it as the source form (counting JSXIdentifier directly) is simpler and matches how other static analyzers handle JSX.
- Do NOT add a framework sentinel as the primary fix. That would paper over the counter bug and leave the gap for future FP classes.
- Do NOT retire `ast-ident-ref-count` label. The label description is accurate once JSX is included.
- Do NOT delete the "known scope-aware shadowing gap" note from the file header. That gap is real and separate.

## 7. Coordinated changes required

1. `_lib/classify-facts.mjs::countFileReferencesAst` — JSX identifier handling (§5.1).
2. `_lib/classify-facts.mjs::isSkipPosition` — JSXAttribute.name, JSXNamespacedName.namespace (§5.1).
3. `tests/test-classify-facts-ast.mjs` — add 12 JSX cases (§5.4), switch their fake file path to `/fake/test.tsx`.
4. `tests/test-corpus.mjs` — add CASE-FP41 (§5.3). Keeps `FP_BUDGET = 0`.
5. `scripts/update-test-doc.mjs` — no change (test file already registered).
6. `classify-dead-exports.mjs` comment at line 193 — add short note that `occ === 0` means "zero identifier + JSX identifier + JSX member-top references", not "zero occurrences via any mechanism".
7. `CHANGELOG.md` — v1.10.0 regression + fix entry. Include the duyet A→C migration audit summary (35/70 over-escalation) as the motivating observation.
8. `canonical/invariants.md` — this skill's own rule 4 "4 failure modes": LOC explosion / duplicate blobs / dependency knots / boundary collapse. FP-41 does not add a new failure mode; it is a self-inflicted instance of failure mode #1 ("Claude invents a new helper because it did not know the existing one") applied to the skill's own codebase — the AST counter was written without checking what the regex counter was actually catching in the JSX case. Memory of this event should be stored; no invariant text change needed.

## 8. Suggested memory entries (for cross-session durability)

To add under `project_grounded_audit.md`:

- FP-41 hypothesis → confirmed from source reading; mechanism is `Identifier`-only walker in `_lib/classify-facts.mjs::countFileReferencesAst`.
- v1.9.11 regex counter coincidentally covered JSX; v1.10.0 AST counter does not.
- Impact scale: 35/70 Tier A→C migrations on duyet over-escalated (50% of sampled migrations).
- Fix route: §5.1 walker extension + §5.3 corpus case. ETA: 1 focused session.

## 9. Open question — NOT in scope of this report

Why did the v1.10.0 P2 precision corpus (budget=0 gate) pass without catching FP-41? Because the corpus cases (CASE-AST / CASE-P1 / CASE-FP40) happen to use `.ts` fixtures, not `.tsx`. The gate was mechanically honest — it passed all defined cases — but the case set had a blind spot mirroring the test file's `.ts`-only path. This is a lesson about corpus construction, not about the gate itself. Separate follow-up: survey the existing corpus for language/feature coverage gaps before adding CASE-FP41, so the gap is closed at the coverage-assurance level, not just at the single-case level.

---

**Report end.**
