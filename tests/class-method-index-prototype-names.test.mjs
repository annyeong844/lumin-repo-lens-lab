import path from "node:path";
import { fileURLToPath } from "node:url";

import { it } from "vitest";

import { buildSymbolsArtifact } from "../skills/lumin-repo-lens-lab/_engine/lib/symbol-graph-artifact.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");
const file = path.join(ROOT, "src/prototype-methods.ts");

function assert(label, ok, detail = "") {
  it(label, () => {
    if (!ok) {
      throw new Error(detail || label);
    }
  });
}

function method(name, line) {
  return {
    identity: `src/prototype-methods.ts::Example#${name}`,
    ownerFile: "src/prototype-methods.ts",
    className: "Example",
    name,
    methodName: name,
    kind: "ClassMethod",
    memberKind: "method",
    visibility: "public",
    static: false,
    computed: false,
    line,
  };
}

let artifact = null;
let thrown = null;
try {
  artifact = buildSymbolsArtifact({
    root: ROOT,
    files: [file],
    defIndex: new Map([[file, new Map()]]),
    fileData: new Map([
      [
        file,
        {
          defs: [],
          uses: [],
          reExports: [],
          classMethods: [
            method("constructor", 2),
            method("toString", 3),
            method("hasOwnProperty", 4),
            method("valueOf", 5),
            method("__proto__", 6),
          ],
          typeEscapes: [],
          loc: 7,
          dynamicImportOpacity: [],
          cjsExportSurface: null,
          cjsRequireOpacity: [],
        },
      ],
    ]),
    parseErrors: 0,
    warnings: [],
    nextCache: { entries: {} },
    unresolvedInternalByPrefix: new Map(),
    prefixExamples: new Map(),
    unresolvedInternalSpecifiers: new Set(),
    unresolvedInternalSpecifierRecords: [],
    languageSupport: {
      ts: { enabled: true, reason: null },
      js: { enabled: true, reason: null },
      python: { enabled: false, reason: "test" },
      go: { enabled: false, reason: "test" },
    },
    totalUses: 0,
    unresolvedUses: 0,
    resolvedInternalUses: 0,
    externalUses: 0,
    dependencyImportConsumers: [],
    resolvedInternalEdges: [],
    generatedConsumerBlindZones: [],
    generatedVirtualSurfaces: new Map(),
    generatedVirtualImportConsumers: [],
    unresolvedInternalUses: 0,
    mdxConsumerUses: 0,
    dead: [],
    trulyDead: [],
    deadInProd: [],
    deadInTest: [],
    symbolFanIn: [],
    fanInByIdentity: {},
    fanInByIdentitySpace: {},
  });
} catch (error) {
  thrown = error;
}

assert(
  "CM-PROTO1. classMethodIndex accepts prototype method names without crashing",
  !thrown,
  thrown?.stack ?? "",
);

const index = artifact?.classMethodIndex?.["src/prototype-methods.ts"] ?? {};
assert(
  "CM-PROTO2. classMethodIndex stores prototype-named methods as own keys",
  Array.isArray(index["constructor"]) &&
    Array.isArray(index["toString"]) &&
    Array.isArray(index["hasOwnProperty"]) &&
    Array.isArray(index["valueOf"]) &&
    Array.isArray(index["__proto__"]),
  JSON.stringify(index, null, 2),
);
