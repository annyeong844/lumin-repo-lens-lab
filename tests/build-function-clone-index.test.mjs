import { execFileSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, "..");
const CLI = path.join(REPO_ROOT, "build-function-clone-index.mjs");

function write(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
}

function createFixture(prefix = "vitest-fn-clone-") {
  const root = mkdtempSync(path.join(tmpdir(), prefix));
  const output = mkdtempSync(path.join(tmpdir(), `${prefix}out-`));
  return {
    root,
    output,
    cleanup() {
      rmSync(root, { recursive: true, force: true });
      rmSync(output, { recursive: true, force: true });
    },
  };
}

function runFunctionCloneIndex(root, output, extraArgs = []) {
  return execFileSync(
    process.execPath,
    [CLI, "--root", root, "--output", output, ...extraArgs],
    {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
}

function readIndex(output) {
  return JSON.parse(
    readFileSync(path.join(output, "function-clones.json"), "utf8"),
  );
}

function groupContaining(groups, identities) {
  return groups?.find((group) =>
    identities.every((identity) => group.identities?.includes(identity)),
  );
}

function nearIdfNoiseSource(count = 58) {
  return Array.from(
    { length: count },
    (_, index) =>
      `export function noiseFunction${index}() { return noiseProbeCall${index}(); }`,
  ).join("\n");
}

describe("build-function-clone-index producer artifact", () => {
  it("writes function-clones.json and surfaces same-structure distant helpers as review cues only", () => {
    const fixture = createFixture();
    try {
      write(
        fixture.root,
        "package.json",
        JSON.stringify({ name: "fn-clone-fixture", type: "module" }),
      );
      write(
        fixture.root,
        "src/money-a.ts",
        [
          "export function formatCurrencyCents(cents: number, currency = 'USD') {",
          "  const dollars = cents / 100;",
          "  return new Intl.NumberFormat('en-US', { style: 'currency', currency }).format(dollars);",
          "}",
          "",
        ].join("\n"),
      );
      write(
        fixture.root,
        "src/money-b.ts",
        [
          "export function renderPaymentTotal(value: number, unit = 'USD') {",
          "  const amount = value / 100;",
          "  return new Intl.NumberFormat('en-US', { style: 'currency', currency: unit }).format(amount);",
          "}",
          "",
        ].join("\n"),
      );

      const stdout = runFunctionCloneIndex(fixture.root, fixture.output);
      const index = readIndex(fixture.output);
      const group = groupContaining(index.structureGroups, [
        "src/money-a.ts::formatCurrencyCents",
        "src/money-b.ts::renderPaymentTotal",
      ]);

      expect(
        existsSync(path.join(fixture.output, "function-clones.json")),
      ).toBe(true);
      expect(stdout).toContain("[function-clones]");
      expect(stdout).toContain("function facts");
      expect(index.schemaVersion).toBe("function-clones.v3");
      expect(index.meta.supports?.semanticEquivalence).toBe(false);
      expect(index.meta.supports?.normalizedStructureHash).toBe(true);
      expect(group).toBeTruthy();
      expect(group.reason).toContain("not proof of semantic equivalence");
    } finally {
      fixture.cleanup();
    }
  });

  it("emits exact normalized body groups for aliased exports while parse errors mark the artifact incomplete", () => {
    const fixture = createFixture("vitest-fn-clone-exact-");
    try {
      write(
        fixture.root,
        "package.json",
        JSON.stringify({ name: "fn-clone-fixture", type: "module" }),
      );
      write(
        fixture.root,
        "src/a.ts",
        [
          "export const parseOne = (raw: string) => {",
          "  const value = raw.trim();",
          "  return value.toUpperCase();",
          "};",
          "",
        ].join("\n"),
      );
      write(
        fixture.root,
        "src/b.ts",
        [
          "const local = (raw: string) => {",
          "  const value = raw.trim();",
          "  return value.toUpperCase();",
          "};",
          "export { local as parseTwo };",
          "",
        ].join("\n"),
      );
      write(fixture.root, "src/bad.ts", "export function broken( {");

      runFunctionCloneIndex(fixture.root, fixture.output);
      const index = readIndex(fixture.output);
      const exact = groupContaining(index.exactBodyGroups, [
        "src/a.ts::parseOne",
        "src/b.ts::parseTwo",
      ]);

      expect(exact).toBeTruthy();
      expect(index.meta.complete).toBe(false);
      expect(
        index.meta.filesWithParseErrors.some(
          (entry) => entry.file === "src/bad.ts",
        ),
      ).toBe(true);
      expect(
        index.facts.some((fact) => fact.identity === "src/a.ts::parseOne"),
      ).toBe(true);
    } finally {
      fixture.cleanup();
    }
  });

  it("excludes test helpers under --production and records production scope", () => {
    const root = mkdtempSync(path.join(tmpdir(), "vitest-fn-clone-prod-"));
    const defaultOut = mkdtempSync(
      path.join(tmpdir(), "vitest-fn-clone-prod-out1-"),
    );
    const productionOut = mkdtempSync(
      path.join(tmpdir(), "vitest-fn-clone-prod-out2-"),
    );
    try {
      write(
        root,
        "package.json",
        JSON.stringify({ name: "fn-clone-fixture", type: "module" }),
      );
      write(root, "src/a.ts", "export function prod() { return 1 + 1; }\n");
      write(
        root,
        "tests/a.test.ts",
        "export function testHelper() { return 1 + 1; }\n",
      );

      runFunctionCloneIndex(root, defaultOut);
      runFunctionCloneIndex(root, productionOut, ["--production"]);
      const defaultIndex = readIndex(defaultOut);
      const productionIndex = readIndex(productionOut);

      expect(
        defaultIndex.facts.some(
          (fact) => fact.identity === "tests/a.test.ts::testHelper",
        ),
      ).toBe(true);
      expect(
        productionIndex.facts.some(
          (fact) => fact.identity === "tests/a.test.ts::testHelper",
        ),
      ).toBe(false);
      expect(productionIndex.meta.scope).toBe(
        "TS/JS production files, top-level exported and file-local functions",
      );
    } finally {
      rmSync(root, { recursive: true, force: true });
      rmSync(defaultOut, { recursive: true, force: true });
      rmSync(productionOut, { recursive: true, force: true });
    }
  });

  it("surfaces structurally different date helpers only as near review candidates", () => {
    const fixture = createFixture("vitest-fn-clone-near-");
    try {
      write(
        fixture.root,
        "package.json",
        JSON.stringify({ name: "fn-clone-near-fixture", type: "module" }),
      );
      write(
        fixture.root,
        "src/date-a.ts",
        [
          "export function formatDate(value: Date) {",
          "  const formatter = new Intl.DateTimeFormat(resolveDateLocale(), { dateStyle: 'medium' });",
          "  return formatter.format(value);",
          "}",
          "",
        ].join("\n"),
      );
      write(
        fixture.root,
        "src/date-b.ts",
        [
          "export function dateFormat(input: Date) {",
          "  return new Intl.DateTimeFormat(resolveDateLocale(), { dateStyle: 'medium' }).format(input);",
          "}",
          "",
        ].join("\n"),
      );
      write(fixture.root, "src/noise.ts", nearIdfNoiseSource());

      runFunctionCloneIndex(fixture.root, fixture.output);
      const index = readIndex(fixture.output);
      const pair = ["src/date-a.ts::formatDate", "src/date-b.ts::dateFormat"];
      const exact = groupContaining(index.exactBodyGroups, pair);
      const structure = groupContaining(index.structureGroups, pair);
      const near = groupContaining(index.nearFunctionCandidates, pair);

      expect(index.meta.supports?.nearFunctionCandidates).toBe(true);
      expect(index.meta.supports?.nearFunctionBoundedRetrieval).toBe(true);
      expect(index.meta.supports?.semanticEquivalence).toBe(false);
      expect(
        index.meta.thresholdPolicies?.some(
          (policy) =>
            policy.policyId === "function-clone-near-policy" &&
            policy.policyVersion === "function-clone-near-policy-v1" &&
            policy.policyClass === "review" &&
            policy.thresholds?.minNearScore === 0.62 &&
            policy.thresholds?.maxNearCandidates === 50 &&
            policy.thresholds?.minSingleTokenIdf === 3 &&
            policy.scoreFormulaVersion ===
              "function-clone-near-score-idf-sum-v1",
        ),
      ).toBe(true);
      expect(exact).toBeUndefined();
      expect(structure).toBeUndefined();
      expect(near).toBeTruthy();
      expect(index.meta.nearFunctionCandidateCount).toBe(1);
      expect(index.candidateGenerationPolicy).toMatchObject({
        mode: "bounded-retrieval",
        retrievalContractVersion: "function-clone-near-retrieval.v1",
      });
      expect(near).toMatchObject({
        kind: "near-function-candidate",
        risk: "review-only",
      });
      expect(near.sharedCallTokens).toContain("DateTimeFormat");
      expect(near.sharedCallTokens).toContain("resolveDateLocale");
      expect(near.sharedSignificantCallTokens).toEqual(
        expect.arrayContaining([
          expect.objectContaining({ token: "DateTimeFormat", retained: true }),
          expect.objectContaining({ token: "resolveDateLocale", retained: true }),
        ]),
      );
      expect(near.nameTokenJaccard).toBeGreaterThanOrEqual(0.5);
      expect(near.reason).toMatch(/not proof of semantic equivalence/);
      expect(near.reason).toMatch(/source review required/);
    } finally {
      fixture.cleanup();
    }
  });

  it("surfaces identical function signatures as review-only groups without body clone lanes", () => {
    const fixture = createFixture("vitest-fn-clone-signature-");
    try {
      write(
        fixture.root,
        "package.json",
        JSON.stringify({ name: "fn-clone-signature-fixture", type: "module" }),
      );
      write(
        fixture.root,
        "src/shallow.ts",
        [
          "export function useShallow<S, U>(selector: (state: S) => U): (state: S) => U {",
          "  return selector;",
          "}",
          "",
          "export function composeProjection<S, U>(selector: (state: S) => U): (state: S) => U {",
          "  return (state) => selector(state);",
          "}",
          "",
        ].join("\n"),
      );

      runFunctionCloneIndex(fixture.root, fixture.output);
      const index = readIndex(fixture.output);
      const pair = [
        "src/shallow.ts::useShallow",
        "src/shallow.ts::composeProjection",
      ];
      const exact = groupContaining(index.exactBodyGroups, pair);
      const structure = groupContaining(index.structureGroups, pair);
      const near = groupContaining(index.nearFunctionCandidates, pair);
      const signature = groupContaining(index.signatureGroups, pair);

      expect(exact).toBeUndefined();
      expect(structure).toBeUndefined();
      expect(near).toBeUndefined();
      expect(signature).toBeTruthy();
      expect(index.meta.signatureGroupCount).toBe(1);
      expect(index.meta.supports?.functionSignatureGroups).toBe(true);
      expect(signature.risk).toBe("review-only");
      expect(signature.reason).toMatch(/not proof of semantic equivalence/);
    } finally {
      fixture.cleanup();
    }
  });

  it("keeps small exact-body clones in exactBodyGroups", () => {
    const fixture = createFixture("vitest-fn-clone-small-exact-");
    try {
      write(
        fixture.root,
        "package.json",
        JSON.stringify({
          name: "fn-clone-small-exact-fixture",
          type: "module",
        }),
      );
      write(
        fixture.root,
        "src/a.ts",
        "export function answerOne() { return 42; }\n",
      );
      write(
        fixture.root,
        "src/b.ts",
        "export function answerTwo() { return 42; }\n",
      );

      runFunctionCloneIndex(fixture.root, fixture.output);
      const index = readIndex(fixture.output);
      const pair = ["src/a.ts::answerOne", "src/b.ts::answerTwo"];
      const firstHash = index.facts.find(
        (fact) => fact.identity === pair[0],
      )?.normalizedExactHash;
      const sameHashFacts = index.facts.filter(
        (fact) =>
          pair.includes(fact.identity) &&
          fact.normalizedExactHash === firstHash,
      );
      const exact = groupContaining(index.exactBodyGroups, pair);

      expect(sameHashFacts).toHaveLength(2);
      expect(exact).toBeTruthy();
    } finally {
      fixture.cleanup();
    }
  });

  it("indexes file-local top-level helper signatures for pre-write review cues", () => {
    const fixture = createFixture("vitest-fn-clone-local-helper-");
    try {
      write(
        fixture.root,
        "package.json",
        JSON.stringify({
          name: "fn-clone-local-helper-fixture",
          type: "module",
        }),
      );
      write(
        fixture.root,
        "src/user-a.ts",
        [
          "function normalizeUserName(raw: string): string {",
          "  return raw.trim().toLowerCase();",
          "}",
          "",
          "export function callA(raw: string) {",
          "  return normalizeUserName(raw);",
          "}",
          "",
        ].join("\n"),
      );
      write(
        fixture.root,
        "src/user-b.ts",
        [
          "const cleanUserName = (value: string): string => {",
          "  return value.trim().toLowerCase();",
          "};",
          "",
          "export function callB(raw: string) {",
          "  return cleanUserName(raw);",
          "}",
          "",
        ].join("\n"),
      );

      runFunctionCloneIndex(fixture.root, fixture.output);
      const index = readIndex(fixture.output);
      const localPair = [
        "src/user-a.ts::normalizeUserName",
        "src/user-b.ts::cleanUserName",
      ];
      const localFacts = index.facts.filter((fact) =>
        localPair.includes(fact.identity),
      );
      const signature = groupContaining(index.signatureGroups, localPair);

      expect(localFacts).toHaveLength(2);
      expect(localFacts).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            identity: "src/user-a.ts::normalizeUserName",
            visibility: "file-local",
            exported: false,
          }),
          expect.objectContaining({
            identity: "src/user-b.ts::cleanUserName",
            visibility: "file-local",
            exported: false,
          }),
        ]),
      );
      expect(
        localFacts.every((fact) => fact.normalizedSignatureHash),
      ).toBe(true);
      expect(
        index.facts.some(
          (fact) =>
            ["src/user-a.ts::callA", "src/user-b.ts::callB"].includes(
              fact.identity,
            ) && fact.normalizedSignatureHash,
        ),
      ).toBe(false);
      expect(signature).toBeTruthy();
      expect(signature.risk).toBe("review-only");
      expect(signature.visibilities).toContain("file-local");
      expect(index.meta.supports?.fileLocalTopLevelFunctions).toBe(true);
      expect(index.meta.scope).toContain("file-local");
    } finally {
      fixture.cleanup();
    }
  });

  it("preserves identifier-backed default exports as exported default facts", () => {
    const fixture = createFixture("vitest-fn-clone-default-alias-");
    try {
      write(
        fixture.root,
        "package.json",
        JSON.stringify({
          name: "fn-clone-default-alias-fixture",
          type: "module",
        }),
      );
      write(
        fixture.root,
        "src/default-fn.ts",
        [
          "function normalizePayload(raw: string): string {",
          "  return raw.trim().toLowerCase();",
          "}",
          "",
          "export default normalizePayload;",
          "",
        ].join("\n"),
      );
      write(
        fixture.root,
        "src/default-const.ts",
        [
          "const serializePayload = (raw: string): string => {",
          "  return raw.trim().toLowerCase();",
          "};",
          "",
          "export default serializePayload;",
          "",
        ].join("\n"),
      );

      runFunctionCloneIndex(fixture.root, fixture.output);
      const index = readIndex(fixture.output);
      const defaultFn = index.facts.find(
        (fact) => fact.identity === "src/default-fn.ts::default",
      );
      const defaultConst = index.facts.find(
        (fact) => fact.identity === "src/default-const.ts::default",
      );

      expect(defaultFn).toMatchObject({
        visibility: "exported",
        exported: true,
        exportedName: "default",
        localName: "normalizePayload",
      });
      expect(defaultFn.normalizedSignatureHash).toBeTruthy();
      expect(defaultConst).toMatchObject({
        visibility: "exported",
        exported: true,
        exportedName: "default",
        localName: "serializePayload",
      });
      expect(defaultConst.normalizedSignatureHash).toBeTruthy();
      expect(
        index.facts.some(
          (fact) =>
            fact.identity === "src/default-fn.ts::normalizePayload" ||
            fact.identity === "src/default-const.ts::serializePayload",
        ),
      ).toBe(false);
    } finally {
      fixture.cleanup();
    }
  });

  it("creates the output directory when the standalone producer writes function-clones.json", () => {
    const root = mkdtempSync(path.join(tmpdir(), "vitest-fn-clone-missing-output-"));
    const parent = mkdtempSync(
      path.join(tmpdir(), "vitest-fn-clone-missing-output-parent-"),
    );
    const output = path.join(parent, "nested", "audit-artifacts");
    try {
      write(
        root,
        "package.json",
        JSON.stringify({
          name: "fn-clone-missing-output-fixture",
          type: "module",
        }),
      );
      write(root, "src/a.ts", "export function ready(): boolean { return true; }\n");

      runFunctionCloneIndex(root, output, ["--no-incremental"]);
      const index = readIndex(output);

      expect(existsSync(path.join(output, "function-clones.json"))).toBe(true);
      expect(
        index.facts.some((fact) => fact.identity === "src/a.ts::ready"),
      ).toBe(true);
    } finally {
      rmSync(root, { recursive: true, force: true });
      rmSync(parent, { recursive: true, force: true });
    }
  });
});
