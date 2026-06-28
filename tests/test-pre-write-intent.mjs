// Tests for _lib/pre-write-intent.mjs — P1-1 step 5.2.
//
// Pinning rules from docs/history/phases/p1/p1-1.md §2.1.1 + §4.2 + §5.2:
//   - 5 top-level arrays are normalized: names, shapes, files,
//     dependencies, plannedTypeEscapes. Empty arrays are OK; missing
//     arrays default to [] with warnings.
//   - names/dependencies accept strings or structured self-declarations.
//   - shapes require fields unless an exact hash/typeLiteral is supplied.
//   - plannedTypeEscapes entries are objects, not strings.
//   - escapeKind MUST be one of the 10 canonical/fact-model.md §3.9 values.
//   - reason REQUIRED on every planned-escape entry.
//   - locationHint REQUIRED (can be the literal string 'unknown').
//   - codeShape / alternativeConsidered OPTIONAL.
//   - On failure: {ok: false, error, errorPath}. errorPath cites the
//     failing key path (e.g. 'plannedTypeEscapes[1].escapeKind').

import { validateIntent, ESCAPE_KINDS } from "../_lib/pre-write-intent.mjs";

let passed = 0,
  failed = 0;
function assert(label, ok, detail = "") {
  if (ok) {
    passed++;
    console.log(`  PASS  ${label}`);
  } else {
    failed++;
    console.log(`  FAIL  ${label}\n        ${detail}`);
  }
}

// ═══ Baseline: all-empty intent ═══

{
  const r = validateIntent({
    names: [],
    shapes: [],
    files: [],
    dependencies: [],
    plannedTypeEscapes: [],
  });
  assert(
    "T1. all-empty intent → ok:true",
    r.ok === true,
    `r=${JSON.stringify(r)}`,
  );
  assert(
    "T1b. normalized intent keeps 5 empty arrays",
    r.ok &&
      Array.isArray(r.intent.names) &&
      r.intent.names.length === 0 &&
      Array.isArray(r.intent.shapes) &&
      r.intent.shapes.length === 0 &&
      Array.isArray(r.intent.files) &&
      r.intent.files.length === 0 &&
      Array.isArray(r.intent.dependencies) &&
      r.intent.dependencies.length === 0 &&
      Array.isArray(r.intent.plannedTypeEscapes) &&
      r.intent.plannedTypeEscapes.length === 0,
  );
  assert(
    "T1c. fully specified intent has no schema warnings",
    r.ok && Array.isArray(r.warnings) && r.warnings.length === 0,
    `warnings=${JSON.stringify(r.warnings)}`,
  );
}

// ═══ Missing keys ═══

{
  const r = validateIntent({
    names: [],
    shapes: [],
    files: [],
    dependencies: [],
    // plannedTypeEscapes missing
  });
  assert(
    "T2. missing plannedTypeEscapes key defaults to []",
    r.ok === true &&
      Array.isArray(r.intent.plannedTypeEscapes) &&
      r.intent.plannedTypeEscapes.length === 0,
    `r=${JSON.stringify(r)}`,
  );
  assert(
    "T2b. missing plannedTypeEscapes emits intent warning",
    r.warnings?.some(
      (w) =>
        w.kind === "missing-intent-key-defaulted" &&
        w.key === "plannedTypeEscapes" &&
        w.action === "defaulted-to-empty-array",
    ),
    `warnings=${JSON.stringify(r.warnings)}`,
  );
}

{
  const r = validateIntent({
    names: [],
    shapes: [],
    files: [],
    plannedTypeEscapes: [],
    // dependencies missing
  });
  assert(
    "T3. missing dependencies key defaults to [] with warning",
    r.ok === true &&
      Array.isArray(r.intent.dependencies) &&
      r.intent.dependencies.length === 0 &&
      r.warnings?.some(
        (w) =>
          w.kind === "missing-intent-key-defaulted" && w.key === "dependencies",
      ),
    `r=${JSON.stringify(r)}`,
  );
}

{
  const r = validateIntent({});
  assert(
    "T4. empty object defaults all five arrays",
    r.ok === true &&
      Array.isArray(r.intent.names) &&
      Array.isArray(r.intent.shapes) &&
      Array.isArray(r.intent.files) &&
      Array.isArray(r.intent.dependencies) &&
      Array.isArray(r.intent.plannedTypeEscapes),
    `r=${JSON.stringify(r)}`,
  );
  assert(
    "T4b. empty object emits five missing-key warnings",
    r.warnings?.length === 5 &&
      r.warnings.every((w) => w.kind === "missing-intent-key-defaulted"),
    `warnings=${JSON.stringify(r.warnings)}`,
  );
}

// ═══ Wrong top-level types ═══

