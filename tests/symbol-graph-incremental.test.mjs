import { execFileSync } from "node:child_process";
import {
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

const NODE = process.execPath;
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");
const CLI = path.join(ROOT, "build-symbol-graph.mjs");
const TEST_TIMEOUT = 60_000;

function fresh() {
  return mkdtempSync(path.join(tmpdir(), "vitest-symbol-inc-"));
}

function write(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
}

function run(root, output, args = []) {
  return execFileSync(
    NODE,
    [CLI, "--root", root, "--output", output, ...args],
    {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
}

function readSymbols(output) {
  return JSON.parse(readFileSync(path.join(output, "symbols.json"), "utf8"));
}

function findSymbolsCacheFile(repo) {
  const base = path.join(repo, ".audit", ".cache", "incremental");
  for (const dir of readdirSync(base, { withFileTypes: true })) {
    if (!dir.isDirectory()) continue;
    const file = path.join(base, dir.name, "symbols.cache.json");
    try {
      readFileSync(file);
      return file;
    } catch {
      // Try the next repo-fingerprint directory.
    }
  }
  throw new Error(`symbols cache not found under ${base}`);
}

function rewriteSymbolsCache(repo, mutateEntry) {
  const file = findSymbolsCacheFile(repo);
  const cache = JSON.parse(readFileSync(file, "utf8"));
  for (const entry of Object.values(cache.entries ?? {})) mutateEntry(entry);
  writeFileSync(file, `${JSON.stringify(cache, null, 2)}\n`);
}

function stableSymbols(symbols) {
  const { meta, ...rest } = symbols;
  return {
    meta: {
      schemaVersion: meta?.schemaVersion,
      supports: meta?.supports,
      languageSupport: meta?.languageSupport,
      warnings: meta?.warnings ?? [],
    },
    ...rest,
  };
}

function setupRepo(repo) {
  write(
    repo,
    "package.json",
    JSON.stringify({ name: "fixture", private: true }),
  );
  write(
    repo,
    "src/a.ts",
    [
      "export function used() { return 1; }",
      "export const unused = 2;",
      "",
    ].join("\n"),
  );
  write(
    repo,
    "src/b.ts",
    ["import { used } from './a';", "export const consumer = used();", ""].join(
      "\n",
    ),
  );
}

describe("symbol graph strict incremental cache", () => {
  it(
    "matches cold public facts, keeps warm equivalence, reports strict mode, and reuses unchanged facts",
    () => {
      const repo = fresh();
      const output = path.join(repo, ".audit");
      try {
        setupRepo(repo);

        run(repo, output, ["--no-incremental"]);
        const cold = readSymbols(output);
        run(repo, output);
        const firstIncremental = readSymbols(output);
        run(repo, output);
        const warm = readSymbols(output);

        expect(stableSymbols(firstIncremental)).toEqual(stableSymbols(cold));
        expect(stableSymbols(warm)).toEqual(stableSymbols(cold));
        expect(warm.meta.incremental).toMatchObject({
          enabled: true,
          identityMode: "strict-content-hash",
        });
        expect(warm.meta.incremental.reusedFiles).toBeGreaterThanOrEqual(2);
      } finally {
        rmSync(repo, { recursive: true, force: true });
      }
    },
    TEST_TIMEOUT,
  );

  it(
    "refreshes changed consumer fan-in while reusing unchanged files",
    () => {
      const repo = fresh();
      const output = path.join(repo, ".audit");
      try {
        setupRepo(repo);
        run(repo, output);
        run(repo, output);

        write(repo, "src/b.ts", ["export const consumer = 0;", ""].join("\n"));
        run(repo, output);
        const symbols = readSymbols(output);

        expect(symbols.fanInByIdentity?.["src/a.ts::used"]).toBe(0);
        expect(symbols.meta.incremental.changedFiles).toBeGreaterThanOrEqual(1);
        expect(symbols.meta.incremental.reusedFiles).toBeGreaterThanOrEqual(1);
      } finally {
        rmSync(repo, { recursive: true, force: true });
      }
    },
    TEST_TIMEOUT,
  );

  it(
    "drops deleted definition facts and reports dropped-file evidence",
    () => {
      const repo = fresh();
      const output = path.join(repo, ".audit");
      try {
        setupRepo(repo);
        run(repo, output);
        run(repo, output);

        rmSync(path.join(repo, "src/a.ts"), { force: true });
        run(repo, output);
        const symbols = readSymbols(output);

        expect(symbols.defIndex?.["src/a.ts"]).toBeUndefined();
        expect(symbols.meta.incremental.droppedFiles).toBeGreaterThanOrEqual(1);
      } finally {
        rmSync(repo, { recursive: true, force: true });
      }
    },
    TEST_TIMEOUT,
  );

  it(
    "reports disabled cache metadata under --no-incremental",
    () => {
      const repo = fresh();
      const output = path.join(repo, ".audit");
      try {
        setupRepo(repo);
        run(repo, output, ["--no-incremental"]);
        const symbols = readSymbols(output);

        expect(symbols.meta.incremental).toMatchObject({
          enabled: false,
          reason: "disabled-by-flag",
        });
      } finally {
        rmSync(repo, { recursive: true, force: true });
      }
    },
    TEST_TIMEOUT,
  );

  it(
    "invalidates legacy symbol caches missing CJS export surface facts",
    () => {
      const repo = fresh();
      const output = path.join(repo, ".audit");
      try {
        write(
          repo,
          "package.json",
          JSON.stringify({ name: "fixture", private: true }),
        );
        write(
          repo,
          "src/exporter.cjs",
          ["exports.foo = 1;", "module.exports = makeExports();", ""].join(
            "\n",
          ),
        );

        run(repo, output);
        rewriteSymbolsCache(repo, (entry) => {
          if (entry.identity?.relPath !== "src/exporter.cjs") return;
          delete entry.payload.cjsExportSurface;
          entry.producerMeta.producerVersion = 1;
          entry.producerMeta.factSchemaVersion = 2;
          entry.producerMeta.parserIdentity = "symbol-graph-extractors:v1";
        });

        run(repo, output);
        const symbols = readSymbols(output);
        const surface = symbols.cjsExportSurfaceByFile?.["src/exporter.cjs"];

        expect(surface?.exact?.some((entry) => entry.name === "foo")).toBe(
          true,
        );
        expect(
          surface?.opaque?.some(
            (entry) => entry.kind === "module-exports-assignment",
          ),
        ).toBe(true);
        expect(
          symbols.meta.incremental.invalidatedFiles,
        ).toBeGreaterThanOrEqual(1);
      } finally {
        rmSync(repo, { recursive: true, force: true });
      }
    },
    TEST_TIMEOUT,
  );

  it(
    "invalidates legacy symbol caches missing dynamic CJS require opacity",
    () => {
      const repo = fresh();
      const output = path.join(repo, ".audit");
      try {
        write(
          repo,
          "package.json",
          JSON.stringify({ name: "fixture", private: true }),
        );
        write(
          repo,
          "src/consumer.js",
          ['const target = "./exporter.js";', "require(target);", ""].join(
            "\n",
          ),
        );

        run(repo, output);
        rewriteSymbolsCache(repo, (entry) => {
          if (entry.identity?.relPath !== "src/consumer.js") return;
          delete entry.payload.cjsRequireOpacity;
          entry.producerMeta.producerVersion = 1;
          entry.producerMeta.factSchemaVersion = 2;
          entry.producerMeta.parserIdentity = "symbol-graph-extractors:v1";
        });

        run(repo, output);
        const symbols = readSymbols(output);

        expect(
          symbols.cjsRequireOpacity?.some(
            (entry) =>
              entry.consumerFile === "src/consumer.js" &&
              entry.kind === "dynamic-require" &&
              entry.line === 2,
          ),
        ).toBe(true);
        expect(
          symbols.meta.incremental.invalidatedFiles,
        ).toBeGreaterThanOrEqual(1);
      } finally {
        rmSync(repo, { recursive: true, force: true });
      }
    },
    TEST_TIMEOUT,
  );

  it(
    "invalidates stale JSON require opacity from older schemas",
    () => {
      const repo = fresh();
      const output = path.join(repo, ".audit");
      try {
        write(
          repo,
          "package.json",
          JSON.stringify({ name: "fixture", private: true }),
        );
        write(
          repo,
          "src/version-checker.js",
          [
            'import path from "node:path";',
            'import { createRequire } from "node:module";',
            "const require = createRequire(import.meta.url);",
            "export function getCurrentVersion() {",
            '  return require(path.resolve(import.meta.dirname, "../package.json")).version;',
            "}",
            "",
          ].join("\n"),
        );

        run(repo, output);
        rewriteSymbolsCache(repo, (entry) => {
          if (entry.identity?.relPath !== "src/version-checker.js") return;
          entry.payload.cjsRequireOpacity = [
            { line: 5, kind: "dynamic-require" },
          ];
          entry.producerMeta.producerVersion = 1;
          entry.producerMeta.factSchemaVersion = 3;
          entry.producerMeta.parserIdentity = "symbol-graph-extractors:v1";
        });

        run(repo, output);
        const symbols = readSymbols(output);

        expect(symbols.cjsRequireOpacity ?? []).toHaveLength(0);
        expect(
          symbols.meta.incremental.invalidatedFiles,
        ).toBeGreaterThanOrEqual(1);
      } finally {
        rmSync(repo, { recursive: true, force: true });
      }
    },
    TEST_TIMEOUT,
  );

  it(
    "invalidates old CJS extractor identities so bracket member fan-in stays precise",
    () => {
      const repo = fresh();
      const output = path.join(repo, ".audit");
      try {
        write(
          repo,
          "package.json",
          JSON.stringify({ name: "fixture", private: true }),
        );
        write(repo, "src/exporter.js", "export const foo = 1;\n");
        write(
          repo,
          "src/consumer.js",
          [
            'const mod = require("./exporter.js");',
            'if (mod) mod["foo"]();',
            "",
          ].join("\n"),
        );

        run(repo, output);
        rewriteSymbolsCache(repo, (entry) => {
          if (entry.identity?.relPath !== "src/consumer.js") return;
          entry.producerMeta.parserIdentity = "symbol-graph-extractors:v1";
        });

        run(repo, output);
        const symbols = readSymbols(output);

        expect(symbols.fanInByIdentity?.["src/exporter.js::foo"]).toBe(1);
        expect(
          symbols.meta.incremental.invalidatedFiles,
        ).toBeGreaterThanOrEqual(1);
      } finally {
        rmSync(repo, { recursive: true, force: true });
      }
    },
    TEST_TIMEOUT,
  );
});
