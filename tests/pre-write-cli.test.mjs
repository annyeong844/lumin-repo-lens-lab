import { execFileSync, spawnSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";

const ROOT = path.resolve(import.meta.dirname, "..");
const NODE = process.execPath;
const PREWRITE = path.join(ROOT, "pre-write.mjs");
const CLI_TEST_TIMEOUT_MS = 30_000;

function itCli(name, fn, timeout = CLI_TEST_TIMEOUT_MS) {
  return it(name, fn, timeout);
}

function write(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
}

function buildFixture(root) {
  write(
    root,
    "package.json",
    JSON.stringify({ name: "pw-fx", type: "module" }),
  );
  write(root, "src/a.ts", "export const formatDate = (d) => d.toString();\n");
  write(
    root,
    "src/b.ts",
    "import { formatDate } from './a';\nexport const useFmt = () => formatDate(new Date());\n",
  );
}

function runBuildSymbols(root, output) {
  execFileSync(
    NODE,
    [
      path.join(ROOT, "build-symbol-graph.mjs"),
      "--root",
      root,
      "--output",
      output,
    ],
    { stdio: ["ignore", "pipe", "pipe"] },
  );
}

function runPreWrite(root, output, intentPath, extraArgs = []) {
  return execFileSync(
    NODE,
    [
      PREWRITE,
      "--root",
      root,
      "--output",
      output,
      "--intent",
      intentPath,
      ...extraArgs,
    ],
    { stdio: ["ignore", "pipe", "pipe"], encoding: "utf8" },
  );
}

function withFixture(prefix, fn) {
  const root = mkdtempSync(path.join(tmpdir(), `${prefix}-`));
  const output = mkdtempSync(path.join(tmpdir(), `${prefix}-out-`));
  try {
    buildFixture(root);
    return fn({ root, output });
  } finally {
    rmSync(root, { recursive: true, force: true });
    rmSync(output, { recursive: true, force: true });
  }
}

function writeIntent(output, intent) {
  const intentPath = path.join(output, "intent.json");
  writeFileSync(intentPath, JSON.stringify(intent));
  return intentPath;
}

function readLatest(output) {
  return JSON.parse(
    readFileSync(path.join(output, "pre-write-advisory.latest.json"), "utf8"),
  );
}

function invocationFiles(output) {
  return readdirSync(output).filter(
    (name) =>
      name.startsWith("pre-write-advisory.") && !name.endsWith(".latest.json"),
  );
}

describe("direct pre-write CLI advisory lifecycle", () => {
  itCli(
    "runs the happy path, writes dual advisory files, and prints invocation handoff",
    () => {
      withFixture("pw-cli-happy", ({ root, output }) => {
        runBuildSymbols(root, output);
        const intentPath = writeIntent(output, {
          names: ["formatDate"],
          shapes: [],
          files: [],
          dependencies: [],
          plannedTypeEscapes: [],
        });

        const stdout = runPreWrite(root, output, intentPath);
        expect(stdout).toContain("pre-write advisory");
        expect(stdout).toContain("### Grounded facts");
        expect(stdout).toContain("formatDate");

        const latestPath = path.join(output, "pre-write-advisory.latest.json");
        expect(existsSync(latestPath)).toBe(true);

        const invFiles = invocationFiles(output);
        expect(invFiles).toHaveLength(1);
        const latestText = readFileSync(latestPath, "utf8");
        expect(readFileSync(path.join(output, invFiles[0]), "utf8")).toBe(
          latestText,
        );

        const advisory = JSON.parse(latestText);
        expect(advisory.intentHash).toMatch(/^[a-f0-9]{64}$/);
        expect(advisory.capabilities.identityFanIn).toBe(true);
        expect(path.basename(advisory.artifactPaths.invocationSpecific)).toBe(
          invFiles[0],
        );
        expect(path.basename(advisory.artifactPaths.latest)).toBe(
          "pre-write-advisory.latest.json",
        );
        expect(stdout).toContain("--pre-write-advisory");
        expect(stdout).toContain(invFiles[0]);
        expect(stdout).not.toContain(
          "--pre-write-advisory pre-write-advisory.latest.json",
        );
      });
    },
  );

  itCli(
    "normalizes rich and compact intents without dropping warnings or metadata",
    () => {
      withFixture("pw-cli-rich-intent", ({ root, output }) => {
        runBuildSymbols(root, output);
        const intentPath = writeIntent(output, {
          names: [
            {
              name: "formatTimestamp",
              kind: "function",
              why: "new display helper",
            },
          ],
          shapes: [
            {
              name: "TimestampViewModel",
              typeLiteral: "{ label: string; iso: string; timezone: string }",
              why: "view model contract",
            },
          ],
          files: ["src/features/time/format-timestamp.ts"],
          dependencies: [
            { specifier: "date-fns", why: "timestamp formatting" },
          ],
          plannedTypeEscapes: [],
        });
        runPreWrite(root, output, intentPath);

        const advisory = readLatest(output);
        expect(advisory.intent.names).toContain("formatTimestamp");
        expect(advisory.intent.dependencies).toContain("date-fns");
        expect(advisory.intent.shapes[0].typeLiteral).toContain("timezone");
        expect(advisory.intent.shapes[0].fields).toEqual([]);
        expect(advisory.intent.nameDeclarations[0].why).toBe(
          "new display helper",
        );
        expect(advisory.intent.dependencyDeclarations[0].why).toBe(
          "timestamp formatting",
        );
        expect(advisory.intent.shapes[0].why).toBe("view model contract");
      });

      withFixture("pw-cli-compact-intent", ({ root, output }) => {
        const intentPath = writeIntent(output, {
          files: ["src/new-helper.ts"],
        });
        const stdout = runPreWrite(root, output, intentPath);
        expect(stdout).not.toContain("Intent schema notes");
        expect(stdout).not.toContain("Missing top-level intent keys defaulted");

        const advisory = readLatest(output);
        expect(advisory.intent.names).toEqual([]);
        expect(advisory.intent.shapes).toEqual([]);
        expect(advisory.intent.files).toContain("src/new-helper.ts");
        expect(advisory.intent.dependencies).toEqual([]);
        expect(advisory.intent.plannedTypeEscapes).toEqual([]);
        expect(advisory.intentWarnings).toHaveLength(4);
        expect(
          advisory.intentWarnings.every(
            (warning) => warning.kind === "missing-intent-key-defaulted",
          ),
        ).toBe(true);
      });
    },
    CLI_TEST_TIMEOUT_MS,
  );

  itCli("handles paths with spaces and shell metacharacters", () => {
    const parent = mkdtempSync(path.join(tmpdir(), "pw-cli-space-"));
    const root = path.join(parent, "my fixture");
    const output = path.join(parent, "my output");
    mkdirSync(root, { recursive: true });
    mkdirSync(output, { recursive: true });
    try {
      buildFixture(root);
      runBuildSymbols(root, output);
      const intentPath = writeIntent(output, {
        names: ["formatDate"],
        shapes: [],
        files: [],
        dependencies: [],
        plannedTypeEscapes: [],
      });
      const stdout = runPreWrite(root, output, intentPath);
      expect(stdout).toContain("### Grounded facts");
      expect(stdout).toContain("formatDate");
      expect(
        existsSync(path.join(output, "pre-write-advisory.latest.json")),
      ).toBe(true);
    } finally {
      rmSync(parent, { recursive: true, force: true });
    }

    const shellParent = mkdtempSync(path.join(tmpdir(), "pw-coldcache-shell-"));
    const shellRoot = path.join(shellParent, "my $fixture");
    const shellOutput = path.join(shellParent, "my $output");
    mkdirSync(shellRoot, { recursive: true });
    mkdirSync(shellOutput, { recursive: true });
    try {
      buildFixture(shellRoot);
      const intentPath = writeIntent(shellOutput, {
        names: ["formatDate"],
        shapes: [],
        files: [],
        dependencies: [],
        plannedTypeEscapes: [],
      });
      const stdout = runPreWrite(shellRoot, shellOutput, intentPath);
      expect(stdout).toContain("### Grounded facts");
      expect(existsSync(path.join(shellOutput, "symbols.json"))).toBe(true);
    } finally {
      rmSync(shellParent, { recursive: true, force: true });
    }
  });

  itCli(
    "degrades missing symbols with no-fresh-audit without claiming grounded absence",
    () => {
      withFixture("pw-cli-nosym", ({ root, output }) => {
        const intentPath = writeIntent(output, {
          names: ["anything"],
          shapes: [],
          files: [],
          dependencies: [],
          plannedTypeEscapes: [],
        });
        const stdout = runPreWrite(root, output, intentPath, [
          "--no-fresh-audit",
        ]);
        expect(stdout).toContain("확인 불가");
        expect(stdout).toContain("Evidence availability");
        expect(stdout).toContain("symbols.json");
        expect(stdout).toContain("same `--output`");
        expect(stdout).toContain("not grounded absence");

        const advisory = readLatest(output);
        expect(advisory.failures).toEqual(
          expect.arrayContaining([
            expect.objectContaining({ kind: "symbols-missing" }),
          ]),
        );
        expect(advisory.evidenceAvailability).toMatchObject({
          status: "missing",
          freshAudit: false,
        });
        expect(advisory.evidenceAvailability.artifacts).toEqual(
          expect.arrayContaining([
            expect.objectContaining({
              artifact: "symbols.json",
              status: "missing",
            }),
          ]),
        );
      });
    },
  );

  itCli(
    "keeps dependency import confidence unavailable without symbols and grounded with fresh symbols",
    () => {
      withFixture("pw-cli-nodepsym", ({ root, output }) => {
        write(
          root,
          "package.json",
          JSON.stringify({
            name: "nodepsym",
            dependencies: { dayjs: "1.0.0" },
          }),
        );
        const intentPath = writeIntent(output, {
          names: [],
          shapes: [],
          files: [],
          dependencies: ["dayjs"],
          plannedTypeEscapes: [],
        });
        const stdout = runPreWrite(root, output, intentPath, [
          "--no-fresh-audit",
        ]);
        expect(stdout).toContain(
          "DEPENDENCY_AVAILABLE_IMPORT_GRAPH_UNAVAILABLE",
        );
        expect(stdout).toContain("import graph unavailable");
        expect(stdout).not.toContain("0 observed consumer");

        const depLookup = readLatest(output).lookups.find(
          (lookup) => lookup.kind === "dependency",
        );
        expect(depLookup.existingImports).toMatchObject({
          observedImportCount: null,
          countConfidence: "unavailable",
        });
      });

      withFixture("pw-cli-dep-consumer", ({ root, output }) => {
        write(
          root,
          "package.json",
          JSON.stringify({
            name: "dep-consumer",
            type: "module",
            dependencies: { dayjs: "1.0.0" },
          }),
        );
        write(
          root,
          "src/use.ts",
          "import dayjs from 'dayjs';\nexport const today = () => dayjs().format('YYYY-MM-DD');\n",
        );
        const intentPath = writeIntent(output, {
          names: [],
          shapes: [],
          files: [],
          dependencies: ["dayjs"],
          plannedTypeEscapes: [],
        });
        const stdout = runPreWrite(root, output, intentPath);
        expect(stdout).toContain("DEPENDENCY_AVAILABLE");
        expect(stdout).not.toContain(
          "DEPENDENCY_AVAILABLE_IMPORT_GRAPH_UNAVAILABLE",
        );

        const symbols = JSON.parse(
          readFileSync(path.join(output, "symbols.json"), "utf8"),
        );
        expect(symbols.meta.supports.dependencyImportConsumers).toBe(true);
        expect(symbols.dependencyImportConsumers).toEqual(
          expect.arrayContaining([
            expect.objectContaining({
              file: "src/use.ts",
              fromSpec: "dayjs",
              depRoot: "dayjs",
            }),
          ]),
        );

        const depLookup = readLatest(output).lookups.find(
          (lookup) => lookup.kind === "dependency",
        );
        expect(depLookup).toMatchObject({
          result: "DEPENDENCY_AVAILABLE",
          existingImports: {
            observedImportCount: 1,
            countConfidence: "grounded",
          },
        });
        expect(depLookup.citations).toEqual(
          expect.arrayContaining([
            expect.stringMatching(/symbols\.json\.dependencyImportConsumers/),
          ]),
        );
      });
    },
  );

  itCli(
    "rejects missing, malformed, and schema-invalid intent inputs with non-zero exits",
    () => {
      withFixture("pw-cli-bad", ({ root, output }) => {
        const missing = spawnSync(
          NODE,
          [PREWRITE, "--root", root, "--output", output],
          { stdio: ["ignore", "pipe", "pipe"], encoding: "utf8" },
        );
        expect(missing.status).not.toBe(0);
        expect(`${missing.stderr}${missing.stdout}`).toMatch(/intent/i);

        const badIntentPath = path.join(output, "bad-intent.json");
        writeFileSync(badIntentPath, "{ this is not valid json");
        const malformed = spawnSync(
          NODE,
          [
            PREWRITE,
            "--root",
            root,
            "--output",
            output,
            "--intent",
            badIntentPath,
          ],
          { stdio: ["ignore", "pipe", "pipe"], encoding: "utf8" },
        );
        expect(malformed.status).not.toBe(0);
        expect(malformed.stderr).toMatch(/parse|JSON/i);

        const schemaBadPath = path.join(output, "schema-bad.json");
        writeFileSync(
          schemaBadPath,
          JSON.stringify({
            names: "formatDate",
            shapes: [],
            files: [],
            dependencies: [],
            plannedTypeEscapes: [],
          }),
        );
        const schemaBad = spawnSync(
          NODE,
          [
            PREWRITE,
            "--root",
            root,
            "--output",
            output,
            "--intent",
            schemaBadPath,
          ],
          { stdio: ["ignore", "pipe", "pipe"], encoding: "utf8" },
        );
        expect(schemaBad.status).not.toBe(0);
        expect(schemaBad.stderr).toContain("names");
      });
    },
  );

  itCli(
    "renders all rich lookup sections and preserves lookup ordering",
    () => {
      withFixture("pw-cli-p12", ({ root, output }) => {
        runBuildSymbols(root, output);
        const intentPath = writeIntent(output, {
          names: ["formatDate"],
          shapes: [{ fields: ["year", "month"] }],
          files: ["src/utils/time.ts"],
          dependencies: ["dayjs"],
          plannedTypeEscapes: [],
        });
        const stdout = runPreWrite(root, output, intentPath);
        expect(stdout).toContain("### Grounded facts");
        expect(stdout).toContain("### New code candidates");
        expect(stdout).toContain("### Unavailable evidence");
        expect(stdout).toContain("NEW_FILE");
        expect(stdout).toContain("src/utils/time.ts");
        expect(stdout).toContain("NEW_PACKAGE");
        expect(stdout).toContain("dayjs");
        expect(stdout).toContain("shape-hash");
        expect(stdout).toContain("P4");

        const kinds = readLatest(output).lookups.map((lookup) => lookup.kind);
        expect(new Set(kinds)).toEqual(
          new Set(["name", "file", "dependency", "shape"]),
        );
        expect(kinds[0]).toBe("name");
      });
    },
  );

  itCli("stamps preWrite.anyInventoryPath on both advisory files", () => {
    withFixture("pw-p20", ({ root, output }) => {
      const intentPath = writeIntent(output, {
        names: ["formatDate"],
        shapes: [],
        files: [],
        dependencies: [],
        plannedTypeEscapes: [],
      });
      runPreWrite(root, output, intentPath);
      const latest = readLatest(output);
      const invFiles = invocationFiles(output);
      expect(invFiles).toHaveLength(1);
      const invocation = JSON.parse(
        readFileSync(path.join(output, invFiles[0]), "utf8"),
      );
      expect(latest.preWrite.anyInventoryPath).toBeTruthy();
      expect(invocation.preWrite.anyInventoryPath).toBeTruthy();
      expect(invocation.preWrite.anyInventoryPath).toBe(
        latest.preWrite.anyInventoryPath,
      );
    });
  });

  itCli("handles missing triage without boundary overclaiming", () => {
    withFixture("pw-cli-p12-notriage", ({ root, output }) => {
      runBuildSymbols(root, output);
      execFileSync(
        NODE,
        [
          path.join(ROOT, "measure-topology.mjs"),
          "--root",
          root,
          "--output",
          output,
        ],
        { stdio: ["ignore", "pipe", "pipe"] },
      );
      const intentPath = writeIntent(output, {
        names: [],
        shapes: [],
        files: ["src/utils/time.ts"],
        dependencies: [],
        plannedTypeEscapes: [],
      });
      const stdout = runPreWrite(root, output, intentPath);
      expect(stdout).toMatch(/NEW_FILE|FILE_STATUS_UNKNOWN/);
      expect(stdout).toMatch(/not.evaluated/i);
      expect(stdout).not.toMatch(/boundary.*ALLOWED/i);
      expect(stdout).not.toMatch(/boundary.*FORBIDDEN/i);
    });
  });

  itCli(
    "runs targeted cold-cache producers and avoids unrelated artifacts",
    () => {
      withFixture("pw-coldcache-full", ({ root, output }) => {
        const intentPath = writeIntent(output, {
          names: ["formatDate"],
          shapes: [],
          files: [],
          dependencies: [],
          plannedTypeEscapes: [],
        });
        const stdout = runPreWrite(root, output, intentPath);
        expect(stdout).toContain("### Grounded facts");
        expect(stdout).toContain("formatDate");
        expect(existsSync(path.join(output, "symbols.json"))).toBe(true);
        expect(existsSync(path.join(output, "topology.json"))).toBe(false);
        expect(existsSync(path.join(output, "triage.json"))).toBe(false);
        expect(existsSync(path.join(output, "shape-index.json"))).toBe(false);
      });

      const shapeCases = [
        {
          prefix: "pw-coldcache-shape",
          intent: {
            names: [],
            shapes: [{ fields: [], typeLiteral: "{ year: number }" }],
            files: [],
            dependencies: [],
            plannedTypeEscapes: [],
          },
          source: "export type CalendarShape = { year: number };\n",
          expected: [
            "same normalized type shape",
            "src/types.ts::CalendarShape",
          ],
        },
        {
          prefix: "pw-coldcache-shape-fields",
          intent: {
            names: [],
            shapes: [{ fields: ["info", "warn", "error", "withContext"] }],
            files: [],
            dependencies: [],
            plannedTypeEscapes: [],
          },
          source:
            "export type LoggerShape = { info: string; warn: string; error: string; withContext: string };\n",
          expected: [
            "UNAVAILABLE",
            "field names alone are not structural equality evidence",
          ],
        },
      ];
      for (const { prefix, intent, source, expected } of shapeCases) {
        const root = mkdtempSync(path.join(tmpdir(), `${prefix}-`));
        const output = mkdtempSync(path.join(tmpdir(), `${prefix}-out-`));
        try {
          write(
            root,
            "package.json",
            JSON.stringify({ name: prefix, type: "module" }),
          );
          write(root, "src/types.ts", source);
          const stdout = runPreWrite(root, output, writeIntent(output, intent));
          expect(existsSync(path.join(output, "shape-index.json"))).toBe(true);
          expect(existsSync(path.join(output, "symbols.json"))).toBe(false);
          expect(existsSync(path.join(output, "topology.json"))).toBe(false);
          expect(existsSync(path.join(output, "triage.json"))).toBe(false);
          for (const text of expected) expect(stdout).toContain(text);
        } finally {
          rmSync(root, { recursive: true, force: true });
          rmSync(output, { recursive: true, force: true });
        }
      }

      const fnRoot = mkdtempSync(path.join(tmpdir(), "pw-coldcache-function-"));
      const fnOutput = mkdtempSync(
        path.join(tmpdir(), "pw-coldcache-function-out-"),
      );
      try {
        write(
          fnRoot,
          "package.json",
          JSON.stringify({ name: "pw-function-signature-fx", type: "module" }),
        );
        write(
          fnRoot,
          "src/shallow.ts",
          "export function useShallow<S, U>(selector: (state: S) => U): (state: S) => U {\n" +
            "  return selector;\n" +
            "}\n",
        );
        const stdout = runPreWrite(
          fnRoot,
          fnOutput,
          writeIntent(fnOutput, {
            names: ["composeProjection"],
            shapes: [
              {
                fields: [],
                typeLiteral:
                  "<S, U>(selector: (state: S) => U) => (state: S) => U",
              },
            ],
            files: [],
            dependencies: [],
            plannedTypeEscapes: [],
          }),
        );
        expect(existsSync(path.join(fnOutput, "function-clones.json"))).toBe(
          true,
        );
        expect(stdout).toContain("same normalized function signature");
        expect(stdout).toContain("src/shallow.ts::useShallow");
      } finally {
        rmSync(fnRoot, { recursive: true, force: true });
        rmSync(fnOutput, { recursive: true, force: true });
      }
    },
    CLI_TEST_TIMEOUT_MS,
  );

  itCli("uses partial cold-cache only for missing topology", () => {
    withFixture("pw-coldcache-partial", ({ root, output }) => {
      runBuildSymbols(root, output);
      execFileSync(
        NODE,
        [
          path.join(ROOT, "triage-repo.mjs"),
          "--root",
          root,
          "--output",
          output,
        ],
        { stdio: ["ignore", "pipe", "pipe"] },
      );
      const symbolsSize = readFileSync(
        path.join(output, "symbols.json"),
      ).length;
      const triageSize = readFileSync(path.join(output, "triage.json")).length;
      expect(existsSync(path.join(output, "topology.json"))).toBe(false);

      runPreWrite(
        root,
        output,
        writeIntent(output, {
          names: [],
          shapes: [],
          files: ["src/a.ts"],
          dependencies: [],
          plannedTypeEscapes: [],
        }),
      );

      expect(existsSync(path.join(output, "topology.json"))).toBe(true);
      expect(readFileSync(path.join(output, "symbols.json")).length).toBe(
        symbolsSize,
      );
      expect(readFileSync(path.join(output, "triage.json")).length).toBe(
        triageSize,
      );
    });
  });

  itCli(
    "keeps no-fresh-audit, producer failure, and timeout as explicit failures",
    () => {
      withFixture("pw-coldcache-nofresh", ({ root, output }) => {
        const stdout = runPreWrite(
          root,
          output,
          writeIntent(output, {
            names: ["anything"],
            shapes: [],
            files: [],
            dependencies: [],
            plannedTypeEscapes: [],
          }),
          ["--no-fresh-audit"],
        );
        expect(existsSync(path.join(output, "symbols.json"))).toBe(false);
        expect(existsSync(path.join(output, "topology.json"))).toBe(false);
        expect(existsSync(path.join(output, "triage.json"))).toBe(false);
        expect(stdout).toContain("확인 불가");
        expect(readLatest(output).failures).toEqual(
          expect.arrayContaining([
            expect.objectContaining({
              kind: expect.stringMatching(/-missing$/),
            }),
          ]),
        );
      });

      const failRoot = mkdtempSync(path.join(tmpdir(), "pw-coldcache-fail-"));
      const failOutput = mkdtempSync(
        path.join(tmpdir(), "pw-coldcache-fail-out-"),
      );
      try {
        write(failRoot, "package.json", "{ not valid json ");
        const intentPath = writeIntent(failOutput, {
          names: ["x"],
          shapes: [],
          files: [],
          dependencies: [],
          plannedTypeEscapes: [],
        });
        const result = spawnSync(
          NODE,
          [
            PREWRITE,
            "--root",
            failRoot,
            "--output",
            failOutput,
            "--intent",
            intentPath,
          ],
          { stdio: ["ignore", "pipe", "pipe"], encoding: "utf8" },
        );
        expect(
          result.status === 0 ||
            result.stdout.length + result.stderr.length > 0,
        ).toBe(true);
        if (result.status === 0) {
          expect(readLatest(failOutput).failures).toEqual(
            expect.arrayContaining([
              expect.objectContaining({
                kind: expect.stringMatching(/cold-cache|parse-error|missing/),
              }),
            ]),
          );
        }
      } finally {
        rmSync(failRoot, { recursive: true, force: true });
        rmSync(failOutput, { recursive: true, force: true });
      }

      withFixture("pw-coldcache-timeout", ({ root, output }) => {
        const result = execFileSync(
          NODE,
          [
            PREWRITE,
            "--root",
            root,
            "--output",
            output,
            "--intent",
            writeIntent(output, {
              names: ["formatDate"],
              shapes: [],
              files: [],
              dependencies: [],
              plannedTypeEscapes: [],
            }),
          ],
          {
            stdio: ["ignore", "pipe", "pipe"],
            env: { ...process.env, PRE_WRITE_COLD_CACHE_TIMEOUT_MS: "1" },
          },
        );
        expect(result.length).toBeGreaterThan(0);
        expect(readLatest(output).failures).toEqual(
          expect.arrayContaining([
            expect.objectContaining({ kind: expect.stringMatching(/timeout/) }),
          ]),
        );
      });
    },
  );

  itCli(
    "keeps stdout advisory output separate from cold-cache diagnostics",
    () => {
      withFixture("pw-coldcache-std", ({ root, output }) => {
        const child = spawnSync(
          NODE,
          [
            PREWRITE,
            "--root",
            root,
            "--output",
            output,
            "--intent",
            writeIntent(output, {
              names: ["formatDate"],
              shapes: [],
              files: [],
              dependencies: [],
              plannedTypeEscapes: [],
            }),
          ],
          { encoding: "utf8" },
        );

        expect(child.stdout).toContain("## pre-write advisory");
        expect(child.stderr).not.toContain("## pre-write advisory");
        expect(child.stderr).toMatch(/\[pre-write\] cold-cache/);
        expect(child.stdout).not.toMatch(/\[pre-write\] cold-cache/);
      });
    },
  );

  itCli(
    "records create-only hints as suppressed cues without default Markdown rendering",
    () => {
      const root = mkdtempSync(path.join(tmpdir(), "pw-cue-create-"));
      const output = mkdtempSync(path.join(tmpdir(), "pw-cue-create-out-"));
      try {
        write(
          root,
          "package.json",
          JSON.stringify({ name: "pw-cue-create", type: "module" }),
        );
        write(root, "src/store.ts", "export const createStore = () => ({});\n");
        write(
          root,
          "src/storage.ts",
          "export const createJSONStorage = () => ({});\n",
        );
        const stdout = runPreWrite(
          root,
          output,
          writeIntent(output, {
            names: [
              {
                name: "createLogger",
                kind: "function",
                why: "create a logger helper",
              },
            ],
            shapes: [],
            files: [],
            dependencies: [],
            plannedTypeEscapes: [],
          }),
        );
        expect(stdout).not.toContain("createStore");
        expect(stdout).not.toContain("createJSONStorage");

        const suppressed = readLatest(output).suppressedCues ?? [];
        expect(
          suppressed.filter(
            (cue) =>
              cue.reason === "domain-token-overlap" &&
              cue.tokenPolicyVersion === "prewrite-token-policy-v1",
          ).length,
        ).toBeGreaterThanOrEqual(2);
        expect(
          suppressed.filter(
            (cue) =>
              cue.evidenceLane === "service-operation-sibling" &&
              cue.reason ===
                "service-sibling-insufficient-suppressed-support" &&
              cue.policyVersion === "prewrite-service-operation-sibling-cue-v1",
          ).length,
        ).toBeGreaterThanOrEqual(2);
      } finally {
        rmSync(root, { recursive: true, force: true });
        rmSync(output, { recursive: true, force: true });
      }
    },
  );
});