{
  const r = validateIntent({
    names: "formatDate", // string instead of array
    shapes: [],
    files: [],
    dependencies: [],
    plannedTypeEscapes: [],
  });
  assert("T5. names as string → ok:false", r.ok === false);
  assert("T5b. errorPath cites names", r.errorPath === "names");
}

{
  const r = validateIntent({
    names: [42, "valid"], // non-string element
    shapes: [],
    files: [],
    dependencies: [],
    plannedTypeEscapes: [],
  });
  assert(
    "T6. names contains unsupported non-string → ok:false",
    r.ok === false,
  );
  assert(
    "T6b. errorPath cites the indexed element",
    /^names\[\d+\]/.test(r.errorPath),
    `errorPath=${r.errorPath}`,
  );
}

{
  const r = validateIntent({
    names: [
      "formatDate",
      { name: "formatTimestamp", kind: "function", why: "display helper" },
    ],
    shapes: [],
    files: [],
    dependencies: [],
    plannedTypeEscapes: [],
  });
  assert(
    "T6c. names accept structured declarations and normalize to strings",
    r.ok === true &&
      r.intent.names.join(",") === "formatDate,formatTimestamp" &&
      r.intent.nameDeclarations?.[0]?.why === "display helper",
    JSON.stringify(r),
  );
}

{
  const r = validateIntent({
    names: [{ kind: "function", why: "missing name" }],
    shapes: [],
    files: [],
    dependencies: [],
    plannedTypeEscapes: [],
  });
  assert(
    "T6d. structured name declaration requires name",
    r.ok === false && r.errorPath === "names[0].name",
    JSON.stringify(r),
  );
}

{
  const r = validateIntent({
    names: [
      {
        name: "searchMime",
        kind: "function",
        why: "search MIME helpers before adding another helper",
        ownerFile: "src/utils/mime.ts",
      },
      {
        name: "lookupCookie",
        file: "src/helper/cookie/index.ts",
      },
      {
        name: "queryPath",
        targetFile: "src/utils/url.ts",
      },
    ],
    shapes: [],
    files: [],
    dependencies: [],
    plannedTypeEscapes: [],
  });
  assert(
    "T6e. structured name declarations preserve owner locality hints",
    r.ok === true &&
      r.intent.nameDeclarations?.[0]?.ownerFile === "src/utils/mime.ts" &&
      r.intent.nameDeclarations?.[1]?.file === "src/helper/cookie/index.ts" &&
      r.intent.nameDeclarations?.[1]?.ownerFile ===
        "src/helper/cookie/index.ts" &&
      r.intent.nameDeclarations?.[2]?.targetFile === "src/utils/url.ts" &&
      r.intent.nameDeclarations?.[2]?.ownerFile === "src/utils/url.ts",
    JSON.stringify(r),
  );
}

{
  const r = validateIntent(null);
  assert("T7. null input → ok:false", r.ok === false);
}

{
  const r = validateIntent("not an object");
  assert("T8. string input → ok:false", r.ok === false);
}

// ═══ shapes — {fields: string[]} ═══

{
  const r = validateIntent({
    names: [],
    shapes: [{ fields: ["a", "b"] }],
    files: [],
    dependencies: [],
    plannedTypeEscapes: [],
  });
  assert("T9. shapes with fields array → ok:true", r.ok === true);
}

{
  const hash = `sha256:${"a".repeat(64)}`;
  const r = validateIntent({
    names: [],
    shapes: [{ fields: [], hash }],
    files: [],
    dependencies: [],
    plannedTypeEscapes: [],
  });
  assert(
    "T9b. shapes may carry optional exact shape hash",
    r.ok === true && r.intent.shapes[0].hash === hash,
  );
}

{
  const r = validateIntent({
    names: [],
    shapes: [{ fields: [], typeLiteral: "{ id: string }" }],
    files: [],
    dependencies: [],
    plannedTypeEscapes: [],
  });
  assert(
    "T9c. shapes may carry optional typeLiteral",
    r.ok === true && r.intent.shapes[0].typeLiteral === "{ id: string }",
  );
}

{
  const r = validateIntent({
    names: [],
    shapes: [
      {
        name: "TimestampViewModel",
        typeLiteral: "{ label: string; iso: string; timezone: string }",
        why: "view model contract",
      },
    ],
    files: [],
    dependencies: [],
    plannedTypeEscapes: [],
  });
  assert(
    "T9d. shape with exact typeLiteral does not require fields array",
    r.ok === true &&
      r.intent.shapes[0].fields.length === 0 &&
      r.intent.shapes[0].name === "TimestampViewModel" &&
      r.intent.shapes[0].why === "view model contract",
    JSON.stringify(r),
  );
}

