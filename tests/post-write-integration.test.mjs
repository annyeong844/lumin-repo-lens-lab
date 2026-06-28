import { it } from "vitest";
// Release-blocking integration test for P2 — covers pre-write → edit → post-write
// end-to-end via audit-repo.mjs. Pinning rules from docs/history/phases/p2/p2-2.md v2 §5.2.
//
// Fixture 1 — multi-label: planned + silent-new + ambiguous-planned-match + incomplete-inventory.
// Fixture 2 — baseline-missing: observed-unbaselined coverage.

import { execFileSync, spawnSync } from "node:child_process";
import {
  writeFileSync,
  readFileSync,
  mkdirSync,
  mkdtempSync,
  rmSync,
  existsSync,
} from "node:fs";
import path from "node:path";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";
import { requiredAcknowledgements } from "../_lib/post-write-delta.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, "..");
const NODE = process.execPath;
const AUDIT_REPO = path.join(DIR, "audit-repo.mjs");

function assert(label, ok, detail = "") {
  it(label, () => {
    if (!ok) {
      throw new Error(detail ? String(detail) : `Assertion failed: ${label}`);
    }
  });
}

function write(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
}

function auditRepo(args, opts = {}) {
  return spawnSync(NODE, [AUDIT_REPO, ...args], { encoding: "utf8", ...opts });
}

// ═══ FIXTURE 1 — Multi-label comprehensive ═══
//
// Two distinct adapter files (DIFFERENT occurrenceKey despite identical code)
// + one unplanned file + one parse-error file.
//   src/adapters/a.ts  —  `r as any`   (planned — deterministic winner)
//   src/adapters/b.ts  —  `r as any`   (planned — ambiguous remainder)
//   src/unplanned.ts   —  `u as any`   (silent-new, clean)
//   src/bad.ts         —  broken       (parse error → afterComplete=false)

