import { it } from "vitest";
// Tests for _lib/post-write-render.mjs — P2-1 step 2.
//
// Pinning rules from docs/history/phases/p2/p2-1.md v3 §5.2 + §4.5.
//   Section order: Any delta → Planned and observed → New escape sites →
//   Observed without baseline → Planned but not observed → Removed →
//   Inventory completeness → Baseline/Capability/Scan-range status → summary.
//   "Any delta" block counts silent-new only.
//   Clean summary fires only when ALL conditions hold (silentNew=0, parities ok,
//     baseline available, both completenesses true).

import { renderMarkdown, renderJson } from "../_lib/post-write-render.mjs";

function assert(label, ok, detail = "") {
  it(label, () => {
    if (!ok) {
      throw new Error(detail ? String(detail) : `Assertion failed: ${label}`);
    }
  });
}

// ── Fixture builder ──────────────────────────────────────────

function makeEntry({
  label,
  escapeKind = "as-any",
  file = "src/a.ts",
  line = 1,
  codeShape = "x as any",
  insideExportedIdentity = null,
  occurrenceKey = "sha256:K",
  plannedEntry = null,
  diagnostics = [],
} = {}) {
  return {
    label,
    escapeKind,
    file,
    line,
    codeShape,
    normalizedCodeShape: codeShape,
    insideExportedIdentity,
    occurrenceKey,
    plannedEntry,
    diagnostics,
  };
}

function makeDelta({
  preWriteInvocationId = "pre-INV",
  deltaInvocationId = "DELTA-1",
  intentHash = "abc",
  baseline = { status: "available", source: "any-inventory.pre.pre-INV.json" },
  capabilityParity = { status: "ok" },
  scanRangeParity = { status: "ok" },
  inventoryCompleteness = {
    afterComplete: true,
    beforeComplete: true,
    filesWithParseErrors: [],
  },
  entries = [],
  summary = null,
  capabilityFailures = [],
} = {}) {
  const s = summary ?? {
    planned: entries.filter((e) => e.label === "planned").length,
    plannedNotObserved: entries.filter(
      (e) => e.label === "planned-not-observed",
    ).length,
    silentNew: entries.filter((e) => e.label === "silent-new").length,
    preExisting: entries.filter((e) => e.label === "pre-existing").length,
    removed: entries.filter((e) => e.label === "removed").length,
    observedUnbaselined: entries.filter(
      (e) => e.label === "observed-unbaselined",
    ).length,
  };
  return {
    preWriteInvocationId,
    deltaInvocationId,
    intentHash,
    baseline,
    capabilityParity,
    scanRangeParity,
    inventoryCompleteness,
    entries,
    summary: s,
    capabilityFailures,
  };
}

// ═══ Fixture A — all planned, no silent-new, clean state ═══

{
  const delta = makeDelta({
    entries: [
      makeEntry({
        label: "planned",
        escapeKind: "as-unknown-as-T",
        file: "src/vendor/wrapper.ts",
        line: 42,
        codeShape: "response as unknown as ThirdPartyShape",
        insideExportedIdentity: "src/vendor/wrapper.ts::adaptResponse",
        occurrenceKey: "sha256:FIX-A",
        plannedEntry: {
          escapeKind: "as-unknown-as-T",
          locationHint: "src/vendor/wrapper.ts::adaptResponse",
          reason: "upstream SDK lacks type exports",
        },
      }),
    ],
  });
  const md = renderMarkdown(delta);
  assert(
    "A. header present",
    md.includes("## post-write delta (canonical/any-contamination §6 Stage 2)"),
  );
  assert(
    'A. "Planned and observed:" section rendered',
    md.includes("Planned and observed:"),
  );
  assert(
    'A. "New escape sites" section OMITTED (silentNew===0)',
    !md.includes("New escape sites"),
  );
  assert(
    "A. clean summary rendered (all conditions satisfied)",
    md.includes("No silent new any in the scan range."),
  );
  assert(
    "A. planned row shows escape code",
    md.includes("response as unknown as ThirdPartyShape"),
  );
  assert(
    "A. planned row shows insideExportedIdentity",
    md.includes("src/vendor/wrapper.ts::adaptResponse"),
  );
}

