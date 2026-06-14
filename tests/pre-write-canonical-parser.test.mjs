import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";

import {
  findCanonicalOwnerClaim,
  parseCanonicalFile,
} from "../_lib/pre-write-canonical-parser.mjs";

function withCanonicalFile(name, content, fn) {
  const dir = mkdtempSync(path.join(tmpdir(), `canon-parser-${name}-`));
  const filePath = path.join(dir, "canonical", "type-ownership.md");

  try {
    mkdirSync(path.dirname(filePath), { recursive: true });
    writeFileSync(filePath, content);
    return fn({ dir, filePath });
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

describe("pre-write canonical parser owner-claim contracts", () => {
  it("keeps missing and free-form canonical files unrecognized", () => {
    const missing = parseCanonicalFile(
      path.join(tmpdir(), "canon-parser-missing-does-not-exist", "x.md"),
    );
    expect(missing.recognized).toBe(false);
    expect(missing.reason).toMatch(/absent/);

    withCanonicalFile(
      "freeform",
      `# Random canon
This is a free-form canonical document written by hand.

## Owners

- SessionId lives in src/protocol/ids.ts
- User lives in src/models/User.ts
`,
      ({ filePath }) => {
        const result = parseCanonicalFile(filePath);
        expect(result.recognized).toBe(false);
        expect(result.ownerTables).toEqual([]);
        expect(result.reason).toMatch(/header/);
      },
    );
  });

  it("extracts single-owner rows from recognized Status and Source schemas", () => {
    withCanonicalFile(
      "recognized",
      `# canonical-draft/type-ownership.md - DRAFT

> **Role:** observed type ownership derived from AST.
> **Status:** draft, v1
> **Generated:** 2026-04-20T10:00:00Z

## 1. Summary

### 2.1 Single owner (strong)

| Type | Owner | Kind | Line | Fan-in | Re-exported through | Status | Tags | Any / unknown signal |
|---|---|---|---|---:|---|---|---|---|
| \`SessionId\` | \`src/protocol/ids.ts\` | TSTypeAliasDeclaration | 14 | 8 | \`src/index.ts\` | ok | - | - |
| \`User\` | \`src/models/User.ts\` | TSInterfaceDeclaration | 3 | 5 | - | ok | - | - |

### 2.2 Single owner (weak / zero-internal-fan-in)

| Type | Owner | Kind | Line | Fan-in | Re-exported through | Status | Tags | Any / unknown signal | Note |
|---|---|---|---|---:|---|---|---|---|---|
| \`InternalFlag\` | \`src/engine/flag.ts\` | TSTypeAliasDeclaration | 3 | 1 | - | weak | - | - | only \`src/engine/flag-consumer.ts\` |
`,
      ({ filePath }) => {
        const result = parseCanonicalFile(filePath);
        expect(result.recognized).toBe(true);
        expect(result.ownerTables).toHaveLength(2);
        expect(
          findCanonicalOwnerClaim(result.ownerTables, "SessionId"),
        ).toMatchObject({
          ownerFile: "src/protocol/ids.ts",
        });
        expect(
          findCanonicalOwnerClaim(result.ownerTables, "SessionId").line,
        ).toBeGreaterThan(0);
        expect(
          findCanonicalOwnerClaim(result.ownerTables, "InternalFlag"),
        ).toMatchObject({
          ownerFile: "src/engine/flag.ts",
        });
      },
    );

    withCanonicalFile(
      "source-header",
      `# type-ownership.md

> **Source:** \`_lib/extract-ts.mjs\` pass (42 files scanned)

### 2.1 Single owner (strong)

| Type | Owner | Kind | Line | Fan-in | Status |
|---|---|---|---|---:|---|
| \`Token\` | \`src/auth/token.ts\` | TSInterfaceDeclaration | 7 | 4 | ok |
`,
      ({ filePath }) => {
        const result = parseCanonicalFile(filePath);
        expect(result.recognized).toBe(true);
        expect(
          findCanonicalOwnerClaim(result.ownerTables, "Token"),
        ).toMatchObject({
          ownerFile: "src/auth/token.ts",
        });
      },
    );
  });

  it("excludes group-level rows while preserving severe owner-level rows", () => {
    withCanonicalFile(
      "group-sections",
      `# type-ownership.md

> **Status:** draft, v1

### 2.4 DUPLICATE_STRONG - likely shared concept, needs resolution

| Type | Files defining | Kinds | Max fan-in | Total fan-in | Tags | Suggested action |
|---|---|---|---:|---:|---|---|
| \`Result\` | \`src/a.ts:5\`, \`src/b.ts:22\` | 2x TSTypeAliasDeclaration | 18 | 21 | - | pick one |

### 2.6 LOCAL_COMMON_NAME

| Name | Locations | Count | Tags | Note |
|---|---|---:|---|---|
| \`Props\` | 14 files | 14 | - | - |
`,
      ({ filePath }) => {
        const result = parseCanonicalFile(filePath);
        expect(result.recognized).toBe(true);
        expect(result.ownerTables).toEqual([]);
        expect(
          findCanonicalOwnerClaim(result.ownerTables, "Result"),
        ).toBeNull();
        expect(findCanonicalOwnerClaim(result.ownerTables, "Props")).toBeNull();
      },
    );

    withCanonicalFile(
      "severe-section",
      `# type-ownership.md

> **Status:** draft, v1

### 2.3 severely-any-contaminated (single-owner, Rule 0)

| Type | Owner | Kind | Line | Fan-in | Tags | Any / unknown signal |
|---|---|---|---|---:|---|---|
| \`LegacyPayload\` | \`src/legacy/payload.ts\` | TSInterfaceDeclaration | 3 | 6 | - | severely-any-contaminated (anyFieldRatio 0.85) |
`,
      ({ filePath }) => {
        const result = parseCanonicalFile(filePath);
        expect(result.recognized).toBe(true);
        expect(result.ownerTables).toHaveLength(1);
        expect(
          findCanonicalOwnerClaim(result.ownerTables, "LegacyPayload"),
        ).toMatchObject({
          ownerFile: "src/legacy/payload.ts",
        });
      },
    );
  });

  it("ignores free-form prose inside recognized files", () => {
    withCanonicalFile(
      "mixed",
      `# type-ownership.md

> **Status:** draft, v1

This prose mentions \`SessionId\`, \`User\`, and \`Token\`, but it is not a
table and must not become owner evidence.

### 2.1 Single owner (strong)

| Type | Owner | Kind | Line | Fan-in | Status |
|---|---|---|---|---:|---|
| \`SessionId\` | \`src/protocol/ids.ts\` | TSTypeAliasDeclaration | 14 | 8 | ok |
`,
      ({ filePath }) => {
        const result = parseCanonicalFile(filePath);
        expect(result.recognized).toBe(true);
        expect(result.ownerTables).toHaveLength(1);
        expect(result.ownerTables[0].rows).toHaveLength(1);
        expect(
          findCanonicalOwnerClaim(result.ownerTables, "SessionId"),
        ).not.toBeNull();
        expect(findCanonicalOwnerClaim(result.ownerTables, "User")).toBeNull();
        expect(findCanonicalOwnerClaim(result.ownerTables, "Token")).toBeNull();
      },
    );
  });

  it("extracts current flat type-ownership owner rows without duplicate/common rows", () => {
    expect(findCanonicalOwnerClaim([], "Anything")).toBeNull();

    withCanonicalFile(
      "flat-current",
      `# Type ownership draft

Generated: 2026-05-05T00:00:00.000Z
Scope: TS/JS production files
Source: fresh-ast-pass

| Name | Identity | Owner | Fan-in | Fan-in space | Status | Tags |
|------|----------|-------|-------:|--------------|--------|------|
| \`Session\` | \`src/session.ts::Session\` | \`src/session.ts:10\` | 3 | value 2, type 1, broad 0 | single-owner-strong | |
| \`Result\` | \`src/a.ts::Result\` | \`src/a.ts:1\` | 3 | value 3, type 0, broad 0 | DUPLICATE_STRONG | |
| \`Props\` | \`src/card.ts::Props\` | \`src/card.ts:4\` | 1 | value 1, type 0, broad 0 | LOCAL_COMMON_NAME | |
`,
      ({ filePath }) => {
        const result = parseCanonicalFile(filePath);
        expect(result.recognized).toBe(true);
        const session = findCanonicalOwnerClaim(result.ownerTables, "Session");
        expect(session).toMatchObject({ ownerFile: "src/session.ts" });
        expect(session.line).toBeGreaterThan(0);
        expect(
          findCanonicalOwnerClaim(result.ownerTables, "Result"),
        ).toBeNull();
        expect(findCanonicalOwnerClaim(result.ownerTables, "Props")).toBeNull();
      },
    );
  });
});
