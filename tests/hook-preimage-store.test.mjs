import assert from "node:assert/strict";
import { it } from "vitest";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  statSync,
  utimesSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import {
  capturePreimage,
  cleanupOldPreimages,
  cleanupPreimage,
  preimagePath,
  readPreimage,
} from "../_lib/hook-preimage-store.mjs";

function check(label, fn) {
  it(label, fn);
}

function fixture() {
  const root = mkdtempSync(path.join(tmpdir(), "lrl-hook-preimage-"));
  const auditRoot = path.join(root, ".audit");
  const src = path.join(root, "src");
  mkdirSync(src, { recursive: true });
  const file = path.join(src, "a.ts");
  writeFileSync(file, 'const secret = "do-not-store";\n');
  return {
    root,
    auditRoot,
    file,
    safe: {
      ok: true,
      repoRoot: root,
      absolute: file,
      repoRel: "src/a.ts",
      ext: ".ts",
      exists: true,
      sizeBytes: statSync(file).size,
      kind: "file",
    },
  };
}

check("HPI1. preimagePath rejects unsafe ids and builds session path", () => {
  const fx = fixture();
  try {
    assert.equal(
      preimagePath(fx.auditRoot, "sid_123", "tool_abc"),
      path.join(
        fx.auditRoot,
        "sessions",
        "sid_123",
        "preimages",
        "tool_abc.json",
      ),
    );
    assert.throws(
      () => preimagePath(fx.auditRoot, "../bad", "tool_abc"),
      /unsafe session id/,
    );
    assert.throws(
      () => preimagePath(fx.auditRoot, "sid_123", "../bad"),
      /unsafe tool use id/,
    );
  } finally {
    rmSync(fx.root, { recursive: true, force: true });
  }
});

check(
  "HPI2. capture existing file writes fingerprint without raw source text",
  () => {
    const fx = fixture();
    try {
      const record = capturePreimage({
        auditRoot: fx.auditRoot,
        sid: "sid_123",
        tid: "tool_abc",
        safe: fx.safe,
        now: new Date("2026-05-08T00:00:00.000Z"),
      });
      assert.equal(record.schemaVersion, "hook-preimage.v1");
      assert.equal(record.repoRel, "src/a.ts");
      assert.equal(record.toolUseId, "tool_abc");
      assert.equal(record.absent, false);
      assert.match(record.fingerprint.sha256, /^sha256:[a-f0-9]{64}$/);
      assert.equal(record.fingerprint.sizeBytes, fx.safe.sizeBytes);
      assert.equal(typeof record.fingerprint.mtimeMs, "number");

      const raw = readFileSync(
        preimagePath(fx.auditRoot, "sid_123", "tool_abc"),
        "utf8",
      );
      assert.equal(raw.includes("do-not-store"), false);

      const readBack = readPreimage(fx.auditRoot, "sid_123", "tool_abc");
      assert.deepEqual(readBack, record);
    } finally {
      rmSync(fx.root, { recursive: true, force: true });
    }
  },
);

check("HPI3. capture missing file records absent preimage", () => {
  const fx = fixture();
  try {
    const missing = path.join(fx.root, "src", "new.ts");
    const record = capturePreimage({
      auditRoot: fx.auditRoot,
      sid: "sid_123",
      tid: "tool_missing",
      safe: {
        ...fx.safe,
        absolute: missing,
        repoRel: "src/new.ts",
        exists: false,
        sizeBytes: null,
        kind: "missing",
      },
      now: new Date("2026-05-08T00:00:00.000Z"),
    });
    assert.equal(record.absent, true);
    assert.equal(record.fingerprint, null);
    assert.deepEqual(
      readPreimage(fx.auditRoot, "sid_123", "tool_missing"),
      record,
    );
  } finally {
    rmSync(fx.root, { recursive: true, force: true });
  }
});