// ═══ Fixture B — one silent-new, no planned ═══

{
  const delta = makeDelta({
    entries: [
      makeEntry({
        label: "silent-new",
        escapeKind: "as-any",
        file: "src/api/client.ts",
        line: 42,
        codeShape: "response as any",
        insideExportedIdentity: "src/api/client.ts::fetchUser",
        occurrenceKey: "sha256:FIX-B",
      }),
    ],
  });
  const md = renderMarkdown(delta);
  assert(
    'B. "New escape sites (silent-new — REQUIRE acknowledgment)" present',
    md.includes("New escape sites (silent-new — REQUIRE acknowledgment):"),
  );
  assert(
    "B. silent-new row cites occurrenceKey",
    md.includes("sha256:FIX-B") && md.includes("[grounded"),
  );
  assert('B. "Any delta" block shows +1 for as-any', /as any:\s+\+1/.test(md));
  assert(
    'B. "Any delta" block shows +0 for other kinds',
    /explicit any:\s+\+0/.test(md),
  );
  assert(
    "B. silent-new summary rendered",
    md.includes("silent-new — REQUIRE acknowledgment: 1"),
  );
  assert(
    'B. "No silent new any in the scan range." NOT present',
    !md.includes("No silent new any in the scan range."),
  );
}

// ═══ Fixture C — one observed-unbaselined, baseline missing ═══

{
  const delta = makeDelta({
    baseline: {
      status: "missing",
      source: null,
      reason: "advisory has no anyInventoryPath",
    },
    scanRangeParity: { status: "baseline-missing" },
    inventoryCompleteness: {
      afterComplete: true,
      beforeComplete: null,
      filesWithParseErrors: [],
    },
    entries: [
      makeEntry({
        label: "observed-unbaselined",
        escapeKind: "explicit-any",
        file: "src/legacy.ts",
        line: 12,
        codeShape: "payload: any",
        occurrenceKey: "sha256:FIX-C",
      }),
    ],
  });
  const md = renderMarkdown(delta);
  assert(
    'C. "Observed without baseline:" section present',
    md.includes("Observed without baseline:"),
  );
  assert(
    "C. row carries [확인 불가] citation (not [grounded])",
    md.includes("[확인 불가") && !md.includes("[grounded, any-inventory"),
  );
  assert(
    'C. "Any delta" shows +0 for all kinds (observed-unbaselined not counted)',
    /as any:\s+\+0/.test(md) && /explicit any:\s+\+0/.test(md),
  );
  assert(
    "C. caveated summary mentions before-inventory missing",
    md.includes("No silent-new acknowledgements required") &&
      md.includes("before-inventory missing"),
  );
  assert(
    "C. clean summary NOT present",
    !md.includes("No silent new any in the scan range."),
  );
}

// ═══ Fixture D — one planned-not-observed, silent-new=0, all ok ═══

{
  const delta = makeDelta({
    entries: [
      makeEntry({
        label: "planned-not-observed",
        escapeKind: "ts-expect-error",
        file: null,
        line: null,
        codeShape: null,
        occurrenceKey: null,
        plannedEntry: {
          escapeKind: "ts-expect-error",
          locationHint: "src/migration.ts::adapter",
          reason: "upstream type bug",
        },
      }),
    ],
  });
  const md = renderMarkdown(delta);
  assert(
    'D. "Planned but not observed:" section present',
    md.includes("Planned but not observed:"),
  );
  assert(
    "D. row mentions plannedEntry locationHint",
    md.includes("src/migration.ts::adapter"),
  );
  assert(
    'D. "Any delta" shows +0 for all kinds',
    /as any:\s+\+0/.test(md) && /ts-expect-error:\s+\+0/.test(md),
  );
  assert(
    "D. clean summary rendered (silentNew=0, all clean)",
    md.includes("No silent new any in the scan range."),
  );
}

// ═══ Fixture E — afterComplete=false, silentNew=0, parities ok ═══

