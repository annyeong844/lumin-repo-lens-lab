# CJS Consumer Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Harden CommonJS consumer extraction so static computed members and non-escaping guards do not create false dead-export confidence.

**Architecture:** Extend the existing `_lib/extract-ts.mjs` CJS member-precision walker rather than introducing a new CJS engine. Keep exact fan-in only for mechanically grounded member reads, keep broad namespace evidence for real escapes, and bump symbol-graph extractor identity because cached per-file facts depend on extraction semantics.

**Tech Stack:** Node.js ESM, OXC AST shape already used by `_lib/extract-ts.mjs`, local fixture tests, symbol graph incremental cache metadata.

---

## File Structure

- Modify `_lib/extract-ts.mjs`
  - Use static computed property names for CJS member reads such as `mod["foo"]`.
  - Treat simple guard reads of tracked CJS namespace identifiers as non-escaping.
- Modify `build-symbol-graph.mjs`
  - Bump `PARSER_IDENTITY` from `symbol-graph-extractors:v1` to `symbol-graph-extractors:v2`.
- Modify `tests/test-extract-cjs-consumer.mjs`
  - Direct extractor red/green coverage for computed literals, guard reads, shadowing, and broad introspection.
- Modify `tests/test-cjs-classification.mjs`
  - Symbol graph coverage that exact CJS hardening affects fan-in and broad cases stay broad.
- Modify `tests/test-symbol-graph-incremental.mjs`
  - Cache invalidation guard for old `symbol-graph-extractors:v1` entries.
- No docs update is required for `tests/README.md` because existing CJS suites already cover this test family.

## Task 1: Add Direct Extractor Red Tests

**Files:**
- Modify: `tests/test-extract-cjs-consumer.mjs`

- [ ] **Step 1: Add helpers near existing `hasUse` helper**

Add this helper after `hasUse(...)`:

```js
function usesFor(source) {
  return extractInfo(source).uses;
}
```

- [ ] **Step 2: Add static computed CJS member tests before the final summary**

Append this block before `console.log(...)`:

```js
{
  const uses = usesFor('const mod = require("./exporter");\nmod["foo"]();\nrequire("./exporter")["bar"];\n');
  assert('CJS10. static computed CJS members are exact consumers',
    hasUse(uses, 'cjs-namespace-member', 'foo') &&
      hasUse(uses, 'cjs-namespace-member', 'bar') &&
      !hasUse(uses, 'cjs-namespace-escape', '*'),
    JSON.stringify(uses));
}
```

- [ ] **Step 3: Add non-escaping guard tests**

Append this block after the `CJS10` block:

```js
{
  const uses = usesFor([
    'const mod = require("./exporter");',
    'if (mod) mod.foo();',
    'mod && mod.bar();',
    'typeof mod !== "undefined" && mod.baz;',
    '',
  ].join('\n'));
  assert('CJS11. simple guard reads do not degrade exact CJS member consumers',
    hasUse(uses, 'cjs-namespace-member', 'foo') &&
      hasUse(uses, 'cjs-namespace-member', 'bar') &&
      hasUse(uses, 'cjs-namespace-member', 'baz') &&
      !hasUse(uses, 'cjs-namespace-escape', '*'),
    JSON.stringify(uses));
}
```

- [ ] **Step 4: Add broad introspection and shadowing guards**

Append this block after the `CJS11` block:

```js
{
  const uses = usesFor('const mod = require("./exporter");\nif ("foo" in mod) mod.foo();\n');
  assert('CJS12. key introspection remains broad CJS evidence',
    hasUse(uses, 'cjs-namespace-escape', '*') &&
      !hasUse(uses, 'cjs-namespace-member', 'foo'),
    JSON.stringify(uses));
}

{
  const uses = usesFor('const mod = require("./exporter");\nfunction f(mod) { mod.foo(); }\n');
  assert('CJS13. shadowed function parameter does not exact-protect outer require',
    !hasUse(uses, 'cjs-namespace-member', 'foo') &&
      !hasUse(uses, 'cjs-namespace-escape', '*'),
    JSON.stringify(uses));
}
```

- [ ] **Step 5: Run direct extractor test and verify RED**

Run:

```bash
node tests/test-extract-cjs-consumer.mjs
```

Expected before implementation:

- `CJS10` fails because static computed members currently degrade to `cjs-namespace-escape`.
- `CJS11` fails because simple `mod` guard reads currently degrade the CJS namespace.
- Existing CJS tests continue to pass.

## Task 2: Implement Static Computed CJS Members And Guard-Neutral Reads

**Files:**
- Modify: `_lib/extract-ts.mjs`

- [ ] **Step 1: Make static computed member names reuse literal string parsing**

Replace `staticMemberPropertyName(...)` with:

```js
function staticMemberPropertyName(node) {
  if (!node?.computed) return memberPropertyName(node);
  return literalStringValue(node.property);
}
```

- [ ] **Step 2: Add a guard helper near `isCallCallee(...)`**

Add this helper after `isCallCallee(parent, key)`:

```js
function isNonEscapingTrackedIdentifierRead(parent, key) {
  if (!parent) return false;
  if (parent.type === 'IfStatement' && key === 'test') return true;
  if (parent.type === 'LogicalExpression' && key === 'left') return true;
  if (parent.type === 'UnaryExpression' && parent.operator === 'typeof' && key === 'argument') return true;
  return false;
}
```

- [ ] **Step 3: Use static member names in direct CJS require member handling**

Replace `handleDirectRequireMemberExpression(...)` with:

```js
function handleDirectRequireMemberExpression(node, state, getNodeLine) {
  if (node.type !== 'MemberExpression') return false;
  const fromSpec = literalRequireSource(node.object);
  if (!fromSpec) return false;
  state.handledCjsRequires.add(node.object);
  const name = staticMemberPropertyName(node);
  if (name) {
    state.cjsDirectUses.push({
      fromSpec,
      name,
      kind: 'cjs-namespace-member',
      typeOnly: false,
      line: getNodeLine(node),
    });
  } else {
    state.cjsFallbackUses.push({
      fromSpec,
      name: '*',
      kind: 'cjs-namespace-escape',
      typeOnly: false,
      line: getNodeLine(node),
      degraded: true,
    });
  }
  return true;
}
```

- [ ] **Step 4: Use static member names for tracked CJS namespace reads**

In `handleTrackedMemberExpression(...)`, replace the CJS branch:

```js
  if (record.kind === 'cjs') {
    const name = !node.computed ? memberPropertyName(node) : null;
    if (name) record.members.push({ name, line: getNodeLine(node) });
    else record.degraded = true;
    return true;
  }
```

with:

```js
  if (record.kind === 'cjs') {
    const name = staticMemberPropertyName(node);
    if (name) record.members.push({ name, line: getNodeLine(node) });
    else record.degraded = true;
    return true;
  }
```

- [ ] **Step 5: Pass parent/key into tracked identifier handling**

In `walkMemberPrecision(...)`, replace:

```js
  if (handleTrackedIdentifier(node, scope)) return;
```

with:

```js
  if (handleTrackedIdentifier(node, scope, parent, key)) return;
```

- [ ] **Step 6: Make simple guards neutral**

Replace `handleTrackedIdentifier(...)` with:

```js
function handleTrackedIdentifier(node, scope, parent, key) {
  if (node.type !== 'Identifier') return false;
  const record = resolveBinding(scope, node.name);
  if (record?.kind === 'namespace' || record?.kind === 'dynamic' || record?.kind === 'cjs') {
    if (isNonEscapingTrackedIdentifierRead(parent, key)) return true;
    record.degraded = true;
  }
  return true;
}
```

- [ ] **Step 7: Run direct extractor test and verify GREEN**

Run:

```bash
node tests/test-extract-cjs-consumer.mjs
```

Expected: all assertions pass, including `CJS10`, `CJS11`, `CJS12`, and `CJS13`.

## Task 3: Add Symbol Graph Integration Coverage

**Files:**
- Modify: `tests/test-cjs-classification.mjs`

- [ ] **Step 1: Add computed and guarded fan-in fixture**

Append this block before the final summary:

```js
{
  const symbols = runSymbolGraph({
    'exporter.js': 'export const foo = 1;\nexport const bar = 2;\nexport const baz = 3;\nexport const unused = 4;\n',
    'consumer.js': [
      'const mod = require("./exporter.js");',
      'if (mod) mod.foo();',
      'mod && mod["bar"];',
      'require("./exporter.js")["baz"];',
      '',
    ].join('\n'),
  });
  assert('G7. guarded and static computed CJS members increase exact fan-in',
    symbols.fanInByIdentity['src/exporter.js::foo'] === 1 &&
      symbols.fanInByIdentity['src/exporter.js::bar'] === 1 &&
      symbols.fanInByIdentity['src/exporter.js::baz'] === 1 &&
      symbols.fanInByIdentity['src/exporter.js::unused'] === 0,
    JSON.stringify(symbols.fanInByIdentity));
}
```

- [ ] **Step 2: Add broad introspection fixture**

Append this block after the `G7` block:

```js
{
  const symbols = runSymbolGraph({
    'exporter.js': 'export const foo = 1;\nexport const bar = 2;\n',
    'consumer.js': 'const mod = require("./exporter.js");\nif ("foo" in mod) mod.foo();\n',
  });
  assert('G8. CJS key introspection stays broad and prevents truly-dead confidence',
    symbols.deadTotal === 2 &&
      symbols.trulyDead === 0 &&
      symbols.fanInByIdentity['src/exporter.js::foo'] === 0,
    JSON.stringify({
      deadTotal: symbols.deadTotal,
      trulyDead: symbols.trulyDead,
      fanIn: symbols.fanInByIdentity,
    }));
}
```

- [ ] **Step 3: Run integration test**

Run:

```bash
node tests/test-cjs-classification.mjs
```

Expected: all assertions pass.

## Task 4: Bump Symbol Graph Extractor Cache Identity

**Files:**
- Modify: `build-symbol-graph.mjs`
- Modify: `tests/test-symbol-graph-incremental.mjs`

- [ ] **Step 1: Bump parser identity**

In `build-symbol-graph.mjs`, change:

```js
const PARSER_IDENTITY = 'symbol-graph-extractors:v1';
```

to:

```js
const PARSER_IDENTITY = 'symbol-graph-extractors:v2';
```

- [ ] **Step 2: Add legacy v1 semantic invalidation test**

In `tests/test-symbol-graph-incremental.mjs`, append this block before the final summary:

```js
{
  const repo = fresh();
  const output = path.join(repo, '.audit');
  try {
    write(repo, 'package.json', JSON.stringify({ name: 'fixture', private: true }));
    write(repo, 'src/exporter.js', 'export const foo = 1;\n');
    write(repo, 'src/consumer.js', [
      'const mod = require("./exporter.js");',
      'if (mod) mod["foo"]();',
      '',
    ].join('\n'));

    run(repo, output);
    rewriteSymbolsCache(repo, (entry) => {
      if (entry.identity?.relPath !== 'src/consumer.js') return;
      entry.producerMeta.parserIdentity = 'symbol-graph-extractors:v1';
    });

    run(repo, output);
    const symbols = readSymbols(output);

    assert('legacy symbol cache with old CJS extractor identity is invalidated',
      symbols.fanInByIdentity?.['src/exporter.js::foo'] === 1 &&
        symbols.meta.incremental?.invalidatedFiles >= 1,
      JSON.stringify({
        fanIn: symbols.fanInByIdentity,
        incremental: symbols.meta.incremental,
      }));
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}
```

- [ ] **Step 3: Run incremental test**

Run:

```bash
node tests/test-symbol-graph-incremental.mjs
```

Expected: all assertions pass, including the new old-parser-identity invalidation guard.

## Task 5: Focused Verification And Commit

**Files:**
- No additional code changes unless checks fail.

- [ ] **Step 1: Run focused CJS checks**

Run:

```bash
node tests/test-extract-cjs-consumer.mjs
node tests/test-cjs-classification.mjs
node tests/test-cjs-integration.mjs
node tests/test-resolved-edges.mjs
node tests/test-symbol-graph-incremental.mjs
```

Expected: all commands exit 0.

- [ ] **Step 2: Run syntax and lint checks**

Run:

```bash
npm run check
npm run lint
git diff --check
```

Expected: all commands exit 0.

- [ ] **Step 3: Commit implementation**

Run:

```bash
git add _lib/extract-ts.mjs build-symbol-graph.mjs tests/test-extract-cjs-consumer.mjs tests/test-cjs-classification.mjs tests/test-symbol-graph-incremental.mjs
git commit -m "Harden CJS consumer precision"
```

Expected: commit succeeds with only the intended files staged.

## Task 6: Calibration Runs

**Files:**
- No code changes unless a regression is discovered.

- [ ] **Step 1: Run local package smoke**

Run:

```bash
npm run check:public-plugin
```

Expected: exits 0 and prepares the public package without maintainer-only leakage.

- [ ] **Step 2: Run quick audits on known CJS-heavy checkouts if present**

Run these only when the paths exist locally:

```bash
node audit-repo.mjs --root C:\Users\endof\Downloads\repo\memento-mcp-main --output C:\Users\endof\Downloads\repo\memento-mcp-main\.audit-cjs-hardening --profile quick
node audit-repo.mjs --root C:\Users\endof\.gemini\antigravity\scratch\suyeon-daemon-followup-p-work-next-20260426 --output C:\Users\endof\.gemini\antigravity\scratch\suyeon-daemon-followup-p-work-next-20260426\.audit-cjs-hardening --profile quick
```

Expected:

- parse errors remain 0 or are explicitly reported,
- CJS opacity appears when unsupported dynamic/broad patterns exist,
- no new `SAFE_FIX` is promoted through relevant CJS broad evidence.

- [ ] **Step 3: Record calibration notes in the PR body**

Do not add a new calibration artifact file in this slice. Summarize the observed before/after CJS behavior in the PR body so the repository stays light.
