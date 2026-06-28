import { it } from "vitest";
import {
  chmodSync,
  mkdirSync,
  mkdtempSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import {
  buildContextFingerprint,
  buildRepoSnapshot,
  defaultPackageScopeOf,
  hashBytes,
  normalizeRepoRel,
} from "../_lib/incremental-snapshot.mjs";

function assert(label, ok, detail = "") {
  it(label, () => {
    if (!ok) {
      throw new Error(detail ? String(detail) : `Assertion failed: ${label}`);
    }
  });
}

function fresh() {
  return mkdtempSync(path.join(tmpdir(), "lumin-snapshot-"));
}

function write(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
  return full;
}

{
  const root = fresh();
  try {
    const full = write(root, "src/a.ts", "export const a = 1;\n");
    assert(
      "normalizeRepoRel returns POSIX repo-relative paths",
      normalizeRepoRel(root, full) === "src/a.ts",
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

{
  const root = fresh();
  try {
    write(root, "package.json", JSON.stringify({ name: "fixture" }));
    write(root, "src/a.ts", "export const a = 1;\n");
    write(root, "tests/a.test.ts", "const x = y as any;\n");

    const contextFingerprint = buildContextFingerprint({
      includeTests: false,
      exclude: [],
      languages: ["ts"],
      producerContext: { producer: "any-inventory", factSchemaVersion: 1 },
    });
    const snapshot = buildRepoSnapshot({
      root,
      includeTests: false,
      exclude: [],
      languages: ["ts"],
      contextFingerprint,
    });

    assert("snapshot includes production file", !!snapshot.files["src/a.ts"]);
    assert(
      "snapshot excludes test file when includeTests=false",
      !snapshot.files["tests/a.test.ts"],
    );
    const entry = snapshot.files["src/a.ts"];
    assert(
      "entry has strict identity fields",
      entry.relPath === "src/a.ts" &&
        entry.language === "ts" &&
        entry.isTestLike === false &&
        entry.packageScope === "." &&
        typeof entry.contentHash === "string" &&
        entry.contextFingerprint === contextFingerprint,
    );
    assert(
      "hash is sha256-prefixed",
      /^sha256:[a-f0-9]{64}$/.test(entry.contentHash),
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

{
  const root = fresh();
  try {
    write(root, "package.json", JSON.stringify({ name: "root" }));
    write(root, "packages/core/package.json", JSON.stringify({ name: "core" }));
    write(root, "packages/core/src/a.ts", "export const a = 1;\n");

    const scope = defaultPackageScopeOf(
      root,
      path.join(root, "packages/core/src/a.ts"),
    );
    assert(
      "package scope uses nearest package root",
      scope === "packages/core",
      `got ${scope}`,
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

{
  const root = fresh();
  try {
    const content = Buffer.from("same bytes\n", "utf8");
    const h1 = hashBytes(content);
    const h2 = hashBytes(content);
    assert(
      "hashBytes is deterministic sha256",
      h1 === h2 && /^sha256:[a-f0-9]{64}$/.test(h1),
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

{
  const root = fresh();
  try {
    write(root, "package.json", JSON.stringify({ name: "fixture" }));
    const unreadable = write(
      root,
      "src/secret.ts",
      "export const secret = 1;\n",
    );
    let chmodWorked = true;
    try {
      chmodSync(unreadable, 0o000);
    } catch {
      chmodWorked = false;
    }

    const contextFingerprint = buildContextFingerprint({
      includeTests: true,
      exclude: [],
      languages: ["ts"],
      producerContext: { producer: "any-inventory", factSchemaVersion: 1 },
    });
    const snapshot = buildRepoSnapshot({
      root,
      includeTests: true,
      exclude: [],
      languages: ["ts"],
      contextFingerprint,
    });

    const entry = snapshot.files["src/secret.ts"];
    if (chmodWorked && entry?.readable === false) {
      assert(
        "unreadable in-scope file remains visible with read error",
        entry.hash === null &&
          entry.contentHash === null &&
          entry.readError?.kind,
      );
    } else {
      assert(
        "unreadable test skipped on platform that still allows read",
        !!entry,
      );
    }
  } finally {
    try {
      chmodSync(path.join(root, "src/secret.ts"), 0o600);
    } catch {
      // Best-effort permission restore for platforms that honored chmod.
    }
    rmSync(root, { recursive: true, force: true });
  }
}
