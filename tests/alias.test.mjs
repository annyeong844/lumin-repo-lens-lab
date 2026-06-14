// Mirrors tests/test-alias.mjs.
//
// This suite protects aliased export-specifier action safety:
// `export { foo as publicThing }` must preserve the distinction between the
// local implementation name (`foo`) and the public exported name
// (`publicThing`) before dead-export actions are proposed.

import { execFileSync } from "node:child_process";
import {
  mkdtempSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { afterAll, beforeAll, it } from "vitest";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, "..");

let root;
let fixtureRoot;
let outputRoot;
let symbols;
let classify;

function runProducer(script, args) {
  execFileSync(process.execPath, [script, ...args], {
    cwd: REPO_ROOT,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
}

function writeFixtureFile(relPath, text) {
  const target = path.join(fixtureRoot, relPath);
  mkdirSync(path.dirname(target), { recursive: true });
  writeFileSync(target, text);
}

function findAliasedProposal(file, symbol) {
  const allProposals = [
    ...(classify.proposal_C_remove_symbol || []),
    ...(classify.proposal_A_demote_to_internal || []),
    ...(classify.proposal_remove_export_specifier || []),
  ];
  return allProposals.find(
    (entry) => entry.file === file && entry.symbol === symbol,
  );
}

beforeAll(() => {
  root = mkdtempSync(path.join(os.tmpdir(), "alias-vitest-"));
  fixtureRoot = path.join(root, "fixture");
  outputRoot = path.join(root, "out");
  mkdirSync(path.join(fixtureRoot, "src"), { recursive: true });

  writeFixtureFile("package.json", '{"name":"alias-fx","type":"module"}');
  writeFixtureFile(
    "src/aliased_local_used.ts",
    [
      "function foo() { return 42; }",
      "export function localConsumer() { return foo() + 1; }",
      "export { foo as publicThing };",
      "",
    ].join("\n"),
  );
  writeFixtureFile(
    "src/aliased_local_dead.ts",
    [
      "function bar() { return 42; }",
      "export { bar as publicThing };",
      "",
    ].join("\n"),
  );
  writeFixtureFile(
    "src/non_aliased.ts",
    ["function helper() { return 1; }", "export { helper };", ""].join("\n"),
  );
  writeFixtureFile(
    "src/consumer.ts",
    [
      "import { localConsumer } from './aliased_local_used';",
      "export const _keepAlive = localConsumer;",
      "",
    ].join("\n"),
  );

  runProducer("build-symbol-graph.mjs", [
    "--root",
    fixtureRoot,
    "--output",
    outputRoot,
  ]);
  runProducer("classify-dead-exports.mjs", [
    "--root",
    fixtureRoot,
    "--output",
    outputRoot,
  ]);

  symbols = JSON.parse(
    readFileSync(path.join(outputRoot, "symbols.json"), "utf8"),
  );
  classify = JSON.parse(
    readFileSync(path.join(outputRoot, "dead-classify.json"), "utf8"),
  );
});

afterAll(() => {
  if (root) rmSync(root, { recursive: true, force: true });
});

it("records localName for aliased export specifiers", () => {
  const aliased = symbols.deadProdList.find(
    (entry) =>
      entry.file === "src/aliased_local_used.ts" &&
      entry.symbol === "publicThing",
  );

  if (!aliased || aliased.localName !== "foo") {
    throw new Error(`got: ${JSON.stringify(aliased)}`);
  }
});

it("does not add a distinct localName for non-aliased export specifiers", () => {
  const nonAliased = symbols.deadProdList.find(
    (entry) => entry.file === "src/non_aliased.ts" && entry.symbol === "helper",
  );

  if (
    !nonAliased ||
    (nonAliased.localName && nonAliased.localName !== "helper")
  ) {
    throw new Error(`got: ${JSON.stringify(nonAliased)}`);
  }
});

it("routes locally used aliased exports away from definition-removal wording", () => {
  const cProposal = classify.proposal_C_remove_symbol || [];
  const aliasedDead = cProposal.find(
    (entry) =>
      entry.file === "src/aliased_local_used.ts" &&
      entry.symbol === "publicThing",
  );

  if (!aliasedDead) {
    const inA = (classify.proposal_A_demote_to_internal || []).find(
      (entry) =>
        entry.file === "src/aliased_local_used.ts" &&
        entry.symbol === "publicThing",
    );
    const inSpec = (classify.proposal_remove_export_specifier || []).find(
      (entry) =>
        entry.file === "src/aliased_local_used.ts" &&
        entry.symbol === "publicThing",
    );

    if (!inSpec && !inA) {
      throw new Error(
        `not found in A or specifier bucket. classify keys: ${Object.keys(
          classify,
        )}`,
      );
    }
    return;
  }

  if (/정의 자체 제거/.test(aliasedDead.action)) {
    throw new Error(`got action: "${aliasedDead.action}"`);
  }
});

it("carries localName on aliased export proposals", () => {
  const aliased = findAliasedProposal(
    "src/aliased_local_used.ts",
    "publicThing",
  );

  if (!aliased || aliased.localName !== "foo") {
    throw new Error(`got: ${JSON.stringify(aliased)}`);
  }
});

it("signals when the local implementation is also dead", () => {
  const aliasedBothDead = findAliasedProposal(
    "src/aliased_local_dead.ts",
    "publicThing",
  );

  if (
    !aliasedBothDead ||
    !(
      aliasedBothDead.localAlsoDead === true ||
      aliasedBothDead.localInternalUses === 0
    )
  ) {
    throw new Error(`got: ${JSON.stringify(aliasedBothDead)}`);
  }
});

it("counts internal references through the local name, not the exported alias", () => {
  const aliased = findAliasedProposal(
    "src/aliased_local_used.ts",
    "publicThing",
  );
  const localUses =
    aliased?.localInternalUses ?? aliased?.fileInternalUses ?? 0;

  if (!aliased || localUses <= 0) {
    throw new Error(`got: ${JSON.stringify(aliased)}`);
  }
});