{
  const r = validateIntent({
    names: [],
    shapes: [{}],
    files: [],
    dependencies: [],
    plannedTypeEscapes: [],
  });
  assert(
    "T9e. shape without fields/hash/typeLiteral remains invalid",
    r.ok === false && r.errorPath === "shapes[0].fields",
    JSON.stringify(r),
  );
}

{
  const r = validateIntent({
    names: [],
    shapes: [{ fields: "a,b" }],
    files: [],
    dependencies: [],
    plannedTypeEscapes: [],
  });
  assert("T10. shapes with non-array fields → ok:false", r.ok === false);
  assert(
    "T10b. errorPath cites shapes[0].fields",
    /^shapes\[0\]\.fields/.test(r.errorPath),
  );
}

{
  const r = validateIntent({
    names: [],
    shapes: [{ fields: [], hash: "abc" }],
    files: [],
    dependencies: [],
    plannedTypeEscapes: [],
  });
  assert(
    "T10c. malformed shape hash → ok:false",
    r.ok === false && r.errorPath === "shapes[0].hash",
    JSON.stringify(r),
  );
}

{
  const r = validateIntent({
    names: [],
    shapes: [{ fields: [], typeLiteral: "" }],
    files: [],
    dependencies: [],
    plannedTypeEscapes: [],
  });
  assert(
    "T10d. empty typeLiteral → ok:false",
    r.ok === false && r.errorPath === "shapes[0].typeLiteral",
    JSON.stringify(r),
  );
}

// ═══ plannedTypeEscapes — all 11 escapeKind values ═══

for (const kind of ESCAPE_KINDS) {
  const r = validateIntent({
    names: [],
    shapes: [],
    files: [],
    dependencies: [],
    plannedTypeEscapes: [
      {
        escapeKind: kind,
        locationHint: "src/vendor/x.ts::fn",
        reason: "test reason",
      },
    ],
  });
  assert(
    `T11-${kind}. valid escapeKind "${kind}" accepted`,
    r.ok === true,
    `r=${JSON.stringify(r)}`,
  );
}

assert(
  "T12. ESCAPE_KINDS enumerates exactly 11 values",
  ESCAPE_KINDS.length === 11,
);

// ═══ plannedTypeEscapes — rejection cases ═══

{
  const r = validateIntent({
    names: [],
    shapes: [],
    files: [],
    dependencies: [],
    plannedTypeEscapes: [
      {
        escapeKind: "any", // not in enum — enum uses 'explicit-any'
        locationHint: "src/x.ts",
        reason: "test",
      },
    ],
  });
  assert('T13. non-enum escapeKind "any" → ok:false', r.ok === false);
  assert(
    "T13b. errorPath cites plannedTypeEscapes[0].escapeKind",
    r.errorPath === "plannedTypeEscapes[0].escapeKind",
    `errorPath=${r.errorPath}`,
  );
}

{
  const r = validateIntent({
    names: [],
    shapes: [],
    files: [],
    dependencies: [],
    plannedTypeEscapes: [
      {
        escapeKind: "as-any",
        locationHint: "src/x.ts",
        // reason missing
      },
    ],
  });
  assert("T14. missing reason → ok:false", r.ok === false);
  assert(
    "T14b. errorPath cites plannedTypeEscapes[0].reason",
    r.errorPath === "plannedTypeEscapes[0].reason",
  );
}

{
  const r = validateIntent({
    names: [],
    shapes: [],
    files: [],
    dependencies: [],
    plannedTypeEscapes: [
      {
        escapeKind: "as-any",
        reason: "test",
        // locationHint missing
      },
    ],
  });
  assert("T15. missing locationHint → ok:false", r.ok === false);
  assert(
    "T15b. errorPath cites plannedTypeEscapes[0].locationHint",
    r.errorPath === "plannedTypeEscapes[0].locationHint",
  );
}

{
  const r = validateIntent({
    names: [],
    shapes: [],
    files: [],
    dependencies: [],
    plannedTypeEscapes: [
      {
        escapeKind: "as-any",
        locationHint: "", // empty string
        reason: "test",
      },
    ],
  });
  assert("T16. empty-string locationHint → ok:false", r.ok === false);
}

{
  const r = validateIntent({
    names: [],
    shapes: [],
    files: [],
    dependencies: [],
    plannedTypeEscapes: [
      {
        escapeKind: "as-any",
        locationHint: "unknown", // literal 'unknown' string IS acceptable
        reason: "third-party SDK lacks type exports",
      },
    ],
  });
  assert(
    'T17. locationHint "unknown" (literal) → ok:true',
    r.ok === true,
    `r=${JSON.stringify(r)}`,
  );
}

// ═══ dependencies — terse or structured ═══