{
  const fx = mkdtempSync(path.join(tmpdir(), "p2int-fx1-"));
  const out = mkdtempSync(path.join(tmpdir(), "p2int-fx1-out-"));
  try {
    // Initial state — no escapes anywhere.
    write(fx, "package.json", JSON.stringify({ name: "fx1", type: "module" }));
    write(fx, "src/adapters/a.ts", `export const adaptA = (r) => r;\n`);
    write(fx, "src/adapters/b.ts", `export const adaptB = (r) => r;\n`);
    write(fx, "src/unplanned.ts", `export const unused = (u) => u;\n`);

    // Intent — declare one planned escape for the adapters/ directory.
    const intent = {
      names: [],
      shapes: [],
      files: [],
      dependencies: [],
      plannedTypeEscapes: [
        {
          escapeKind: "as-any",
          locationHint: "src/adapters/",
          codeShape: "r as any",
          reason: "upstream SDK lacks type exports",
        },
      ],
    };
    const intentPath = path.join(out, "intent.json");
    writeFileSync(intentPath, JSON.stringify(intent));

    // Step 1: pre-write.
    const pre = auditRepo([
      "--root",
      fx,
      "--output",
      out,
      "--profile",
      "quick",
      "--pre-write",
      "--intent",
      intentPath,
    ]);
    assert(
      "F1. pre-write step exit 0",
      pre.status === 0,
      `stderr=${pre.stderr.slice(0, 300)}`,
    );
    const preManifest = JSON.parse(
      readFileSync(path.join(out, "manifest.json"), "utf8"),
    );
    assert(
      "F1b. manifest.preWrite.ran === true",
      preManifest.preWrite?.ran === true,
    );

    // Step 2: edit — introduce escapes + parse-error file.
    write(
      fx,
      "src/adapters/a.ts",
      `export const adaptA = (r) => {\n` + `  return r as any;\n` + `};\n`,
    );
    write(
      fx,
      "src/adapters/b.ts",
      `export const adaptB = (r) => {\n` + `  return r as any;\n` + `};\n`,
    );
    write(fx, "src/unplanned.ts", `export const unused = (u) => u as any;\n`);
    write(fx, "src/bad.ts", `const x = ;;;broken\n`);

    // Step 3: post-write via audit-repo.
    const advisory = path.join(out, "pre-write-advisory.latest.json");
    const post = auditRepo([
      "--root",
      fx,
      "--output",
      out,
      "--profile",
      "quick",
      "--post-write",
      "--pre-write-advisory",
      advisory,
    ]);
    assert(
      "F1c. post-write step exit 0",
      post.status === 0,
      `stderr=${post.stderr.slice(0, 300)}`,
    );
    assert(
      'F1d. stdout contains "## post-write delta"',
      post.stdout.includes("## post-write delta"),
    );

    // Manifest summary (P0-1 pinning source).
    const m = JSON.parse(readFileSync(path.join(out, "manifest.json"), "utf8"));
    assert("F1e. manifest.postWrite.ran === true", m.postWrite?.ran === true);
    assert(
      "F1f. manifest.postWrite.deltaPath exists on disk",
      typeof m.postWrite?.deltaPath === "string" &&
        existsSync(m.postWrite.deltaPath),
    );

    // Delta JSON.
    const delta = JSON.parse(readFileSync(m.postWrite.deltaPath, "utf8"));

    // Two silent-new (one from adapters/b.ts ambiguous, one from unplanned.ts clean).
    assert(
      "F1g. manifest.postWrite.silentNew === 2",
      m.postWrite?.silentNew === 2,
    );
    assert(
      "F1h. manifest.postWrite.silentNew matches delta.summary.silentNew",
      m.postWrite?.silentNew === delta.summary.silentNew,
    );
    assert(
      "F1i. manifest.postWrite.requiredAcknowledgementCount === 2",
      m.postWrite?.requiredAcknowledgementCount === 2,
    );
    assert(
      "F1j. manifest.postWrite.afterComplete === false (bad.ts parse error)",
      m.postWrite?.afterComplete === false,
    );
    assert(
      'F1k. manifest.postWrite.baselineStatus === "available"',
      m.postWrite?.baselineStatus === "available",
    );

    // Delta preWriteInvocationId pairs with advisory.
    const adv = JSON.parse(readFileSync(advisory, "utf8"));
    assert(
      "F1l. delta.preWriteInvocationId === advisory.invocationId",
      delta.preWriteInvocationId === adv.invocationId,
    );

    // Summary counts.
    assert("F1m. summary.planned === 1", delta.summary.planned === 1);
    assert("F1n. summary.silentNew === 2", delta.summary.silentNew === 2);

    // Exactly one planned on one of the two adapters.
    const plannedEntries = delta.entries.filter((e) => e.label === "planned");
    assert(
      'F1o. exactly one "planned" entry on src/adapters/a.ts or b.ts',
      plannedEntries.length === 1 &&
        (plannedEntries[0].file === "src/adapters/a.ts" ||
          plannedEntries[0].file === "src/adapters/b.ts"),
    );

    // The OTHER adapter file is silent-new with ambiguous-planned-match diagnostic.
    const adapterEntries = delta.entries.filter((e) =>
      e.file?.startsWith("src/adapters/"),
    );
    const adapterAmbiguous = adapterEntries.filter(
      (e) =>
        e.label === "silent-new" &&
        (e.diagnostics ?? []).includes("ambiguous-planned-match"),
    );
    assert(
      "F1p. exactly one silent-new adapter has ambiguous-planned-match diagnostic",
      adapterAmbiguous.length === 1,
    );
    assert(
      "F1q. planned and ambiguous remainder are on DIFFERENT adapter files",
      plannedEntries[0].file !== adapterAmbiguous[0].file,
    );

    // unplanned.ts classifies as plain silent-new (no ambiguity).
    const unplannedEntries = delta.entries.filter(
      (e) => e.file === "src/unplanned.ts",
    );
    assert(
      "F1r. src/unplanned.ts → silent-new with empty diagnostics",
      unplannedEntries.length === 1 &&
        unplannedEntries[0].label === "silent-new" &&
        (unplannedEntries[0].diagnostics ?? []).length === 0,
    );

    // Incomplete inventory — bad.ts surfaces via inventoryCompleteness.
    assert(
      "F1s. inventoryCompleteness.afterComplete === false",
      delta.inventoryCompleteness?.afterComplete === false,
    );
    const badParseEntry = (
      delta.inventoryCompleteness?.filesWithParseErrors ?? []
    ).find((e) => e.side === "after" && /bad\.ts$/.test(e.file));
    assert(
      'F1t. filesWithParseErrors includes src/bad.ts on "after" side',
      badParseEntry !== undefined,
    );

    // requiredAcknowledgements returns exactly the 2 silent-new entries.
    const req = requiredAcknowledgements(delta);
    assert(
      "F1u. requiredAcknowledgements(delta).length === 2",
      req.length === 2,
    );
    assert(
      "F1v. every entry in requiredAcknowledgements has label === silent-new",
      req.every((e) => e.label === "silent-new"),
    );

    // Markdown stdout pinning.
    assert(
      'F1w. stdout includes "silent-new — REQUIRE acknowledgment: 2 entries"',
      post.stdout.includes("silent-new — REQUIRE acknowledgment: 2 entries"),
    );
    // When silentNew > 0 the summary is the REQUIRE-acknowledgment line;
    // incomplete inventory surfaces in the Inventory completeness section.
    assert(
      'F1x. stdout includes "Inventory completeness:" section with parse-error file listed',
      post.stdout.includes("Inventory completeness:") &&
        /src\/bad\.ts/.test(post.stdout),
    );

    // JSON round-trip.
    const roundTripped = JSON.parse(JSON.stringify(delta));
    assert(
      "F1y. JSON round-trip produces structurally equal object",
      JSON.stringify(roundTripped) === JSON.stringify(delta),
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ FIXTURE 2 — Baseline-missing (observed-unbaselined coverage) ═══
//
//   --no-fresh-audit at pre-write skips the P2-0 hook → advisory has NO
//   preWrite.anyInventoryPath → post-write degrades to observed-unbaselined.

{
  const fx = mkdtempSync(path.join(tmpdir(), "p2int-fx2-"));
  const out = mkdtempSync(path.join(tmpdir(), "p2int-fx2-out-"));
  try {
    write(fx, "package.json", JSON.stringify({ name: "fx2", type: "module" }));
    write(fx, "src/a.ts", `export const foo = (x) => x;\n`);

    const intent = {
      names: [],
      shapes: [],
      files: [],
      dependencies: [],
      plannedTypeEscapes: [],
    };
    const intentPath = path.join(out, "intent.json");
    writeFileSync(intentPath, JSON.stringify(intent));

    // Pre-write with --no-fresh-audit → hook skipped → advisory.preWrite has no anyInventoryPath.
    const pre = auditRepo([
      "--root",
      fx,
      "--output",
      out,
      "--profile",
      "quick",
      "--pre-write",
      "--intent",
      intentPath,
      "--no-fresh-audit",
    ]);
    assert(
      "F2. pre-write exit 0",
      pre.status === 0,
      `stderr=${pre.stderr.slice(0, 300)}`,
    );

    const advisory = path.join(out, "pre-write-advisory.latest.json");
    const adv = JSON.parse(readFileSync(advisory, "utf8"));
    assert(
      "F2b. advisory has no preWrite.anyInventoryPath (pre-condition)",
      !adv.preWrite || !("anyInventoryPath" in adv.preWrite),
    );

    // Edit — add one as any.
    write(fx, "src/a.ts", `export const foo = (x) => x as any;\n`);

    const post = auditRepo([
      "--root",
      fx,
      "--output",
      out,
      "--profile",
      "quick",
      "--post-write",
      "--pre-write-advisory",
      advisory,
    ]);
    assert(
      "F2c. post-write exit 0",
      post.status === 0,
      `stderr=${post.stderr.slice(0, 300)}`,
    );

    const m = JSON.parse(readFileSync(path.join(out, "manifest.json"), "utf8"));
    assert("F2d. manifest.postWrite.ran === true", m.postWrite?.ran === true);
    assert(
      'F2e. manifest.postWrite.baselineStatus === "missing"',
      m.postWrite?.baselineStatus === "missing",
    );
    assert(
      "F2f. manifest.postWrite.silentNew === 0",
      m.postWrite?.silentNew === 0,
    );
    assert(
      "F2g. manifest.postWrite.requiredAcknowledgementCount === 0",
      m.postWrite?.requiredAcknowledgementCount === 0,
    );

    const delta = JSON.parse(readFileSync(m.postWrite.deltaPath, "utf8"));
    assert(
      'F2h. delta.baseline.status === "missing"',
      delta.baseline.status === "missing",
    );
    assert(
      "F2i. delta.baseline.reason mentions anyInventoryPath",
      /anyInventoryPath/.test(delta.baseline.reason ?? ""),
    );

    const observedEntries = delta.entries.filter(
      (e) => e.label === "observed-unbaselined",
    );
    assert(
      "F2j. exactly one observed-unbaselined entry on src/a.ts",
      observedEntries.length === 1 && observedEntries[0].file === "src/a.ts",
    );
    assert("F2k. summary.silentNew === 0", delta.summary.silentNew === 0);

    assert(
      "F2l. requiredAcknowledgements returns empty array",
      requiredAcknowledgements(delta).length === 0,
    );

    assert(
      "F2m. stdout summary mentions before-inventory missing",
      post.stdout.includes("No silent-new acknowledgements required") &&
        post.stdout.includes("before-inventory missing"),
    );

    // JSON round-trip.
    const roundTripped = JSON.parse(JSON.stringify(delta));
    assert(
      "F2n. JSON round-trip structurally equal",
      JSON.stringify(roundTripped) === JSON.stringify(delta),
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}
