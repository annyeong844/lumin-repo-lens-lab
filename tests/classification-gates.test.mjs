// Tests for classification gates mirror + emitted label set — P3-1 Step 0 + P3-2 Step 0.
//
// This test is the FORCING FUNCTION for drift between
// `canonical/classification-gates.md` and the `_lib/canon-draft.mjs`
// code constants + classifier emission.
//
// Execution order within P3-1 (types):
//   Step 0: write THIS file (failing test — import error on missing module).
//   Step 1: implement `_lib/canon-draft.mjs` with the mirror constant +
//           classifier functions the test expects. Step 0 turns GREEN.
//
// Execution order within P3-2 (helpers):
//   P3-2-pre: add canonical §10 Helper classification + update T4/T5 regex
//             to target "The full type label set" (was "The full label set").
//   Step 0:   extend this file with H1..H21 helper assertions. Initially RED
//             because classifyHelperGroup / classifyHelperIdentity /
//             LOW_INFO_HELPER_NAMES don't exist yet.
//   Step 1:   implement helper classifiers in `_lib/canon-draft.mjs`.
//             Helper assertions turn GREEN.
//
// Pinning rules from docs/history/phases/p3/p3-1.md v2 §5.1 + docs/history/phases/p3/p3-2.md v2 §5.1:
//   - LOW_INFO_NAMES mirror byte-equal to canonical §3 (names + order).
//   - LOW_INFO_HELPER_NAMES mirror byte-equal to canonical §10.4 (names + order).
//   - §9 type label enumeration is the canonical 9-label type set exactly.
//   - §9 helper label enumeration is the canonical 9-label helper set exactly.
//   - emitted-type-label Set ⊆ canonical type 9-label set.
//   - emitted-helper-label Set ⊆ canonical helper 9-label set.
//   - typeName vs exportedName identity pin (types).
//   - helperName / calleeName identity pin (helpers).
//   - Contamination-unavailability pin: severely-any-contaminated-helper +
//     ANY_COLLISION_HELPER structurally unreachable when contamination arg
//     is undefined / contaminationByIdentity is empty (p3-2 PF-5 / §4.5).
//   - Fan-in semantics pin: topCallees.count never assigned to fanIn
//     in _lib/canon-draft.mjs (p3-2 PF-4).

import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { expect, it } from "vitest";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, "..");

function assert(label, ok, detail = "") {
  it(label, () => {
    expect(ok, detail).toBeTruthy();
  });
}

// Post-P3-cleanup (2026-04-21): classifier code lives in 5 leaf modules
// (`canon-draft-utils`, `-types`, `-helpers`, `-topology`, `-naming`). The
// former 2064-LOC `canon-draft.mjs` is now a 69-line re-export facade.
// Structural source-grep pins below validate the IMPLEMENTATION code, so
// they must read every canon-draft*.mjs file together — not just the
// facade (which has no implementation).
function readAllCanonDraftSources() {
  const files = [
    "canon-draft-utils.mjs",
    "canon-draft-types.mjs",
    "canon-draft-helpers.mjs",
    "canon-draft-topology.mjs",
    "canon-draft-naming.mjs",
  ];
  return files
    .map(
      (name) =>
        `// ── ${name} ──\n` +
        readFileSync(path.join(DIR, "_lib", name), "utf8"),
    )
    .join("\n");
}

// ── Canonical authoritative values (hardcoded) ──────────────
//
// Hardcoded here as the P3-1 pin. If canonical file is edited, this
// test fails — forcing an update to `_lib/canon-draft.mjs` mirror AND
// the expected lists below, so drift is visible at test time rather
// than at downstream-consumer time.

const EXPECTED_LOW_INFO_NAMES = [
  "Props",
  "Options",
  "Config",
  "State",
  "Result",
  "Meta",
  "Item",
  "Data",
  "Context",
  "Args",
  "Params",
  "Response",
  "Request",
  "Handler",
  "Input",
  "Output",
];

const EXPECTED_LOW_INFO_HELPER_NAMES = [
  "get",
  "set",
  "parse",
  "format",
  "fetch",
  "load",
  "save",
  "build",
  "make",
  "create",
  "update",
  "handle",
  "run",
  "process",
  "convert",
];

// Canonical §9 type-label enumeration, unordered.
const EXPECTED_LABELS = [
  "zero-internal-fan-in",
  "low-signal-type-name",
  "DUPLICATE_STRONG",
  "DUPLICATE_REVIEW",
  "LOCAL_COMMON_NAME",
  "single-owner-strong",
  "single-owner-weak",
  "severely-any-contaminated",
  "ANY_COLLISION",
];

// Canonical §9 helper-label enumeration (added in v1.1 for P3-2).
const EXPECTED_HELPER_LABELS = [
  "HELPER_DUPLICATE_STRONG",
  "HELPER_DUPLICATE_REVIEW",
  "HELPER_LOCAL_COMMON",
  "ANY_COLLISION_HELPER",
  "severely-any-contaminated-helper",
  "central-helper",
  "shared-helper",
  "zero-internal-fan-in-helper",
  "low-signal-helper-name",
];

// Canonical §11 topology-label enumeration (added in v1.2 for P3-3).
const EXPECTED_TOPOLOGY_LABELS = [
  "cyclic-submodule",
  "isolated-submodule",
  "shared-submodule",
  "leaf-submodule",
  "scoped-submodule",
  "forbidden-cycle",
  "oversize",
  "extreme-oversize",
];

const EXPECTED_TOPOLOGY_UNCERTAIN_REASONS = [
  "topology-artifact-incomplete",
  "topology-artifact-stale",
  "submodule-boundary-mismatch",
];

// Canonical §12 naming-label enumeration (added in v1.3 for P3-4).
// 10 labels: 7 cohort (§12.1) + 3 per-item (§12.2).
const EXPECTED_NAMING_LABELS = [
  "camelCase-dominant",
  "PascalCase-dominant",
  "kebab-case-dominant",
  "snake_case-dominant",
  "UPPER_SNAKE-dominant",
  "mixed-convention",
  "insufficient-evidence",
  "convention-match",
  "convention-outlier",
  "low-info-excluded",
];

const EXPECTED_NAMING_CONVENTIONS = [
  "camelCase",
  "PascalCase",
  "kebab-case",
  "snake_case",
  "UPPER_SNAKE",
  "mixed",
];

const EXPECTED_NAMING_UNCERTAIN_REASONS = [
  "parse-error",
  "cohort-insufficient-evidence",
];

const EXPECTED_CANON_DRAFT_SOURCES = [
  "type-ownership",
  "helper-registry",
  "topology",
  "naming",
];

const gatesPath = path.join(DIR, "canonical", "classification-gates.md");
const gatesText = readFileSync(gatesPath, "utf8");

// ── Canonical parsing helpers ───────────────────────────────

function extractFencedNamesFromMarkdown(text, markerHeading) {
  // Find the fenced code block immediately under the given heading.
  // §3 (LOW_INFO_NAMES) and §10.4 (LOW_INFO_HELPER_NAMES) share the same
  // format: comma-separated names in a triple-backtick block.
  const idx = text.indexOf(markerHeading);
  if (idx < 0) return null;
  const rest = text.slice(idx);
  const fenceStart = rest.indexOf("```");
  if (fenceStart < 0) return null;
  const fenceContent = rest.slice(fenceStart + 3);
  const fenceEnd = fenceContent.indexOf("```");
  if (fenceEnd < 0) return null;
  const body = fenceContent.slice(0, fenceEnd);
  return body
    .split(",")
    .map((s) => s.trim())
    .filter(Boolean);
}

function extractLowInfoNamesFromMarkdown(text) {
  return extractFencedNamesFromMarkdown(text, "## 3. LOW_INFO_NAMES");
}

function extractLowInfoHelperNamesFromMarkdown(text) {
  return extractFencedNamesFromMarkdown(text, "### 10.4 LOW_INFO_HELPER_NAMES");
}

function extractLabelsFromInvariantLine(text, lineHead) {
  // §9 invariants carry two label enumerations — type and helper.
  // Each line reads: "The full {type|helper} label set ...: `A` / `B` / .... Adding a new one requires revising this file."
  const re = new RegExp(lineHead + "[^:]*:\\s*(.+?)\\. Adding", "s");
  const m = text.match(re);
  if (!m) return null;
  const body = m[1];
  const matches = [...body.matchAll(/`([a-zA-Z_-]+)`/g)];
  return matches.map((x) => x[1]);
}

function extractLabelsFromMarkdown(text) {
  return extractLabelsFromInvariantLine(text, "The full type label set");
}

function extractHelperLabelsFromMarkdown(text) {
  return extractLabelsFromInvariantLine(text, "The full helper label set");
}

function extractTopologyLabelsFromMarkdown(text) {
  return extractLabelsFromInvariantLine(text, "The full topology label set");
}

function extractNamingLabelsFromMarkdown(text) {
  return extractLabelsFromInvariantLine(text, "The full naming label set");
}

// ── T1–T4. Canonical file sanity ────────────────────────────