{
  const r = validateIntent({
    names: [],
    shapes: [],
    files: [],
    dependencies: [
      "date-fns",
      { specifier: "dayjs", why: "timestamp formatting" },
    ],
    plannedTypeEscapes: [],
  });
  assert(
    "T17b. dependencies accept structured declarations and normalize to strings",
    r.ok === true &&
      r.intent.dependencies.join(",") === "date-fns,dayjs" &&
      r.intent.dependencyDeclarations?.[0]?.why === "timestamp formatting",
    JSON.stringify(r),
  );
}

{
  const r = validateIntent({
    names: [],
    shapes: [],
    files: [],
    dependencies: [{ why: "missing specifier" }],
    plannedTypeEscapes: [],
  });
  assert(
    "T17c. structured dependency declaration requires specifier",
    r.ok === false && r.errorPath === "dependencies[0].specifier",
    JSON.stringify(r),
  );
}

// Second entry bad — errorPath should include index 1.
{
  const r = validateIntent({
    names: [],
    shapes: [],
    files: [],
    dependencies: [],
    plannedTypeEscapes: [
      { escapeKind: "as-any", locationHint: "src/a.ts", reason: "valid" },
      { escapeKind: "bogus", locationHint: "src/b.ts", reason: "second" },
    ],
  });
  assert(
    "T18. second entry bad → errorPath uses index 1",
    r.ok === false && r.errorPath === "plannedTypeEscapes[1].escapeKind",
    `errorPath=${r.errorPath}`,
  );
}

// codeShape / alternativeConsidered are optional — both present.
{
  const r = validateIntent({
    names: [],
    shapes: [],
    files: [],
    dependencies: [],
    plannedTypeEscapes: [
      {
        escapeKind: "as-unknown-as-T",
        locationHint: "src/vendor/wrapper.ts::adaptResponse",
        codeShape: "response as unknown as ThirdPartyShape",
        reason: "upstream SDK lacks type exports",
        alternativeConsidered:
          "unknown + decoder; rejected because runtime validation library not yet approved",
      },
    ],
  });
  assert(
    "T19. optional fields present → ok:true, preserved",
    r.ok === true &&
      r.intent.plannedTypeEscapes[0].codeShape ===
        "response as unknown as ThirdPartyShape" &&
      r.intent.plannedTypeEscapes[0].alternativeConsidered?.startsWith(
        "unknown + decoder",
      ),
  );
}

// ═══ Normalization: extra keys rejected or stripped? ═══

{
  // Extra top-level keys beyond the 5 required are allowed but stripped
  // in the normalized output. This keeps the schema forward-compatible
  // if future intents grow.
  const r = validateIntent({
    names: [],
    shapes: [],
    files: [],
    dependencies: [],
    plannedTypeEscapes: [],
    taskId: "TASK-42", // extra, allowed
  });
  assert(
    "T20. extra top-level keys allowed; taskId preserved on normalized intent",
    r.ok === true,
  );
}

// ═══ refactorSources — optional inline extraction source hints ═══

{
  const r = validateIntent({
    names: ["writeOrDestroyConnection"],
    shapes: [],
    files: ["src/connection-write.ts"],
    dependencies: [],
    plannedTypeEscapes: [],
    refactorSources: [
      {
        file: "src/server.ts",
        lines: [498, 577, 661, 689],
        why: "extract repeated catch-destroy handling",
      },
    ],
  });
  assert(
    "T21. refactorSources valid entry → ok:true",
    r.ok === true,
    JSON.stringify(r),
  );
  assert(
    "T21b. refactorSources preserved in normalized intent",
    r.ok === true &&
      r.intent.refactorSources?.[0]?.file === "src/server.ts" &&
      r.intent.refactorSources?.[0]?.lines?.join(",") === "498,577,661,689" &&
      r.intent.refactorSources?.[0]?.why ===
        "extract repeated catch-destroy handling",
    JSON.stringify(r.intent.refactorSources),
  );
}

{
  const r = validateIntent({
    names: [],
    shapes: [],
    files: [],
    dependencies: [],
    plannedTypeEscapes: [],
    refactorSources: [{ file: "../server.ts", lines: [1] }],
  });
  assert(
    "T22. refactorSources rejects parent traversal path",
    r.ok === false && r.errorPath === "refactorSources[0].file",
    JSON.stringify(r),
  );
}

{
  const r = validateIntent({
    names: [],
    shapes: [],
    files: [],
    dependencies: [],
    plannedTypeEscapes: [],
    refactorSources: [{ file: "src/server.ts", lines: [0] }],
  });
  assert(
    "T23. refactorSources rejects non-positive line",
    r.ok === false && r.errorPath === "refactorSources[0].lines[0]",
    JSON.stringify(r),
  );
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