{
  const delta = makeDelta({
    inventoryCompleteness: {
      afterComplete: false,
      beforeComplete: true,
      filesWithParseErrors: [
        {
          side: "after",
          file: "src/bad.ts",
          message: "Unexpected token",
          line: 12,
        },
      ],
    },
    entries: [],
  });
  const md = renderMarkdown(delta);
  assert(
    'E. "Inventory completeness:" section present',
    md.includes("Inventory completeness:"),
  );
  assert(
    "E. after-inventory parse-error file listed",
    md.includes("src/bad.ts") && md.includes("Unexpected token"),
  );
  assert(
    "E. caveated summary mentions after-inventory incomplete",
    md.includes("No silent-new acknowledgements required") &&
      md.includes("after-inventory incomplete"),
  );
  assert(
    "E. clean summary NOT rendered",
    !md.includes("No silent new any in the scan range."),
  );
}

// ═══ Fixture F — mixed all labels + scanRangeParity mismatch ═══

{
  const delta = makeDelta({
    scanRangeParity: {
      status: "mismatch",
      mismatchDetail: "includeTests: before=false after=true",
    },
    entries: [
      makeEntry({
        label: "planned",
        escapeKind: "as-any",
        file: "src/planned.ts",
        line: 10,
        codeShape: "x as any",
        occurrenceKey: "sha256:FIX-F-P",
        plannedEntry: {
          escapeKind: "as-any",
          locationHint: "src/planned.ts",
          reason: "r",
        },
      }),
      makeEntry({
        label: "silent-new",
        escapeKind: "as-any",
        file: "src/new.ts",
        line: 20,
        codeShape: "y as any",
        occurrenceKey: "sha256:FIX-F-S",
      }),
      makeEntry({
        label: "observed-unbaselined",
        escapeKind: "explicit-any",
        file: "src/obs.ts",
        line: 30,
        codeShape: "z: any",
        occurrenceKey: "sha256:FIX-F-O",
      }),
      makeEntry({
        label: "planned-not-observed",
        escapeKind: "ts-ignore",
        occurrenceKey: null,
        plannedEntry: {
          escapeKind: "ts-ignore",
          locationHint: "src/pno.ts",
          reason: "r",
        },
      }),
      makeEntry({
        label: "removed",
        escapeKind: "as-any",
        file: "src/old.ts",
        line: 5,
        codeShape: "legacy as any",
        occurrenceKey: "sha256:FIX-F-R",
      }),
    ],
  });
  const md = renderMarkdown(delta);
  assert(
    "F. all 5 occurrence sections rendered",
    md.includes("Planned and observed:") &&
      md.includes("New escape sites") &&
      md.includes("Observed without baseline:") &&
      md.includes("Planned but not observed:") &&
      md.includes("Removed:"),
  );
  assert(
    'F. "Any delta" shows +1 for as-any (silent-new kind only)',
    /as any:\s+\+1/.test(md),
  );
  assert(
    'F. "Any delta" shows +0 for explicit-any (observed-unbaselined does NOT count)',
    /explicit any:\s+\+0/.test(md),
  );
  assert(
    "F. summary is acknowledgment-required (silentNew > 0 wins)",
    md.includes("silent-new — REQUIRE acknowledgment: 1"),
  );
  assert(
    "F. scanRangeParity mismatch status line rendered",
    md.includes("Scan-range parity") && md.includes("mismatch"),
  );
}

// ═══ Pinning — section order ═══

{
  const delta = makeDelta({
    entries: [
      makeEntry({
        label: "planned",
        occurrenceKey: "k1",
        plannedEntry: { escapeKind: "as-any", locationHint: "x", reason: "r" },
      }),
      makeEntry({ label: "silent-new", occurrenceKey: "k2" }),
      makeEntry({ label: "observed-unbaselined", occurrenceKey: "k3" }),
      makeEntry({
        label: "planned-not-observed",
        plannedEntry: { escapeKind: "as-any", locationHint: "y", reason: "r" },
      }),
      makeEntry({ label: "removed", occurrenceKey: "k4" }),
    ],
  });
  const md = renderMarkdown(delta);
  const order = [
    "Any delta",
    "Planned and observed:",
    "New escape sites",
    "Observed without baseline:",
    "Planned but not observed:",
    "Removed:",
    "Inventory completeness:",
    "Baseline status:",
    "Capability parity:",
    "Scan-range parity:",
  ];
  let prev = -1;
  let monotonic = true;
  for (const marker of order) {
    const idx = md.indexOf(marker);
    if (idx < 0) {
      monotonic = false;
      break;
    }
    if (idx < prev) {
      monotonic = false;
      break;
    }
    prev = idx;
  }
  assert("ORDER. sections appear in documented order", monotonic);
}

