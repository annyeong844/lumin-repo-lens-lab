import { it } from "vitest";
import { execFileSync } from "node:child_process";
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const NODE = process.execPath;
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");
const ANY = path.join(ROOT, "any-inventory.mjs");
const POST = path.join(ROOT, "post-write.mjs");

function assert(label, ok, detail = "") {
  it(label, () => {
    if (!ok) {
      throw new Error(detail ? String(detail) : `Assertion failed: ${label}`);
    }
  });
}

function fresh() {
  return mkdtempSync(path.join(tmpdir(), "lumin-post-inc-"));
}

function write(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
}

function runAny(root, out, args = []) {
  execFileSync(NODE, [ANY, "--root", root, "--output", out, ...args], {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
}

function readJson(file) {
  return JSON.parse(readFileSync(file, "utf8"));
}

{
  const repo = fresh();
  const out = path.join(repo, ".audit");
  try {
    write(repo, "package.json", JSON.stringify({ name: "fixture" }));
    write(repo, "src/a.ts", "const a = value as any;\n");
    mkdirSync(out, { recursive: true });

    runAny(repo, out, ["--artifact-name", "any-inventory.pre.invocation.json"]);
    const advisory = {
      invocationId: "pre-write-test",
      preWrite: {
        anyInventoryPath: "any-inventory.pre.invocation.json",
        fileInventory: { status: "available", files: ["src/a.ts"] },
      },
      intent: { files: ["src/a.ts"], plannedTypeEscapes: [] },
      scanRange: { output: out },
    };
    const advisoryPath = path.join(out, "pre-write-advisory.json");
    writeFileSync(advisoryPath, JSON.stringify(advisory, null, 2));

    write(
      repo,
      "src/a.ts",
      "const a = value as any;\nconst b = value as unknown as string;\n",
    );
    execFileSync(
      NODE,
      [
        POST,
        "--root",
        repo,
        "--output",
        out,
        "--pre-write-advisory",
        advisoryPath,
      ],
      { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] },
    );

    const after = readJson(path.join(out, "any-inventory.json"));
    assert(
      "post-write after-snapshot uses incremental any-inventory by default",
      after.meta.incremental?.enabled === true,
      JSON.stringify(after.meta.incremental),
    );

    const before = readJson(
      path.join(out, "any-inventory.pre.invocation.json"),
    );
    assert(
      "pre-write baseline artifact is not mutated by post-write",
      !before.typeEscapes.some((fact) => fact.escapeKind === "as-unknown-as-T"),
    );
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

{
  const repo = fresh();
  const out = path.join(repo, ".audit");
  try {
    write(repo, "package.json", JSON.stringify({ name: "fixture" }));
    write(repo, "src/a.ts", "const a = value as any;\n");
    mkdirSync(out, { recursive: true });

    runAny(repo, out, ["--artifact-name", "any-inventory.pre.invocation.json"]);
    const advisoryPath = path.join(out, "pre-write-advisory.json");
    writeFileSync(
      advisoryPath,
      JSON.stringify(
        {
          invocationId: "pre-write-test",
          preWrite: {
            anyInventoryPath: "any-inventory.pre.invocation.json",
            fileInventory: { status: "available", files: ["src/a.ts"] },
          },
          intent: { files: ["src/a.ts"], plannedTypeEscapes: [] },
          scanRange: { output: out },
        },
        null,
        2,
      ),
    );

    execFileSync(
      NODE,
      [
        POST,
        "--root",
        repo,
        "--output",
        out,
        "--pre-write-advisory",
        advisoryPath,
        "--no-incremental",
      ],
      { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] },
    );

    const after = readJson(path.join(out, "any-inventory.json"));
    assert(
      "post-write forwards --no-incremental to after-snapshot",
      after.meta.incremental?.enabled === false &&
        after.meta.incremental?.reason === "disabled-by-flag",
      JSON.stringify(after.meta.incremental),
    );
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}
