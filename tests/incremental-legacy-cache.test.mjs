import { it } from "vitest";
import {
  mkdtempSync,
  rmSync,
  statSync,
  utimesSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import {
  cacheBanner,
  loadCache,
  pickChangedFiles,
  saveCache,
} from "../_lib/incremental.mjs";

function assert(label, ok, detail = "") {
  it(label, () => {
    if (!ok) {
      throw new Error(detail ? String(detail) : `Assertion failed: ${label}`);
    }
  });
}

function fresh() {
  return mkdtempSync(path.join(tmpdir(), "inc-"));
}

{
  const dir = fresh();
  try {
    const a = path.join(dir, "a.ts");
    const b = path.join(dir, "b.ts");
    writeFileSync(a, "export const x = 1;\n");
    writeFileSync(b, "export const y = 2;\n");
    const cache = loadCache(dir, "demo");
    const { changed, unchanged, dropped, nextCache } = pickChangedFiles(
      [a, b],
      cache,
    );
    assert(
      "T1. first run: both files in changed",
      changed.length === 2 && unchanged.length === 0,
    );
    assert("T1b. dropped empty on first run", dropped.length === 0);
    assert(
      "T1c. nextCache entries have hash + mtimeMs + size",
      Object.values(nextCache.entries).every(
        (e) =>
          typeof e.hash === "string" &&
          typeof e.mtimeMs === "number" &&
          typeof e.size === "number",
      ),
    );
    saveCache(dir, "demo", nextCache);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

{
  const dir = fresh();
  try {
    const a = path.join(dir, "a.ts");
    writeFileSync(a, "export const x = 1;\n");
    const next = pickChangedFiles([a], loadCache(dir, "demo")).nextCache;
    saveCache(dir, "demo", next);
    const second = pickChangedFiles([a], loadCache(dir, "demo"));
    assert(
      "T2. second run with no change: file in unchanged",
      second.unchanged.length === 1 && second.changed.length === 0,
    );
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

{
  const dir = fresh();
  try {
    const a = path.join(dir, "a.ts");
    writeFileSync(a, "export const x = 1;\n");
    const st = statSync(a);
    const FAKE_HASH = "fake-stat-cut-proof";
    const plantedCache = {
      version: loadCache(dir, "demo").version,
      entries: {
        [a]: { hash: FAKE_HASH, mtimeMs: st.mtimeMs, size: st.size },
      },
    };
    const { changed, unchanged, nextCache } = pickChangedFiles(
      [a],
      plantedCache,
    );
    assert(
      "T3a. stat match -> file classified as unchanged (no hash computed)",
      unchanged.length === 1 && changed.length === 0,
    );
    assert(
      "T3b. stat-first-cut preserved the planted fake hash (proves hash skipped)",
      nextCache.entries[a].hash === FAKE_HASH,
    );
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

{
  const dir = fresh();
  try {
    const a = path.join(dir, "a.ts");
    writeFileSync(a, "export const x = 1;\n");
    const next = pickChangedFiles([a], loadCache(dir, "demo")).nextCache;
    const realHash = next.entries[a].hash;
    saveCache(dir, "demo", next);

    const past = statSync(a).mtime;
    const future = new Date(past.getTime() + 2000);
    utimesSync(a, future, future);

    const second = pickChangedFiles([a], loadCache(dir, "demo"));
    assert(
      "T4a. mtime changed but content same -> still unchanged (hash re-matched)",
      second.unchanged.length === 1 && second.changed.length === 0,
    );
    assert(
      "T4b. nextCache.mtimeMs updated to the new mtime",
      second.nextCache.entries[a].mtimeMs !== next.entries[a].mtimeMs,
    );
    assert(
      "T4c. hash preserved identical to prior",
      second.nextCache.entries[a].hash === realHash,
    );
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

{
  const dir = fresh();
  try {
    const a = path.join(dir, "a.ts");
    writeFileSync(a, "export const x = 1;\n");
    const next = pickChangedFiles([a], loadCache(dir, "demo")).nextCache;
    saveCache(dir, "demo", next);

    writeFileSync(a, "export const x = 99;\n");
    const second = pickChangedFiles([a], loadCache(dir, "demo"));
    assert(
      "T5. content edit -> changed",
      second.changed.length === 1 && second.unchanged.length === 0,
    );
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

{
  const dir = fresh();
  try {
    const a = path.join(dir, "a.ts");
    const b = path.join(dir, "b.ts");
    writeFileSync(a, "export const x = 1;\n");
    writeFileSync(b, "export const y = 2;\n");
    saveCache(
      dir,
      "demo",
      pickChangedFiles([a, b], loadCache(dir, "demo")).nextCache,
    );

    const second = pickChangedFiles([a], loadCache(dir, "demo"));
    assert(
      "T6. file no longer in list appears in dropped",
      second.dropped.length === 1 && second.dropped[0] === b,
    );
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

{
  const dir = fresh();
  try {
    const a = path.join(dir, "a.ts");
    writeFileSync(a, "export const x = 1;\n");
    saveCache(dir, "demo", {
      version: 0,
      entries: { [a]: { hash: "old", mtimeMs: 1, size: 1 } },
    });
    const loaded = loadCache(dir, "demo");
    assert(
      "T7. obsolete version resets cache to empty",
      Object.keys(loaded.entries).length === 0,
    );
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

{
  const s = cacheBanner("topology", [1, 2], [3, 4, 5, 6, 7, 8], [9]);
  assert(
    "T8. cacheBanner mentions the name + counts + percentage",
    s.includes("topology") &&
      s.includes("2 changed") &&
      s.includes("6 cached") &&
      /75%/.test(s) &&
      s.includes("1 dropped"),
  );
}