// ═══ Pinning — ambiguous-planned-match suffix ═══

{
  const delta = makeDelta({
    entries: [
      makeEntry({
        label: "pre-existing",
        occurrenceKey: "k-amb",
        diagnostics: ["ambiguous-planned-match"],
      }),
    ],
  });
  // pre-existing is NOT rendered in per-occurrence sections, so test via silent-new label
  const delta2 = makeDelta({
    entries: [
      makeEntry({
        label: "silent-new",
        occurrenceKey: "k-amb2",
        diagnostics: ["ambiguous-planned-match"],
      }),
    ],
  });
  const md2 = renderMarkdown(delta2);
  assert(
    "AMB. silent-new entry with ambiguous-planned-match shows suffix",
    md2.includes("(ambiguous planned-match)"),
  );
}

// ═══ Pinning — capability mismatch/missing suppresses per-occurrence sections ═══

{
  const delta = makeDelta({
    capabilityParity: {
      status: "mismatch",
      mismatchDetail: "supports.typeEscapes !== true",
    },
    entries: [],
  });
  const md = renderMarkdown(delta);
  assert(
    "CAP_MIS. capability mismatch: no per-occurrence sections",
    !md.includes("Planned and observed:") &&
      !md.includes("New escape sites") &&
      !md.includes("Observed without baseline:"),
  );
  assert(
    "CAP_MIS. capability mismatch status line rendered clearly",
    md.includes("Capability parity") && md.includes("mismatch"),
  );
  assert(
    "CAP_MIS. mismatchDetail surfaced",
    md.includes("supports.typeEscapes"),
  );
}

{
  const delta = makeDelta({
    capabilityParity: {
      status: "missing",
      mismatchDetail: "afterInventory absent",
    },
    entries: [],
  });
  const md = renderMarkdown(delta);
  assert(
    "CAP_MIS2. capabilityParity: missing rendered as distinct state",
    md.includes("Capability parity") && md.includes("missing"),
  );
  assert(
    "CAP_MIS2. mentions after-inventory-missing / absent",
    md.includes("after-inventory") || md.includes("afterInventory absent"),
  );
  assert(
    "CAP_MIS2. no per-occurrence sections",
    !md.includes("Planned and observed:") && !md.includes("New escape sites"),
  );
}

// ═══ Pinning — literal guards ═══

{
  const silentDelta = makeDelta({
    entries: [makeEntry({ label: "silent-new", occurrenceKey: "k-sn" })],
  });
  const md = renderMarkdown(silentDelta);
  // "silent-new — REQUIRE acknowledgment" appears in both the section header
  // AND the summary line (for silentNew>0). Count occurrences to pin both.
  const count = (md.match(/silent-new — REQUIRE acknowledgment/g) ?? []).length;
  assert(
    'LIT. "silent-new — REQUIRE acknowledgment" appears (section header + summary)',
    count >= 2,
  );
  // But NOT anywhere unrelated (e.g. not in Observed without baseline section)
  const cleanDelta = makeDelta({
    entries: [makeEntry({ label: "observed-unbaselined", occurrenceKey: "x" })],
    baseline: { status: "missing", reason: "r" },
  });
  const cleanMd = renderMarkdown(cleanDelta);
  assert(
    'LIT. "silent-new — REQUIRE acknowledgment" absent when silentNew=0',
    !cleanMd.includes("silent-new — REQUIRE acknowledgment"),
  );
}

// ═══ renderJson = delta as-is ═══

{
  const delta = makeDelta({
    entries: [makeEntry({ label: "silent-new", occurrenceKey: "k" })],
  });
  const json = renderJson(delta);
  assert(
    "JSON. renderJson returns structurally identical delta",
    JSON.stringify(json) === JSON.stringify(delta),
  );
}