const parsedLowInfo = extractLowInfoNamesFromMarkdown(gatesText);
assert(
  "T1. canonical §3 LOW_INFO_NAMES block parsed",
  Array.isArray(parsedLowInfo) && parsedLowInfo.length > 0,
);

assert(
  "T2. canonical LOW_INFO_NAMES has exactly 16 entries",
  parsedLowInfo && parsedLowInfo.length === 16,
  `got ${parsedLowInfo?.length}`,
);

assert(
  "T3. canonical LOW_INFO_NAMES matches expected list AND order",
  parsedLowInfo &&
    JSON.stringify(parsedLowInfo) === JSON.stringify(EXPECTED_LOW_INFO_NAMES),
  `canonical=${JSON.stringify(parsedLowInfo)}\n        expected=${JSON.stringify(EXPECTED_LOW_INFO_NAMES)}`,
);

const parsedLabels = extractLabelsFromMarkdown(gatesText);
assert(
  "T4. canonical §9 label enumeration parses to exactly 9 labels",
  parsedLabels && parsedLabels.length === 9,
  `got ${parsedLabels?.length}: ${JSON.stringify(parsedLabels)}`,
);

assert(
  "T5. canonical §9 label set equals expected 9-label set",
  parsedLabels &&
    new Set(parsedLabels).size === 9 &&
    EXPECTED_LABELS.every((l) => parsedLabels.includes(l)),
  `canonical=${JSON.stringify(parsedLabels)}`,
);

// ── T6–T10. Mirror + classifier emission (via _lib/canon-draft.mjs) ─
//
// Step 0 RED: importing `_lib/canon-draft.mjs` before Step 1 fails the
// whole test on import error. Step 1 adds the module; Step 0 turns
// GREEN at that point. Do not guard the import with try/catch — the
// failure IS the signal.

const canonDraft = await import("../_lib/canon-draft.mjs");

assert(
  "T6. _lib/canon-draft.mjs exports LOW_INFO_NAMES",
  Array.isArray(canonDraft.LOW_INFO_NAMES),
);

assert(
  "T7. LOW_INFO_NAMES mirror is frozen (Object.freeze)",
  Object.isFrozen(canonDraft.LOW_INFO_NAMES),
);

assert(
  "T8. LOW_INFO_NAMES mirror is byte-equal to canonical §3 (names + order)",
  JSON.stringify([...canonDraft.LOW_INFO_NAMES]) ===
    JSON.stringify(EXPECTED_LOW_INFO_NAMES),
  `mirror=${JSON.stringify(canonDraft.LOW_INFO_NAMES)}`,
);

assert(
  "T9. classifyTypeNameGroup exported",
  typeof canonDraft.classifyTypeNameGroup === "function",
);

assert(
  "T10. classifySingleIdentity exported",
  typeof canonDraft.classifySingleIdentity === "function",
);

// ── T11. Emitted label values ⊆ canonical 9-label set ──
//
// Call both classifiers with representative inputs covering every branch,
// collect emitted label values, assert subset relationship.

{
  const expected = new Set(EXPECTED_LABELS);
  const emitted = new Set();

  // Group classifier — exercise every Rule branch.
  const id1 = "a.ts::Foo";
  const id2 = "b.ts::Foo";

  // Rule 0: all contaminated → ANY_COLLISION
  emitted.add(
    canonDraft.classifyTypeNameGroup({
      name: "Foo",
      identities: [id1, id2],
      fanInByIdentity: { [id1]: 5, [id2]: 5 },
      contaminationByIdentity: {
        [id1]: { label: "any-contaminated" },
        [id2]: { label: "severely-any-contaminated" },
      },
    }).label,
  );

  // Rule 1: DUPLICATE_STRONG
  emitted.add(
    canonDraft.classifyTypeNameGroup({
      name: "Bar",
      identities: [id1, id2],
      fanInByIdentity: { [id1]: 5, [id2]: 0 },
      contaminationByIdentity: {},
    }).label,
  );

  // Rule 2: LOCAL_COMMON_NAME
  emitted.add(
    canonDraft.classifyTypeNameGroup({
      name: "Props",
      identities: [id1, id2],
      fanInByIdentity: { [id1]: 1, [id2]: 2 },
      contaminationByIdentity: {},
    }).label,
  );

  // Rule 3: DUPLICATE_REVIEW
  emitted.add(
    canonDraft.classifyTypeNameGroup({
      name: "Xyz",
      identities: [id1, id2],
      fanInByIdentity: { [id1]: 1, [id2]: 1 },
      contaminationByIdentity: {},
    }).label,
  );

  // Single-identity classifier — exercise every Rule branch.
  emitted.add(
    canonDraft.classifySingleIdentity({
      identity: id1,
      fanIn: 5,
      kind: "TSTypeAliasDeclaration",
      contamination: { label: "severely-any-contaminated" },
    }).label,
  );
  emitted.add(
    canonDraft.classifySingleIdentity({
      identity: id1,
      fanIn: 2,
      kind: "TSTypeAliasDeclaration",
      contamination: null,
    }).label,
  ); // name-len check needs single-char name:
  emitted.add(
    canonDraft.classifySingleIdentity({
      identity: "a.ts::X",
      fanIn: 2,
      kind: "TSTypeAliasDeclaration",
      contamination: null,
    }).label,
  );
  emitted.add(
    canonDraft.classifySingleIdentity({
      identity: id1,
      fanIn: 5,
      kind: "TSInterfaceDeclaration",
      contamination: null,
    }).label,
  );
  emitted.add(
    canonDraft.classifySingleIdentity({
      identity: id1,
      fanIn: 2,
      kind: "TSInterfaceDeclaration",
      contamination: null,
    }).label,
  );
  emitted.add(
    canonDraft.classifySingleIdentity({
      identity: id1,
      fanIn: 0,
      kind: "TSInterfaceDeclaration",
      contamination: null,
    }).label,
  );

  const strayLabels = [...emitted].filter((l) => !expected.has(l));
  assert(
    "T11. emitted classifier labels ⊆ canonical 9-label set",
    strayLabels.length === 0,
    `stray labels NOT in canonical §9: ${JSON.stringify(strayLabels)}`,
  );
}

// ── T12. typeName vs exportedName — canonical identity pin ──
//
// The identity field on a type-owner fact is `exportedName` (per
// `canonical/fact-model.md` §3.1). `typeName` is display alias; on
// type-owner facts the two are equal, but a hypothetical future
// divergence MUST NOT leak into identity / Map-key / Set-member logic.
// This pin is structural — source-grep on `_lib/canon-draft.mjs`.