check(
  "HPI4. readPreimage returns null for missing, malformed, or unsafe records",
  () => {
    const fx = fixture();
    try {
      assert.equal(readPreimage(fx.auditRoot, "sid_123", "tool_none"), null);
      mkdirSync(
        path.dirname(preimagePath(fx.auditRoot, "sid_123", "tool_bad")),
        { recursive: true },
      );
      writeFileSync(
        preimagePath(fx.auditRoot, "sid_123", "tool_bad"),
        "{ bad json",
      );
      assert.equal(readPreimage(fx.auditRoot, "sid_123", "tool_bad"), null);
      assert.equal(readPreimage(fx.auditRoot, "../bad", "tool_bad"), null);
      assert.equal(readPreimage(fx.auditRoot, "sid_123", "../bad"), null);
    } finally {
      rmSync(fx.root, { recursive: true, force: true });
    }
  },
);

check("HPI5. cleanupPreimage removes only the requested tool preimage", () => {
  const fx = fixture();
  try {
    capturePreimage({
      auditRoot: fx.auditRoot,
      sid: "sid_123",
      tid: "tool_a",
      safe: fx.safe,
    });
    capturePreimage({
      auditRoot: fx.auditRoot,
      sid: "sid_123",
      tid: "tool_b",
      safe: fx.safe,
    });
    assert.equal(cleanupPreimage(fx.auditRoot, "sid_123", "tool_a"), true);
    assert.equal(readPreimage(fx.auditRoot, "sid_123", "tool_a"), null);
    assert.notEqual(readPreimage(fx.auditRoot, "sid_123", "tool_b"), null);
  } finally {
    rmSync(fx.root, { recursive: true, force: true });
  }
});

check("HPI6. cleanupOldPreimages removes only old json preimages", () => {
  const fx = fixture();
  try {
    capturePreimage({
      auditRoot: fx.auditRoot,
      sid: "sid_123",
      tid: "tool_old",
      safe: fx.safe,
    });
    capturePreimage({
      auditRoot: fx.auditRoot,
      sid: "sid_123",
      tid: "tool_new",
      safe: fx.safe,
    });
    const oldPath = preimagePath(fx.auditRoot, "sid_123", "tool_old");
    const newPath = preimagePath(fx.auditRoot, "sid_123", "tool_new");
    const oldDate = new Date("2026-05-08T00:00:00.000Z");
    const newDate = new Date("2026-05-08T02:00:00.000Z");
    utimesSync(oldPath, oldDate, oldDate);
    utimesSync(newPath, newDate, newDate);
    const removed = cleanupOldPreimages(fx.auditRoot, "sid_123", {
      now: newDate,
      maxAgeMs: 60 * 60 * 1000,
    });
    assert.equal(removed, 1);
    assert.equal(existsSync(oldPath), false);
    assert.equal(existsSync(newPath), true);
  } finally {
    rmSync(fx.root, { recursive: true, force: true });
  }
});

check("HPI7. capture rejects unsafe safe-path payloads", () => {
  const fx = fixture();
  try {
    assert.throws(
      () =>
        capturePreimage({
          auditRoot: fx.auditRoot,
          sid: "sid_123",
          tid: "tool_bad",
          safe: { ...fx.safe, repoRel: "../outside.ts" },
        }),
      /unsafe repo-relative path/,
    );
    assert.throws(
      () =>
        capturePreimage({
          auditRoot: fx.auditRoot,
          sid: "sid_123",
          tid: "tool_bad",
          safe: { ok: false, reason: "outside-repo" },
        }),
      /safe path is required/,
    );
  } finally {
    rmSync(fx.root, { recursive: true, force: true });
  }
});

check(
  "HPI8. capture re-derives absolute path from repo root and repoRel",
  () => {
    const fx = fixture();
    const outside = path.join(
      fx.root,
      "..",
      `${path.basename(fx.root)}-outside.ts`,
    );
    try {
      writeFileSync(
        outside,
        'const secret = "outside-file-with-different-size";\n',
      );
      const record = capturePreimage({
        auditRoot: fx.auditRoot,
        sid: "sid_123",
        tid: "tool_rederive",
        safe: {
          ...fx.safe,
          absolute: outside,
          sizeBytes: statSync(outside).size,
        },
      });
      const raw = readFileSync(
        preimagePath(fx.auditRoot, "sid_123", "tool_rederive"),
        "utf8",
      );
      assert.equal(raw.includes("outside-file"), false);
      assert.equal(record.fingerprint.sizeBytes, statSync(fx.file).size);
    } finally {
      rmSync(outside, { force: true });
      rmSync(fx.root, { recursive: true, force: true });
    }
  },
);