{
  const src = readAllCanonDraftSources();

  // Strip comments so prose mentions of typeName don't false-positive.
  const stripped = src
    .replace(/\/\*[\s\S]*?\*\//g, "")
    .replace(/\/\/[^\n]*/g, "");

  // Flag typeName used as a Map-key / Set-member / object-key shape.
  // Acceptable: `{ typeName: x.typeName }` in a display-row builder.
  // Not acceptable: `new Map(...typeName)`, `typeNameIndex.set(typeName`,
  // `typeUsesByIdentity.set(typeName` — identity-shaped keying.
  const suspect = [
    /\.set\(\s*[a-zA-Z_][a-zA-Z0-9_]*\.typeName\b/,
    /typeNameIndex\s*=\s*new\s+Map/, // identity-name Map by typeName
    /new\s+Set\(\s*[a-zA-Z_][a-zA-Z0-9_]*\.typeName/,
  ];

  const flagged = suspect.filter((re) => re.test(stripped));
  assert(
    "T12. _lib/canon-draft.mjs does NOT key identity/Map/Set on typeName",
    flagged.length === 0,
    `flagged patterns: ${flagged.map((r) => r.toString()).join(", ")}`,
  );
}

// ── T13. Classification-gates document anchor ──

assert(
  "T13. canonical/classification-gates.md §9 mentions all 9 labels with backticks",
  EXPECTED_LABELS.every((l) => new RegExp("`" + l + "`").test(gatesText)),
);

// ════════════════════════════════════════════════════════════════
// P3-2 helper assertions — H1..H21
// ════════════════════════════════════════════════════════════════

// ── H1–H2. Canonical §10.4 LOW_INFO_HELPER_NAMES block ──

const parsedHelperLowInfo = extractLowInfoHelperNamesFromMarkdown(gatesText);
assert(
  "H1. canonical §10.4 LOW_INFO_HELPER_NAMES block parsed",
  Array.isArray(parsedHelperLowInfo) && parsedHelperLowInfo.length > 0,
);

assert(
  "H2. canonical LOW_INFO_HELPER_NAMES matches expected 15-name list AND order",
  parsedHelperLowInfo &&
    JSON.stringify(parsedHelperLowInfo) ===
      JSON.stringify(EXPECTED_LOW_INFO_HELPER_NAMES),
  `canonical=${JSON.stringify(parsedHelperLowInfo)}\n        expected=${JSON.stringify(EXPECTED_LOW_INFO_HELPER_NAMES)}`,
);

// ── H3–H4. Canonical §9 helper label enumeration ──

const parsedHelperLabels = extractHelperLabelsFromMarkdown(gatesText);
assert(
  "H3. canonical §9 helper-label enumeration parses to exactly 9 labels",
  parsedHelperLabels && parsedHelperLabels.length === 9,
  `got ${parsedHelperLabels?.length}: ${JSON.stringify(parsedHelperLabels)}`,
);

assert(
  "H4. canonical §9 helper-label set equals expected 9-label set",
  parsedHelperLabels &&
    new Set(parsedHelperLabels).size === 9 &&
    EXPECTED_HELPER_LABELS.every((l) => parsedHelperLabels.includes(l)),
  `canonical=${JSON.stringify(parsedHelperLabels)}`,
);

// ── H5–H9. Mirror + classifier exports (via _lib/canon-draft.mjs) ──

assert(
  "H5. _lib/canon-draft.mjs exports LOW_INFO_HELPER_NAMES",
  Array.isArray(canonDraft.LOW_INFO_HELPER_NAMES),
);

assert(
  "H6. LOW_INFO_HELPER_NAMES mirror is frozen (Object.freeze)",
  Object.isFrozen(canonDraft.LOW_INFO_HELPER_NAMES),
);

assert(
  "H7. LOW_INFO_HELPER_NAMES mirror is byte-equal to canonical §10.4 (names + order)",
  canonDraft.LOW_INFO_HELPER_NAMES &&
    JSON.stringify([...canonDraft.LOW_INFO_HELPER_NAMES]) ===
      JSON.stringify(EXPECTED_LOW_INFO_HELPER_NAMES),
  `mirror=${JSON.stringify(canonDraft.LOW_INFO_HELPER_NAMES)}`,
);

assert(
  "H8. classifyHelperGroup exported",
  typeof canonDraft.classifyHelperGroup === "function",
);

assert(
  "H9. classifyHelperIdentity exported",
  typeof canonDraft.classifyHelperIdentity === "function",
);

// ── H10. Emitted helper-classifier labels ⊆ canonical 9-label helper set ──

{
  const expected = new Set(EXPECTED_HELPER_LABELS);
  const emitted = new Set();

  const id1 = "a.ts::parseJson";
  const id2 = "b.ts::parseJson";

  // Group Rule 0: ANY_COLLISION_HELPER (all contaminated)
  emitted.add(
    canonDraft.classifyHelperGroup({
      name: "parseJson",
      identities: [id1, id2],
      fanInByIdentity: { [id1]: 5, [id2]: 5 },
      contaminationByIdentity: {
        [id1]: { label: "any-contaminated" },
        [id2]: { label: "severely-any-contaminated" },
      },
    }).label,
  );

  // Group Rule 1: HELPER_DUPLICATE_STRONG
  emitted.add(
    canonDraft.classifyHelperGroup({
      name: "renderThing",
      identities: [id1, id2],
      fanInByIdentity: { [id1]: 5, [id2]: 0 },
      contaminationByIdentity: {},
    }).label,
  );

  // Group Rule 2: HELPER_LOCAL_COMMON (low-info helper name, low fanIn)
  emitted.add(
    canonDraft.classifyHelperGroup({
      name: "get",
      identities: [id1, id2],
      fanInByIdentity: { [id1]: 1, [id2]: 2 },
      contaminationByIdentity: {},
    }).label,
  );

  // Group Rule 3: HELPER_DUPLICATE_REVIEW
  emitted.add(
    canonDraft.classifyHelperGroup({
      name: "renderThing",
      identities: [id1, id2],
      fanInByIdentity: { [id1]: 1, [id2]: 1 },
      contaminationByIdentity: {},
    }).label,
  );

  // Single Rule 0: severely-any-contaminated-helper
  emitted.add(
    canonDraft.classifyHelperIdentity({
      identity: id1,
      fanIn: 5,
      kind: "FunctionDeclaration",
      contamination: { label: "severely-any-contaminated" },
    }).label,
  );

  // Single Rule 1: low-signal-helper-name (name ∈ low-info helper names, fanIn < 3)
  emitted.add(
    canonDraft.classifyHelperIdentity({
      identity: "a.ts::get",
      fanIn: 2,
      kind: "FunctionDeclaration",
      contamination: null,
      exportedName: "get",
    }).label,
  );

  // Single Rule 2: central-helper (fanIn ≥ 3)
  emitted.add(
    canonDraft.classifyHelperIdentity({
      identity: id1,
      fanIn: 5,
      kind: "FunctionDeclaration",
      contamination: null,
      exportedName: "parseJson",
    }).label,
  );

  // Single Rule 3: shared-helper (fanIn 1 or 2)
  emitted.add(
    canonDraft.classifyHelperIdentity({
      identity: id1,
      fanIn: 2,
      kind: "FunctionDeclaration",
      contamination: null,
      exportedName: "parseJson",
    }).label,
  );

  // Single Rule 4: zero-internal-fan-in-helper
  emitted.add(
    canonDraft.classifyHelperIdentity({
      identity: id1,
      fanIn: 0,
      kind: "FunctionDeclaration",
      contamination: null,
      exportedName: "parseJson",
    }).label,
  );

  const strayLabels = [...emitted].filter((l) => !expected.has(l));
  assert(
    "H10. emitted helper-classifier labels ⊆ canonical 9-label helper set",
    strayLabels.length === 0,
    `stray labels NOT in canonical §10.3: ${JSON.stringify(strayLabels)}`,
  );
}

// ── H11–H13. ANY_COLLISION_HELPER Rule 0 scope (universal quantifier) ──

{
  const id1 = "a.ts::fetch";
  const id2 = "b.ts::fetch";

  // H11. has-any only → NOT ANY_COLLISION_HELPER
  const r1 = canonDraft.classifyHelperGroup({
    name: "fetch",
    identities: [id1, id2],
    fanInByIdentity: { [id1]: 5, [id2]: 5 },
    contaminationByIdentity: {
      [id1]: { label: "has-any" },
      [id2]: { label: "has-any" },
    },
  });
  assert(
    "H11. group of has-any-only helpers does NOT trigger ANY_COLLISION_HELPER",
    r1.label !== "ANY_COLLISION_HELPER",
    `got=${r1.label}`,
  );

  // H12. unknown-surface only → NOT ANY_COLLISION_HELPER
  const r2 = canonDraft.classifyHelperGroup({
    name: "fetch",
    identities: [id1, id2],
    fanInByIdentity: { [id1]: 5, [id2]: 5 },
    contaminationByIdentity: {
      [id1]: { label: "unknown-surface" },
      [id2]: { label: "unknown-surface" },
    },
  });
  assert(
    "H12. group of unknown-surface-only helpers does NOT trigger ANY_COLLISION_HELPER",
    r2.label !== "ANY_COLLISION_HELPER",
    `got=${r2.label}`,
  );

  // H13. mixed (one severe + one clean) → NOT ANY_COLLISION_HELPER (universal, not existential)
  const r3 = canonDraft.classifyHelperGroup({
    name: "fetch",
    identities: [id1, id2],
    fanInByIdentity: { [id1]: 5, [id2]: 5 },
    contaminationByIdentity: {
      [id1]: { label: "severely-any-contaminated" },
      // id2: absent (clean)
    },
  });
  assert(
    "H13. mixed contaminated+clean helper group does NOT trigger ANY_COLLISION_HELPER",
    r3.label !== "ANY_COLLISION_HELPER",
    `got=${r3.label}`,
  );
}

// ── H14. Precedence: HELPER_DUPLICATE_STRONG wins over HELPER_LOCAL_COMMON at fanIn ≥ 3 ──

{
  const id1 = "a.ts::parse";
  const id2 = "b.ts::parse";
  const r = canonDraft.classifyHelperGroup({
    name: "parse", // ∈ LOW_INFO_HELPER_NAMES
    identities: [id1, id2],
    fanInByIdentity: { [id1]: 4, [id2]: 3 }, // max-fanIn ≥ 3
    contaminationByIdentity: {},
  });
  assert(
    "H14. group `parse` (low-info name) + fanIn ≥ 3 → HELPER_DUPLICATE_STRONG (Rule 1 wins over Rule 2)",
    r.label === "HELPER_DUPLICATE_STRONG",
    `got=${r.label}`,
  );
}

// ── H15–H16. Single-identity Rule 1 vs Rule 2 precedence for low-info helper names ──

{
  // H15. `get` with fanIn 3 → central-helper (Rule 2 wins over Rule 1)
  const r1 = canonDraft.classifyHelperIdentity({
    identity: "a.ts::get",
    fanIn: 3,
    kind: "FunctionDeclaration",
    contamination: null,
    exportedName: "get",
  });
  assert(
    "H15. single `get` + fanIn 3 → central-helper (Rule 2 fires over Rule 1 at threshold)",
    r1.label === "central-helper",
    `got=${r1.label}`,
  );

  // H16. `get` with fanIn 2 → low-signal-helper-name (Rule 1 fires)
  const r2 = canonDraft.classifyHelperIdentity({
    identity: "a.ts::get",
    fanIn: 2,
    kind: "FunctionDeclaration",
    contamination: null,
    exportedName: "get",
  });
  assert(
    "H16. single `get` + fanIn 2 → low-signal-helper-name (Rule 1 fires below threshold)",
    r2.label === "low-signal-helper-name",
    `got=${r2.label}`,
  );
}

// ── H17. Contamination-unavailability pin (single-identity) ──
//
// Fresh-AST-only mode: contamination argument is `undefined` or null. 50+
// varied invocations across fan-in / name tiers MUST NOT produce
// `severely-any-contaminated-helper` — Rule 0 is structurally unreachable.

{
  const names = ["parseJson", "get", "fetchData", "format", "doThing", "a"];
  const fanIns = [0, 1, 2, 3, 5, 10, 15, 100];
  const kinds = [
    "FunctionDeclaration",
    "ArrowFunctionExpression",
    "FunctionExpression",
  ];
  // name × fanIn × kind = 6 × 8 × 3 = 144 invocations
  let severeSeen = 0;
  for (const name of names) {
    for (const fi of fanIns) {
      for (const k of kinds) {
        const r = canonDraft.classifyHelperIdentity({
          identity: `a.ts::${name}`,
          fanIn: fi,
          kind: k,
          contamination: undefined,
          exportedName: name,
        });
        if (r.label === "severely-any-contaminated-helper") severeSeen++;
      }
    }
  }
  assert(
    `H17. severely-any-contaminated-helper never appears across 144 classifier runs when contamination=undefined`,
    severeSeen === 0,
    `saw ${severeSeen} severely-any-contaminated-helper emissions`,
  );
}

// ── H18. Contamination-unavailability pin (group) ──
//
// Group classifier with empty contaminationByIdentity across 30+ varied groups
// MUST NOT produce ANY_COLLISION_HELPER.

{
  const names = ["parseJson", "get", "fetchData", "format", "doThing", "a"];
  const fanInPairs = [
    [5, 5],
    [3, 0],
    [1, 2],
    [0, 0],
    [10, 10],
  ];
  let collisionSeen = 0;
  for (const name of names) {
    for (const [f1, f2] of fanInPairs) {
      const r = canonDraft.classifyHelperGroup({
        name,
        identities: [`a.ts::${name}`, `b.ts::${name}`],
        fanInByIdentity: { [`a.ts::${name}`]: f1, [`b.ts::${name}`]: f2 },
        contaminationByIdentity: {},
      });
      if (r.label === "ANY_COLLISION_HELPER") collisionSeen++;
    }
  }
  assert(
    `H18. ANY_COLLISION_HELPER never appears across 30 group-classifier runs when contaminationByIdentity is empty`,
    collisionSeen === 0,
    `saw ${collisionSeen} ANY_COLLISION_HELPER emissions`,
  );
}

// ── H19. Source-grep — no helperName / calleeName Map/Set keying ──

{
  const src = readAllCanonDraftSources();
  const stripped = src
    .replace(/\/\*[\s\S]*?\*\//g, "")
    .replace(/\/\/[^\n]*/g, "");

  // Identity-shaped keying on raw helper/callee name (not `exportedName`).
  const suspect = [
    /\.set\(\s*[a-zA-Z_][a-zA-Z0-9_]*\.helperName\b/,
    /\.set\(\s*[a-zA-Z_][a-zA-Z0-9_]*\.calleeName\b/,
    /helperNameIndex\s*=\s*new\s+Map/,
    /calleeNameIndex\s*=\s*new\s+Map/,
    /new\s+Set\(\s*[a-zA-Z_][a-zA-Z0-9_]*\.helperName/,
    /new\s+Set\(\s*[a-zA-Z_][a-zA-Z0-9_]*\.calleeName/,
  ];
  const flagged = suspect.filter((re) => re.test(stripped));
  assert(
    "H19. _lib/canon-draft.mjs does NOT key identity/Map/Set on helperName or calleeName",
    flagged.length === 0,
    `flagged patterns: ${flagged.map((r) => r.toString()).join(", ")}`,
  );
}

// ── H20. Fan-in semantics pin — topCallees.count never assigned to fanIn ──
//
// Per docs/history/phases/p3/p3-2.md v2 PF-4: fan-in is consumer-file-count (via fresh AST),
// NOT aggregated call-site count from `call-graph.json.topCallees.count`.
// Pin the negative: any assignment like `fanIn = ...topCallees.count` is a
// defect. Permit reads of `topCallees` in cross-check / diagnostic context.

{
  const src = readAllCanonDraftSources();
  const stripped = src
    .replace(/\/\*[\s\S]*?\*\//g, "")
    .replace(/\/\/[^\n]*/g, "");

  // Strict: `fanIn = <anything>.topCallees[...]...count` or
  //          `fanIn = topCallees.count` on one line.
  const suspect = [
    /\bfanIn\s*=\s*[^;\n]*topCallees[^;\n]*\.count\b/,
    /\bfanIn\s*:\s*[^,;\n]*topCallees[^,;\n]*\.count\b/,
  ];
  const flagged = suspect.filter((re) => re.test(stripped));
  assert(
    "H20. _lib/canon-draft.mjs does NOT assign topCallees.count to fanIn (PF-4)",
    flagged.length === 0,
    `flagged patterns: ${flagged.map((r) => r.toString()).join(", ")}`,
  );
}

// ── H21. `[확인 불가]` reason enum — exactly 4 allowed values ──
//
// Per docs/history/phases/p3/p3-2.md v2 §4.1 the 4 reasons are fixed. If _lib/canon-draft.mjs
// exports a frozen enum of allowed reasons (design choice during Step 1),
// assert its values; otherwise surface as absence.

{
  const EXPECTED_REASONS = [
    "ambiguous-star-reexport",
    "resolveIdentity-depth-exceeded",
    "unresolved-specifier",
    "helper-owner-facts-unavailable",
  ];
  const exported = canonDraft.UNCERTAIN_REASONS;
  assert(
    "H21. _lib/canon-draft.mjs exports UNCERTAIN_REASONS enum with exactly 4 values",
    Array.isArray(exported) &&
      Object.isFrozen(exported) &&
      JSON.stringify([...exported].sort()) ===
        JSON.stringify([...EXPECTED_REASONS].sort()),
    `got=${JSON.stringify(exported)}`,
  );
}

// ════════════════════════════════════════════════════════════════
// P3-3 topology assertions — TP1..TP18
// ════════════════════════════════════════════════════════════════

// ── TP1–TP4. Canonical §11 topology label enumeration ──

const parsedTopologyLabels = extractTopologyLabelsFromMarkdown(gatesText);
assert(
  "TP1. canonical §9 topology-label enumeration parses to exactly 8 labels",
  parsedTopologyLabels && parsedTopologyLabels.length === 8,
  `got ${parsedTopologyLabels?.length}: ${JSON.stringify(parsedTopologyLabels)}`,
);

assert(
  "TP2. canonical §9 topology-label set equals expected 8-label set",
  parsedTopologyLabels &&
    new Set(parsedTopologyLabels).size === 8 &&
    EXPECTED_TOPOLOGY_LABELS.every((l) => parsedTopologyLabels.includes(l)),
  `canonical=${JSON.stringify(parsedTopologyLabels)}`,
);

assert(
  "TP3. canonical §11.4 label set block has all 8 expected labels",
  EXPECTED_TOPOLOGY_LABELS.every((l) =>
    new RegExp(`^${l}$`, "m").test(gatesText),
  ),
);

assert(
  "TP4. canonical §11 Topology classification section exists",
  /## 11\. Topology classification/.test(gatesText),
);

// ── TP5–TP9. Code-side mirror + classifier exports ──

assert(
  "TP5. _lib/canon-draft.mjs exports TOPOLOGY_LABELS",
  Array.isArray(canonDraft.TOPOLOGY_LABELS),
);

assert(
  "TP6. TOPOLOGY_LABELS mirror is frozen",
  Object.isFrozen(canonDraft.TOPOLOGY_LABELS),
);

assert(
  "TP7. TOPOLOGY_LABELS byte-equal to canonical §11.4 (names + order)",
  canonDraft.TOPOLOGY_LABELS &&
    JSON.stringify([...canonDraft.TOPOLOGY_LABELS]) ===
      JSON.stringify(EXPECTED_TOPOLOGY_LABELS),
  `mirror=${JSON.stringify(canonDraft.TOPOLOGY_LABELS)}`,
);

assert(
  "TP8. classifyTopologySubmodule exported",
  typeof canonDraft.classifyTopologySubmodule === "function",
);

assert(
  "TP9. classifyTopologyScc + classifyTopologyFile exported",
  typeof canonDraft.classifyTopologyScc === "function" &&
    typeof canonDraft.classifyTopologyFile === "function",
);

// ── TP10. Emitted topology-classifier labels ⊆ canonical §11 set ──

{
  const expected = new Set(EXPECTED_TOPOLOGY_LABELS);
  const emitted = new Set();

  // Submodule classifier — exercise every Rule branch.
  // Rule 0: cyclic-submodule
  emitted.add(
    canonDraft.classifyTopologySubmodule({
      name: "a",
      inDegree: 10,
      outDegree: 5,
      sccMember: true,
      crossEdgeSource: "full-list",
    }).label,
  );
  // Rule 1: isolated-submodule (only fires in full-list mode)
  emitted.add(
    canonDraft.classifyTopologySubmodule({
      name: "b",
      inDegree: 0,
      outDegree: 0,
      sccMember: false,
      crossEdgeSource: "full-list",
    }).label,
  );
  // Rule 2: shared-submodule
  emitted.add(
    canonDraft.classifyTopologySubmodule({
      name: "c",
      inDegree: 7,
      outDegree: 2,
      sccMember: false,
      crossEdgeSource: "full-list",
    }).label,
  );
  // Rule 3: leaf-submodule
  emitted.add(
    canonDraft.classifyTopologySubmodule({
      name: "d",
      inDegree: 1,
      outDegree: 5,
      sccMember: false,
      crossEdgeSource: "full-list",
    }).label,
  );
  // Rule 4: scoped-submodule
  emitted.add(
    canonDraft.classifyTopologySubmodule({
      name: "e",
      inDegree: 2,
      outDegree: 2,
      sccMember: false,
      crossEdgeSource: "full-list",
    }).label,
  );

  // SCC + file classifiers
  emitted.add(
    canonDraft.classifyTopologyScc({ sccIndex: 0, members: ["a.ts", "b.ts"] })
      .label,
  );
  const fileA = canonDraft.classifyTopologyFile({ file: "big.ts", loc: 500 });
  const fileB = canonDraft.classifyTopologyFile({ file: "huge.ts", loc: 1200 });
  emitted.add(fileA.label);
  emitted.add(fileB.label);

  const strayLabels = [...emitted].filter((l) => !expected.has(l));
  assert(
    "TP10. emitted topology-classifier labels ⊆ canonical 8-label set",
    strayLabels.length === 0,
    `stray labels NOT in canonical §11.4: ${JSON.stringify(strayLabels)}`,
  );
}

// ── TP11. Rule 0 cyclic wins over high in-degree ──

{
  const r = canonDraft.classifyTopologySubmodule({
    name: "hub-cyclic",
    inDegree: 20,
    outDegree: 0,
    sccMember: true,
    crossEdgeSource: "full-list",
  });
  assert(
    "TP11. SCC member with high inDegree → cyclic-submodule (Rule 0 wins over Rule 2)",
    r.label === "cyclic-submodule",
    `got=${r.label}`,
  );
}

// ── TP12. isolated-submodule requires crossEdgeSource === "full-list" (degraded-mode guard) ──

{
  // In full-list mode: zero in/out → isolated
  const rFull = canonDraft.classifyTopologySubmodule({
    name: "x",
    inDegree: 0,
    outDegree: 0,
    sccMember: false,
    crossEdgeSource: "full-list",
  });
  assert(
    "TP12a. full-list mode + zero in/out → isolated-submodule",
    rFull.label === "isolated-submodule",
    `got=${rFull.label}`,
  );

  // In top-30-only mode: zero in/out → fall through to scoped (conservative)
  const rDegraded = canonDraft.classifyTopologySubmodule({
    name: "x",
    inDegree: 0,
    outDegree: 0,
    sccMember: false,
    crossEdgeSource: "top-30-only",
  });
  assert(
    "TP12b. top-30-only mode + zero in/out → scoped-submodule (NOT isolated; §11.1 Rule 1 guard)",
    rDegraded.label === "scoped-submodule",
    `got=${rDegraded.label}`,
  );
}

// ── TP13. File classifier thresholds ──

{
  const small = canonDraft.classifyTopologyFile({ file: "small.ts", loc: 399 });
  const exact400 = canonDraft.classifyTopologyFile({ file: "a.ts", loc: 400 });
  const exact1000 = canonDraft.classifyTopologyFile({
    file: "b.ts",
    loc: 1000,
  });
  const huge = canonDraft.classifyTopologyFile({ file: "c.ts", loc: 5000 });

  assert("TP13a. loc < 400 → no label (returns null)", small === null);
  assert("TP13b. loc === 400 → oversize", exact400?.label === "oversize");
  assert(
    "TP13c. loc === 1000 → extreme-oversize (Rule 0 wins at threshold)",
    exact1000?.label === "extreme-oversize",
  );
  assert(
    "TP13d. loc >> 1000 → extreme-oversize",
    huge?.label === "extreme-oversize",
  );
}

// ── TP14. Source-grep pin — no Map<file, ...> as submodule index ──

{
  const src = readAllCanonDraftSources();
  const stripped = src
    .replace(/\/\*[\s\S]*?\*\//g, "")
    .replace(/\/\/[^\n]*/g, "");

  // Flag any Map keyed on filePath that's labeled as a submodule container.
  // Identity-shaped submodule keying would look like `submodulesByFile = new Map`
  // or `submoduleByFile.set(file, ...)` — topology identity is SUBMODULE PATH.
  const suspect = [
    /submodulesByFile\s*=\s*new\s+Map/,
    /\bsubmoduleByFile\s*=\s*new\s+Map/,
  ];
  const flagged = suspect.filter((re) => re.test(stripped));
  assert(
    "TP14. _lib/canon-draft.mjs does NOT use file-keyed Map as submodule index",
    flagged.length === 0,
    `flagged patterns: ${flagged.map((r) => r.toString()).join(", ")}`,
  );
}

// ── TP15. Source-grep pin — classification block reads crossSubmoduleEdges, not crossSubmoduleTop ──

{
  const src = readAllCanonDraftSources();
  const stripped = src
    .replace(/\/\*[\s\S]*?\*\//g, "")
    .replace(/\/\/[^\n]*/g, "");

  // The aggregator that computes inDegree / outDegree for classification
  // MUST consume `crossSubmoduleEdges` (structured full list), NOT
  // `crossSubmoduleTop` (top-30 display). Source-grep: if
  // `crossSubmoduleTop` appears in the file, it must be in a clearly
  // display-labeled context. We approximate by: any assignment like
  // `inDegree = ...crossSubmoduleTop...` or `outDegree = ...crossSubmoduleTop...`
  // is a defect.
  const suspect = [
    /\binDegree\s*=\s*[^;\n]*crossSubmoduleTop/,
    /\boutDegree\s*=\s*[^;\n]*crossSubmoduleTop/,
    /\binDegree\s*\+=\s*[^;\n]*crossSubmoduleTop/,
    /\boutDegree\s*\+=\s*[^;\n]*crossSubmoduleTop/,
  ];
  const flagged = suspect.filter((re) => re.test(stripped));
  assert(
    "TP15. _lib/canon-draft.mjs does NOT derive inDegree/outDegree from crossSubmoduleTop (PF-6)",
    flagged.length === 0,
    `flagged patterns: ${flagged.map((r) => r.toString()).join(", ")}`,
  );
}

// ── TP16. TOPOLOGY_UNCERTAIN_REASONS enum — separate from helper UNCERTAIN_REASONS ──

{
  const exported = canonDraft.TOPOLOGY_UNCERTAIN_REASONS;
  assert(
    "TP16a. TOPOLOGY_UNCERTAIN_REASONS exported + frozen",
    Array.isArray(exported) && Object.isFrozen(exported),
  );
  assert(
    "TP16b. TOPOLOGY_UNCERTAIN_REASONS has exactly 3 values matching spec",
    exported &&
      JSON.stringify([...exported].sort()) ===
        JSON.stringify([...EXPECTED_TOPOLOGY_UNCERTAIN_REASONS].sort()),
    `got=${JSON.stringify(exported)}`,
  );
  assert(
    "TP16c. helper UNCERTAIN_REASONS stays at 4 values (NOT widened by topology addition)",
    Array.isArray(canonDraft.UNCERTAIN_REASONS) &&
      canonDraft.UNCERTAIN_REASONS.length === 4,
  );
}

// ── TP17. Submodule inventory source order documented in canon-draft ──
//
// Classification requires classifier input `crossEdgeSource` ∈ {full-list, top-30-only}
// string literal. Pin the enum.

{
  const r1 = canonDraft.classifyTopologySubmodule({
    name: "z",
    inDegree: 0,
    outDegree: 0,
    sccMember: false,
    crossEdgeSource: "full-list",
  });
  const r2 = canonDraft.classifyTopologySubmodule({
    name: "z",
    inDegree: 0,
    outDegree: 0,
    sccMember: false,
    crossEdgeSource: "top-30-only",
  });
  assert(
    "TP17. classifier branches on crossEdgeSource string literal (isolated vs scoped)",
    r1.label === "isolated-submodule" && r2.label === "scoped-submodule",
  );
}

// ── TP18. Canonical §11.4 block parses with 8 names ──

{
  // §11.4 has a code-fenced list of 8 labels, one per line. Verify parse.
  const marker = "### 11.4 Label set";
  const idx = gatesText.indexOf(marker);
  let parsed = null;
  if (idx >= 0) {
    const rest = gatesText.slice(idx);
    const fenceStart = rest.indexOf("```");
    if (fenceStart >= 0) {
      const fenceContent = rest.slice(fenceStart + 3);
      const fenceEnd = fenceContent.indexOf("```");
      if (fenceEnd >= 0) {
        parsed = fenceContent
          .slice(0, fenceEnd)
          .split("\n")
          .map((s) => s.trim())
          .filter(Boolean);
      }
    }
  }
  assert(
    "TP18. canonical §11.4 label-set block parses to exactly 8 entries matching EXPECTED_TOPOLOGY_LABELS",
    parsed &&
      JSON.stringify(parsed) === JSON.stringify(EXPECTED_TOPOLOGY_LABELS),
    `parsed=${JSON.stringify(parsed)}`,
  );
}

// ════════════════════════════════════════════════════════════════
// P3-4 naming assertions — TN1..TN15
// ════════════════════════════════════════════════════════════════

// ── TN1–TN4. Canonical §12 naming label enumeration ──

const parsedNamingLabels = extractNamingLabelsFromMarkdown(gatesText);
assert(
  "TN1. canonical §9 naming-label enumeration parses to exactly 10 labels",
  parsedNamingLabels && parsedNamingLabels.length === 10,
  `got ${parsedNamingLabels?.length}: ${JSON.stringify(parsedNamingLabels)}`,
);

assert(
  "TN2. canonical §9 naming-label set equals expected 10-label set",
  parsedNamingLabels &&
    new Set(parsedNamingLabels).size === 10 &&
    EXPECTED_NAMING_LABELS.every((l) => parsedNamingLabels.includes(l)),
  `canonical=${JSON.stringify(parsedNamingLabels)}`,
);

assert(
  "TN3. canonical §12 Naming classification section exists",
  /## 12\. Naming classification/.test(gatesText),
);

// Parse §12.3 fenced block — all 10 labels one per line.
{
  const marker = "### 12.3 Label set";
  const idx = gatesText.indexOf(marker);
  let parsed = null;
  if (idx >= 0) {
    const rest = gatesText.slice(idx);
    const fenceStart = rest.indexOf("```");
    if (fenceStart >= 0) {
      const fenceContent = rest.slice(fenceStart + 3);
      const fenceEnd = fenceContent.indexOf("```");
      if (fenceEnd >= 0) {
        parsed = fenceContent
          .slice(0, fenceEnd)
          .split("\n")
          .map((s) => s.trim())
          .filter(Boolean);
      }
    }
  }
  assert(
    "TN4. canonical §12.3 label-set block parses to 10 entries matching EXPECTED_NAMING_LABELS",
    parsed && JSON.stringify(parsed) === JSON.stringify(EXPECTED_NAMING_LABELS),
    `parsed=${JSON.stringify(parsed)}`,
  );
}

// ── TN5–TN8. Code-side mirror + classifier exports ──

assert(
  "TN5. _lib/canon-draft.mjs exports NAMING_LABELS",
  Array.isArray(canonDraft.NAMING_LABELS),
);

assert(
  "TN6. NAMING_LABELS byte-equal to canonical §12.3",
  canonDraft.NAMING_LABELS &&
    Object.isFrozen(canonDraft.NAMING_LABELS) &&
    JSON.stringify([...canonDraft.NAMING_LABELS]) ===
      JSON.stringify(EXPECTED_NAMING_LABELS),
  `mirror=${JSON.stringify(canonDraft.NAMING_LABELS)}`,
);

assert(
  "TN7. NAMING_CONVENTIONS exported + frozen + matches canonical §12.5 (6 values)",
  Array.isArray(canonDraft.NAMING_CONVENTIONS) &&
    Object.isFrozen(canonDraft.NAMING_CONVENTIONS) &&
    JSON.stringify([...canonDraft.NAMING_CONVENTIONS].sort()) ===
      JSON.stringify([...EXPECTED_NAMING_CONVENTIONS].sort()),
  `got=${JSON.stringify(canonDraft.NAMING_CONVENTIONS)}`,
);

assert(
  "TN8. naming classifier surface — classifyNamingCohort + classifyNamingItem + detectConvention + normalizeFileBasename",
  typeof canonDraft.classifyNamingCohort === "function" &&
    typeof canonDraft.classifyNamingItem === "function" &&
    typeof canonDraft.detectConvention === "function" &&
    typeof canonDraft.normalizeFileBasename === "function",
);

// ── TN9. Emitted naming labels ⊆ canonical §12.3 ──

{
  const expected = new Set(EXPECTED_NAMING_LABELS);
  const emitted = new Set();

  // Cohort classifier — exercise every Rule branch.
  // Rule 0: insufficient-evidence (< 3 effective)
  emitted.add(
    canonDraft.classifyNamingCohort({
      cohortId: "x",
      members: [{ name: "a" }, { name: "b" }],
      kind: "file",
      lowInfoExclusions: new Set(),
    }).label,
  );
  // Rule 1: camelCase-dominant
  emitted.add(
    canonDraft.classifyNamingCohort({
      cohortId: "x",
      members: [
        { name: "fooBar" },
        { name: "fooBaz" },
        { name: "fooQux" },
        { name: "fooZap" },
        { name: "FooBar" },
      ],
      kind: "symbol",
      lowInfoExclusions: new Set(),
    }).label,
  );
  // Rule 2: mixed-convention
  emitted.add(
    canonDraft.classifyNamingCohort({
      cohortId: "x",
      members: [
        { name: "fooBar" },
        { name: "FooBar" },
        { name: "foo-bar" },
        { name: "foo_bar" },
        { name: "FOO_BAR" },
      ],
      kind: "symbol",
      lowInfoExclusions: new Set(),
    }).label,
  );

  // Per-item classifier — exercise every Rule branch.
  emitted.add(
    canonDraft.classifyNamingItem({
      name: "get",
      convention: "camelCase",
      dominantConvention: "camelCase",
      isLowInfo: true,
    }).label,
  ); // Rule 0: low-info-excluded
  emitted.add(
    canonDraft.classifyNamingItem({
      name: "foo",
      convention: "camelCase",
      dominantConvention: null,
      isLowInfo: false,
    }).label,
  ); // Rule 1: convention-match (no dominant)
  emitted.add(
    canonDraft.classifyNamingItem({
      name: "fooBar",
      convention: "camelCase",
      dominantConvention: "camelCase",
      isLowInfo: false,
    }).label,
  ); // Rule 2: convention-match
  emitted.add(
    canonDraft.classifyNamingItem({
      name: "FooBar",
      convention: "PascalCase",
      dominantConvention: "camelCase",
      isLowInfo: false,
    }).label,
  ); // Rule 3: convention-outlier

  const strayLabels = [...emitted].filter((l) => !expected.has(l));
  assert(
    "TN9. emitted naming-classifier labels ⊆ canonical 10-label set",
    strayLabels.length === 0,
    `stray labels NOT in canonical §12.3: ${JSON.stringify(strayLabels)}`,
  );
}

// ── TN10. Effective cohort size drives < 3 threshold (P0-3) ──

{
  // Raw 10 members, 8 low-info → effective 2 → insufficient-evidence
  const lowInfo = new Set(["get", "set", "parse", "format", "fetch"]);
  const members = [
    { name: "fooBar" },
    { name: "bazQux" },
    { name: "get" },
    { name: "set" },
    { name: "parse" },
    { name: "format" },
    { name: "fetch" },
    { name: "get" },
    { name: "set" },
    { name: "parse" },
  ];
  const r = canonDraft.classifyNamingCohort({
    cohortId: "x",
    members,
    kind: "symbol",
    lowInfoExclusions: lowInfo,
  });
  assert(
    "TN10. raw 10 members with 8 low-info → effective size 2 → insufficient-evidence",
    r.label === "insufficient-evidence",
    `got=${r.label}`,
  );
}

// ── TN11. low-info-excluded Rule 0 priority at item level (P0-4) ──

{
  // Item is low-info AND matches dominant → still low-info-excluded (Rule 0 wins).
  const r1 = canonDraft.classifyNamingItem({
    name: "get",
    convention: "camelCase",
    dominantConvention: "camelCase",
    isLowInfo: true,
  });
  assert(
    "TN11a. low-info item + matches dominant → low-info-excluded (Rule 0 wins over Rule 2)",
    r1.label === "low-info-excluded",
    `got=${r1.label}`,
  );

  // Item is low-info AND cohort has no dominant → still low-info-excluded
  const r2 = canonDraft.classifyNamingItem({
    name: "get",
    convention: "camelCase",
    dominantConvention: null,
    isLowInfo: true,
  });
  assert(
    "TN11b. low-info item + no dominant → low-info-excluded (Rule 0 wins over Rule 1)",
    r2.label === "low-info-excluded",
    `got=${r2.label}`,
  );

  // Item is low-info AND differs from dominant → still low-info-excluded
  const r3 = canonDraft.classifyNamingItem({
    name: "get",
    convention: "camelCase",
    dominantConvention: "PascalCase",
    isLowInfo: true,
  });
  assert(
    "TN11c. low-info item + differs from dominant → low-info-excluded (Rule 0 wins over Rule 3)",
    r3.label === "low-info-excluded",
    `got=${r3.label}`,
  );
}

// ── TN12. Basename normalization (P0-2) ──

assert(
  "TN12a. `_lib/canon-draft.mjs` normalizes to `canon-draft`",
  canonDraft.normalizeFileBasename("_lib/canon-draft.mjs") === "canon-draft",
);
assert(
  "TN12b. `src/UserCard.tsx` normalizes to `UserCard`",
  canonDraft.normalizeFileBasename("src/UserCard.tsx") === "UserCard",
);
assert(
  "TN12c. `tests/user-profile.test.tsx` strips .test + .tsx → `user-profile`",
  canonDraft.normalizeFileBasename("tests/user-profile.test.tsx") ===
    "user-profile",
);
assert(
  "TN12d. `src/api.d.ts` strips .d + .ts → `api`",
  canonDraft.normalizeFileBasename("src/api.d.ts") === "api",
);
assert(
  "TN12e. `src/Foo.stories.tsx` strips .stories + .tsx → `Foo`",
  canonDraft.normalizeFileBasename("src/Foo.stories.tsx") === "Foo",
);

// ── TN13. detectConvention sanity ──

assert(
  "TN13a. detectConvention(`fooBar`) === `camelCase`",
  canonDraft.detectConvention("fooBar") === "camelCase",
);
assert(
  "TN13b. detectConvention(`FooBar`) === `PascalCase`",
  canonDraft.detectConvention("FooBar") === "PascalCase",
);
assert(
  "TN13c. detectConvention(`foo-bar`) === `kebab-case`",
  canonDraft.detectConvention("foo-bar") === "kebab-case",
);
assert(
  "TN13d. detectConvention(`foo_bar`) === `snake_case`",
  canonDraft.detectConvention("foo_bar") === "snake_case",
);
assert(
  "TN13e. detectConvention(`FOO_BAR`) === `UPPER_SNAKE`",
  canonDraft.detectConvention("FOO_BAR") === "UPPER_SNAKE",
);
assert(
  "TN13f. detectConvention(`Foo_bar`) === `mixed`",
  canonDraft.detectConvention("Foo_bar") === "mixed",
);

// ── TN14. NAMING_UNCERTAIN_REASONS + CANON_DRAFT_SOURCES ──

assert(
  "TN14a. NAMING_UNCERTAIN_REASONS frozen + matches expected (2 values)",
  Array.isArray(canonDraft.NAMING_UNCERTAIN_REASONS) &&
    Object.isFrozen(canonDraft.NAMING_UNCERTAIN_REASONS) &&
    JSON.stringify([...canonDraft.NAMING_UNCERTAIN_REASONS].sort()) ===
      JSON.stringify([...EXPECTED_NAMING_UNCERTAIN_REASONS].sort()),
  `got=${JSON.stringify(canonDraft.NAMING_UNCERTAIN_REASONS)}`,
);

assert(
  "TN14b. `cohort-insufficient-evidence` reason string distinct from `insufficient-evidence` label",
  canonDraft.NAMING_UNCERTAIN_REASONS?.includes(
    "cohort-insufficient-evidence",
  ) && !canonDraft.NAMING_UNCERTAIN_REASONS?.includes("insufficient-evidence"),
);

assert(
  "TN14c. CANON_DRAFT_SOURCES exported + frozen + 4 values",
  Array.isArray(canonDraft.CANON_DRAFT_SOURCES) &&
    Object.isFrozen(canonDraft.CANON_DRAFT_SOURCES) &&
    JSON.stringify([...canonDraft.CANON_DRAFT_SOURCES]) ===
      JSON.stringify(EXPECTED_CANON_DRAFT_SOURCES),
  `got=${JSON.stringify(canonDraft.CANON_DRAFT_SOURCES)}`,
);

// ── TN15. Source-grep pin — naming classifier does NOT key cohorts on ownerFile::exportedName ──

{
  const src = readAllCanonDraftSources();
  const stripped = src
    .replace(/\/\*[\s\S]*?\*\//g, "")
    .replace(/\/\/[^\n]*/g, "");

  // Naming cohorts keyed on submodule (file) or submodule::kind (symbol).
  // Any Map<ownerFile::exportedName, ...> labeled as a naming cohort index
  // is a defect. We approximate with `cohortsByIdentity` or
  // `cohortsByOwnerExport` suspect patterns.
  const suspect = [
    /cohortsByOwnerExport\s*=\s*new\s+Map/,
    /namingCohortsByIdentity\s*=\s*new\s+Map/,
  ];
  const flagged = suspect.filter((re) => re.test(stripped));
  assert(
    "TN15. _lib/canon-draft.mjs does NOT key naming cohorts on ownerFile::exportedName",
    flagged.length === 0,
    `flagged patterns: ${flagged.map((r) => r.toString()).join(", ")}`,
  );
}

// ── FACADE-PIN-1 (post-P3 cleanup, 2026-04-21) ──────────────
//
// `_lib/canon-draft.mjs` is a drift-test-only facade after the 5-module
// split. This test file is the ONLY legitimate consumer (uses namespace
// import at line ~237 for drift-test across all mirrors).
//
// Production code MUST import from the specific leaf module
// (`canon-draft-utils` / `-types` / `-helpers` / `-topology` / `-naming`).
// This pin grep-walks every `.mjs` outside `tests/` and flags any that
// imports from the facade — catching regressions where a future consumer
// re-adopts the god-module surface.

{
  const { readdirSync, statSync } = await import("node:fs");
  const REPO_ROOT = DIR;
  // Directories to scan. `.mjs` production code lives at repo root +
  // `_lib/` + `scripts/`. Tests are excluded (this file is the 1 allowed
  // consumer).
  const searchRoots = ["", "_lib", "scripts"];
  const offenders = [];

  function scan(dir) {
    let entries;
    try {
      entries = readdirSync(dir, { withFileTypes: true });
    } catch {
      return;
    }
    for (const e of entries) {
      if (e.name.startsWith(".") || e.name === "node_modules") continue;
      const full = path.join(dir, e.name);
      if (e.isDirectory()) {
        // Only recurse into the three allowed search roots and their
        // subdirectories; skip tests/, output/, canonical-draft/.
        const rel = full.slice(REPO_ROOT.length + 1).replace(/\\/g, "/");
        if (
          rel.startsWith("tests") ||
          rel.startsWith("output") ||
          rel.startsWith("canonical-draft") ||
          rel.startsWith("canonical") ||
          rel.startsWith("node_modules") ||
          rel.startsWith("p1") ||
          rel.startsWith("p2") ||
          rel.startsWith("p3")
        )
          continue;
        scan(full);
      } else if (e.name.endsWith(".mjs")) {
        const src = readFileSync(full, "utf8");
        // Match: `from '.../canon-draft.mjs'` or `await import('.../canon-draft.mjs')`
        // but NOT sibling helpers such as `audit-canon-draft.mjs`.
        const re =
          /(?:from\s+|await\s+import\s*\(\s*)['"](?:[^'"]*[\\/])?canon-draft\.mjs['"]/g;
        if (re.test(src)) {
          offenders.push(full.slice(REPO_ROOT.length + 1).replace(/\\/g, "/"));
        }
      }
    }
  }

  // Scan repo root (non-recursive for dirs other than the allowed roots).
  for (const sub of searchRoots) {
    const dir = sub ? path.join(REPO_ROOT, sub) : REPO_ROOT;
    try {
      if (sub === "") {
        // Only `.mjs` files at repo root, not subdirs.
        const entries = readdirSync(dir, { withFileTypes: true });
        for (const e of entries) {
          if (!e.isFile() || !e.name.endsWith(".mjs")) continue;
          const full = path.join(dir, e.name);
          const src = readFileSync(full, "utf8");
          const re =
            /(?:from\s+|await\s+import\s*\(\s*)['"](?:[^'"]*[\\/])?canon-draft\.mjs['"]/g;
          if (re.test(src)) offenders.push(e.name);
        }
      } else {
        if (statSync(dir).isDirectory()) scan(dir);
      }
    } catch {
      /* dir may not exist — benign */
    }
  }

  assert(
    "FACADE-PIN-1. production code does NOT import from _lib/canon-draft.mjs facade (leaf modules only)",
    offenders.length === 0,
    `offenders: ${offenders.join(", ")}`,
  );
}

// ── DC-1..DC-18. canon-drift.md drift category mirror (P5-0) ───
//
// These assertions pin the drift category + family-tag enums defined
// in `canonical/canon-drift.md` §3. Until that file is written in
// P5-0 step 1, these assertions run RED (intentional — step 0 is the
// RED test commit). Step 1 lands canon-drift.md and turns them GREEN.
//
// Mirrors `docs/history/phases/p5/p5-0.md` §4.3 — category → family mapping is 1:1 in v1.

const EXPECTED_DRIFT_KINDS = [
  "type-drift",
  "helper-drift",
  "topology-drift",
  "naming-drift",
];

// Per-kind category enum. 19 total across the 4 kinds.
const EXPECTED_DRIFT_CATEGORIES = {
  "type-drift": [
    "identity-added",
    "identity-removed",
    "label-changed",
    "owner-changed",
  ],
  "helper-drift": [
    "helper-added",
    "helper-removed",
    "label-changed",
    "contamination-changed",
    "fan-in-tier-changed",
  ],
  "topology-drift": [
    "submodule-added",
    "submodule-removed",
    "scc-status-changed",
    "oversize-changed",
    "cross-edge-added",
    "cross-edge-removed",
  ],
  "naming-drift": [
    "cohort-added",
    "cohort-removed",
    "cohort-convention-shifted",
    "new-outlier-introduced",
    "outlier-resolved",
  ],
};

const EXPECTED_FAMILY_TAGS = [
  "added",
  "removed",
  "label-changed",
  "structural-status-changed",
  "content-shifted",
];

// Category → family. Keyed `<kind>::<category>` because `label-changed`
// appears as a category in multiple kinds AND as a family; the compound
// key disambiguates.
const EXPECTED_CATEGORY_TO_FAMILY = {
  "type-drift::identity-added": "added",
  "type-drift::identity-removed": "removed",
  "type-drift::label-changed": "label-changed",
  "type-drift::owner-changed": "structural-status-changed",
  "helper-drift::helper-added": "added",
  "helper-drift::helper-removed": "removed",
  "helper-drift::label-changed": "label-changed",
  "helper-drift::contamination-changed": "content-shifted",
  "helper-drift::fan-in-tier-changed": "label-changed",
  "topology-drift::submodule-added": "added",
  "topology-drift::submodule-removed": "removed",
  "topology-drift::scc-status-changed": "structural-status-changed",
  "topology-drift::oversize-changed": "content-shifted",
  "topology-drift::cross-edge-added": "added",
  "topology-drift::cross-edge-removed": "removed",
  "naming-drift::cohort-added": "added",
  "naming-drift::cohort-removed": "removed",
  "naming-drift::cohort-convention-shifted": "label-changed",
  "naming-drift::new-outlier-introduced": "content-shifted",
  "naming-drift::outlier-resolved": "content-shifted",
};

const EXPECTED_PERSOURCE_STATUS_DOMAIN = [
  "drift",
  "clean",
  "skipped-missing-canon",
  "skipped-unrecognized-schema",
  "parse-error",
];

const EXPECTED_SOURCE_FILE_NAMES = [
  "type-ownership.md",
  "helper-registry.md",
  "topology.md",
  "naming.md",
];

const driftPath = path.join(DIR, "canonical", "canon-drift.md");
let driftText = "";
let driftExists = false;
try {
  driftText = readFileSync(driftPath, "utf8");
  driftExists = true;
} catch {
  /* RED until step 1 lands canon-drift.md */
}

assert(
  "DC-1. canonical/canon-drift.md exists (step 1 makes this GREEN)",
  driftExists,
  "expected canonical/canon-drift.md present",
);

// Parse §3 category → family mapping from the canonical table.
//
// Expected §3 format (per p5-0.md §4.3):
//   ```
//   <kind>:
//     <category>    (family: <family>)
//     ...
//   ```
// We extract `<kind>::<category> → <family>` pairs by scanning every
// fenced code block in §3 for the `(family: ...)` pattern, plus
// tracking the most recent kind header seen inside the block.
function extractDriftCategoryMap(text) {
  const sectionStart = text.indexOf("## 3.");
  if (sectionStart < 0) return null;
  const section4 = text.indexOf("## 4.", sectionStart);
  const body = text.slice(sectionStart, section4 > 0 ? section4 : undefined);

  const categoryToFamily = {};
  const allFamilies = new Set();
  const kindHeaderRe =
    /^(type-drift|helper-drift|topology-drift|naming-drift)\s*:\s*$/;
  const categoryLineRe =
    /^\s+([a-z][a-z-]+)\s*\(family:\s*([a-z][a-z-]+)\s*\)\s*$/;

  let currentKind = null;
  for (const line of body.split(/\r?\n/)) {
    const kh = line.trim().match(kindHeaderRe);
    if (kh) {
      currentKind = kh[1];
      continue;
    }
    const cl = line.match(categoryLineRe);
    if (cl && currentKind) {
      const cat = cl[1];
      const fam = cl[2];
      categoryToFamily[`${currentKind}::${cat}`] = fam;
      allFamilies.add(fam);
    }
  }
  return { categoryToFamily, allFamilies };
}

const parsed = driftExists ? extractDriftCategoryMap(driftText) : null;
const parsedMap = parsed?.categoryToFamily ?? {};

assert(
  "DC-2. §1 Purpose section present",
  driftText.includes("## 1. Purpose"),
  'expected "## 1. Purpose" heading',
);
assert(
  "DC-3. §2 Drift fact kinds section present",
  driftText.includes("## 2. Drift fact kinds"),
  'expected "## 2. Drift fact kinds" heading',
);
assert(
  "DC-4. §3 Drift categories section present",
  driftText.includes("## 3."),
  'expected "## 3." section',
);
assert(
  "DC-5. §4 Identity contract section present",
  driftText.includes("## 4. Identity contract"),
  'expected "## 4. Identity contract" heading',
);
assert(
  "DC-6. §5 Parser contract section present",
  driftText.includes("## 5. Parser contract"),
  'expected "## 5. Parser contract" heading',
);
assert(
  "DC-7. §6 JSON artifact shape section present",
  driftText.includes("## 6. JSON artifact shape"),
  'expected "## 6. JSON artifact shape" heading',
);
assert(
  "DC-8. §7 Non-goals section present",
  driftText.includes("## 7. Non-goals"),
  'expected "## 7. Non-goals" heading',
);

const section2Slice = (() => {
  const s = driftText.indexOf("## 2.");
  const e = driftText.indexOf("## 3.", s);
  return s >= 0 ? driftText.slice(s, e > 0 ? e : undefined) : "";
})();
assert(
  "DC-9. §2 names every one of the 4 drift kinds",
  EXPECTED_DRIFT_KINDS.every((k) => section2Slice.includes("`" + k + "`")),
  `missing kinds in §2 from: ${EXPECTED_DRIFT_KINDS.filter((k) => !section2Slice.includes("`" + k + "`")).join(", ")}`,
);

assert(
  "DC-10. §3 category enum count equals 20 (4 + 5 + 6 + 5)",
  Object.keys(parsedMap).length === 20,
  `parsed ${Object.keys(parsedMap).length} categories; expected 20`,
);

assert(
  "DC-11. §3 per-kind category enum matches expected",
  EXPECTED_DRIFT_KINDS.every((kind) =>
    EXPECTED_DRIFT_CATEGORIES[kind].every((cat) =>
      Object.prototype.hasOwnProperty.call(parsedMap, `${kind}::${cat}`),
    ),
  ),
  "some expected `<kind>::<category>` pairs missing from §3 parse",
);

assert(
  "DC-12. §3 family tag set equals exactly 5 canonical values",
  parsed &&
    parsed.allFamilies.size === 5 &&
    EXPECTED_FAMILY_TAGS.every((f) => parsed.allFamilies.has(f)),
  `got families: ${parsed ? [...parsed.allFamilies].sort().join(",") : "(none)"}`,
);

assert(
  "DC-13. §3 category → family mapping is 1:1 per canonical table",
  Object.entries(EXPECTED_CATEGORY_TO_FAMILY).every(
    ([k, v]) => parsedMap[k] === v,
  ),
  "mapping mismatch — inspect §3 for drifted category/family entries",
);

const section4Slice = (() => {
  const s = driftText.indexOf("## 4.");
  const e = driftText.indexOf("## 5.", s);
  return s >= 0 ? driftText.slice(s, e > 0 ? e : undefined) : "";
})();
assert(
  "DC-14. §4 identity contract mentions each of the 4 kinds",
  EXPECTED_DRIFT_KINDS.every((k) => section4Slice.includes(k)),
  `missing kinds in §4 from: ${EXPECTED_DRIFT_KINDS.filter((k) => !section4Slice.includes(k)).join(", ")}`,
);

const section5Slice = (() => {
  const s = driftText.indexOf("## 5.");
  const e = driftText.indexOf("## 6.", s);
  return s >= 0 ? driftText.slice(s, e > 0 ? e : undefined) : "";
})();
assert(
  "DC-15. §5 parser contract names all 4 source files",
  EXPECTED_SOURCE_FILE_NAMES.every((f) => section5Slice.includes(f)),
  `missing source filenames in §5: ${EXPECTED_SOURCE_FILE_NAMES.filter((f) => !section5Slice.includes(f)).join(", ")}`,
);

const section6Slice = (() => {
  const s = driftText.indexOf("## 6.");
  const e = driftText.indexOf("## 7.", s);
  return s >= 0 ? driftText.slice(s, e > 0 ? e : undefined) : "";
})();
assert(
  "DC-16. §6 JSON shape lists all 5 perSource.status domain values",
  EXPECTED_PERSOURCE_STATUS_DOMAIN.every((v) =>
    section6Slice.includes("`" + v + "`"),
  ),
  `missing status values in §6: ${EXPECTED_PERSOURCE_STATUS_DOMAIN.filter((v) => !section6Slice.includes("`" + v + "`")).join(", ")}`,
);

assert(
  "DC-17. §6 JSON shape shows drifts[] carrying {kind, category, family, identity}",
  ['"kind"', '"category"', '"family"', '"identity"'].every((k) =>
    section6Slice.includes(k),
  ),
  "expected JSON example keys kind/category/family/identity in §6",
);

assert(
  "DC-18. §5 parser contract cross-refs classification-gates.md label sets",
  section5Slice.includes("classification-gates.md"),
  "expected §5 to cross-ref classification-gates.md for label validation",
);
