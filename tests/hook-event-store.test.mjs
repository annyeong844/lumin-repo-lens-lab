import assert from "node:assert/strict";
import { it } from "vitest";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  utimesSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import {
  appendEventIfNotDeduped,
  claimDueDeliveriesAndAdvanceCursor,
  cleanupAckedEntries,
  eventStoreDir,
  markAcknowledged,
  markDelivered,
  readEventStoreState,
} from "../_lib/hook-event-store.mjs";

function check(label, fn) {
  it(label, fn);
}

function fixture() {
  const root = mkdtempSync(path.join(tmpdir(), "lrl-hook-event-"));
  return {
    root,
    auditRoot: path.join(root, ".audit"),
    sid: "sid_123",
  };
}

function event(overrides = {}) {
  return {
    kind: "silent-new",
    severity: "warn",
    ack_required: true,
    delivery_policy: "until_ack",
    diff_key: "diff_a",
    dedupe_key: "dedupe_a",
    occurrence_delta: 1,
    data: {
      file: "src/a.ts",
      line: 12,
      escape_kind: "as-any",
      snippet: "value as any",
      enclosing_symbol: "parse",
      matched_line_text: "const value = raw as any",
    },
    ...overrides,
  };
}

check("HES1. eventStoreDir rejects unsafe session ids and builds path", () => {
  const fx = fixture();
  try {
    assert.equal(
      eventStoreDir(fx.auditRoot, fx.sid),
      path.join(fx.auditRoot, "sessions", fx.sid, "event-store"),
    );
    assert.throws(
      () => eventStoreDir(fx.auditRoot, "../bad"),
      /unsafe session id/,
    );
  } finally {
    rmSync(fx.root, { recursive: true, force: true });
  }
});

check("HES2. appendEventIfNotDeduped appends new active event", () => {
  const fx = fixture();
  try {
    const result = appendEventIfNotDeduped(fx.auditRoot, fx.sid, event(), {
      now: new Date("2026-05-08T00:00:00.000Z"),
    });
    assert.equal(result.appended, true);
    assert.match(result.eventId, /^evt_[a-f0-9]{16}$/);
    const state = readEventStoreState(fx.auditRoot, fx.sid);
    assert.equal(state.entries.length, 1);
    assert.equal(state.entries[0].active, true);
    assert.equal(state.entries[0].session_id, fx.sid);
    assert.equal(state.entries[0].occurrence_count, 1);
    assert.equal(state.entries[0].delivered_count, 0);
    assert.equal(state.entries[0].acknowledged, false);
  } finally {
    rmSync(fx.root, { recursive: true, force: true });
  }
});

check("HES3. duplicate append merges active unacknowledged event", () => {
  const fx = fixture();
  try {
    const first = appendEventIfNotDeduped(
      fx.auditRoot,
      fx.sid,
      event({ occurrence_delta: 1 }),
    );
    const second = appendEventIfNotDeduped(
      fx.auditRoot,
      fx.sid,
      event({
        occurrence_delta: 3,
        data: {
          ...event().data,
          line: 20,
          snippet: "newer as any",
        },
      }),
    );
    assert.equal(second.appended, false);
    assert.equal(second.eventId, first.eventId);
    const [entry] = readEventStoreState(fx.auditRoot, fx.sid).entries;
    assert.equal(entry.occurrence_count, 4);
    assert.equal(entry.data.line, 20);
    assert.equal(entry.data.snippet, "newer as any");
  } finally {
    rmSync(fx.root, { recursive: true, force: true });
  }
});

check("HES4. acknowledged active event suppresses same dedupe key", () => {
  const fx = fixture();
  try {
    const first = appendEventIfNotDeduped(fx.auditRoot, fx.sid, event());
    assert.equal(
      markAcknowledged(fx.auditRoot, fx.sid, first.eventId, "intentional", {
        now: new Date("2026-05-08T00:10:00.000Z"),
      }),
      true,
    );
    const second = appendEventIfNotDeduped(fx.auditRoot, fx.sid, event());
    assert.equal(second.appended, false);
    assert.equal(second.eventId, first.eventId);
    assert.equal(readEventStoreState(fx.auditRoot, fx.sid).entries.length, 1);
  } finally {
    rmSync(fx.root, { recursive: true, force: true });
  }
});

check(
  "HES5. until_ack tombstone suppresses same dedupe key after cleanup",
  () => {
    const fx = fixture();
    try {
      const first = appendEventIfNotDeduped(fx.auditRoot, fx.sid, event());
      markAcknowledged(fx.auditRoot, fx.sid, first.eventId, "fixed");
      assert.equal(
        cleanupAckedEntries(fx.auditRoot, fx.sid, {
          now: new Date("2026-05-08T00:20:00.000Z"),
        }),
        1,
      );
      const tombstone = readEventStoreState(fx.auditRoot, fx.sid).entries[0];
      assert.equal(tombstone.active, false);
      assert.equal(tombstone.archive_reason, "acked-cleanup");

      const second = appendEventIfNotDeduped(fx.auditRoot, fx.sid, event());
      assert.equal(second.appended, false);
      assert.equal(second.eventId, first.eventId);
      assert.equal(readEventStoreState(fx.auditRoot, fx.sid).entries.length, 1);
    } finally {
      rmSync(fx.root, { recursive: true, force: true });
    }
  },
);

check(
  "HES6. claimDueDeliveriesAndAdvanceCursor returns due active unacked entries only",
  () => {
    const fx = fixture();
    try {
      const first = appendEventIfNotDeduped(
        fx.auditRoot,
        fx.sid,
        event({ dedupe_key: "a", diff_key: "a" }),
      );
      const second = appendEventIfNotDeduped(
        fx.auditRoot,
        fx.sid,
        event({ dedupe_key: "b", diff_key: "b" }),
      );
      markAcknowledged(fx.auditRoot, fx.sid, second.eventId, "noted");

      const claim = claimDueDeliveriesAndAdvanceCursor(fx.auditRoot, fx.sid, {
        now: new Date("2026-05-08T00:30:00.000Z"),
        limit: 5,
      });
      assert.deepEqual(
        claim.snapshots.map((x) => x.id),
        [first.eventId],
      );
      assert.equal(
        readEventStoreState(fx.auditRoot, fx.sid).cursor.lastClaimedAt,
        "2026-05-08T00:30:00.000Z",
      );
    } finally {
      rmSync(fx.root, { recursive: true, force: true });
    }
  },
);

check(
  "HES7. markDelivered increments delivery metadata and redelivery delay",
  () => {
    const fx = fixture();
    try {
      const first = appendEventIfNotDeduped(fx.auditRoot, fx.sid, event());
      assert.equal(
        markDelivered(fx.auditRoot, fx.sid, first.eventId, {
          now: new Date("2026-05-08T00:40:00.000Z"),
          redeliverAfterMs: 60000,
        }),
        true,
      );
      const [entry] = readEventStoreState(fx.auditRoot, fx.sid).entries;
      assert.equal(entry.delivered_count, 1);
      assert.equal(entry.delivered_at, "2026-05-08T00:40:00.000Z");
      assert.equal(entry.next_redeliver_at, "2026-05-08T00:41:00.000Z");
    } finally {
      rmSync(fx.root, { recursive: true, force: true });
    }
  },
);

check("HES8. append sanitizes render data before persistence", () => {
  const fx = fixture();
  try {
    appendEventIfNotDeduped(
      fx.auditRoot,
      fx.sid,
      event({
        data: {
          file: "src/a.ts",
          line: 12,
          escape_kind: "as-any",
          snippet: `\u001b[31m${"x".repeat(220)}\`raw\`\nnext`,
          enclosing_symbol: "parse weird-name! with spaces and symbols ".repeat(
            4,
          ),
          matched_line_text: "const x = raw as any;\nconst y = 1;",
        },
      }),
    );
    const [entry] = readEventStoreState(fx.auditRoot, fx.sid).entries;
    assert.equal(entry.data.snippet.includes("\u001b"), false);
    assert.equal(entry.data.snippet.includes("`"), false);
    assert.equal(entry.data.snippet.includes("\n"), false);
    assert.ok(entry.data.snippet.length <= 160);
    assert.match(entry.data.enclosing_symbol, /^[A-Za-z0-9_.]{1,64}$/);
    assert.equal(entry.data.matched_line_text.includes("\n"), false);
  } finally {
    rmSync(fx.root, { recursive: true, force: true });
  }
});

check("HES9. malformed ledger read degrades to empty state", () => {
  const fx = fixture();
  try {
    const dir = eventStoreDir(fx.auditRoot, fx.sid);
    mkdirSync(dir, { recursive: true });
    writeFileSync(path.join(dir, "ledger.json"), "{ bad json");
    const state = readEventStoreState(fx.auditRoot, fx.sid);
    assert.equal(state.schemaVersion, "hook-event-store.v1");
    assert.deepEqual(state.entries, []);
  } finally {
    rmSync(fx.root, { recursive: true, force: true });
  }
});

check("HES10. unsafe session id reads are safe and do not traverse", () => {
  const fx = fixture();
  try {
    assert.deepEqual(readEventStoreState(fx.auditRoot, "../bad").entries, []);
    assert.equal(
      claimDueDeliveriesAndAdvanceCursor(fx.auditRoot, "../bad").snapshots
        .length,
      0,
    );
    assert.equal(
      markAcknowledged(fx.auditRoot, "../bad", "evt_bad", "fixed"),
      false,
    );
  } finally {
    rmSync(fx.root, { recursive: true, force: true });
  }
});

check("HES11. append times out on fresh lock without corrupting store", () => {
  const fx = fixture();
  try {
    const dir = eventStoreDir(fx.auditRoot, fx.sid);
    mkdirSync(path.join(dir, ".event-store.lock"), { recursive: true });
    const result = appendEventIfNotDeduped(fx.auditRoot, fx.sid, event(), {
      lockTimeoutMs: 20,
      lockStaleMs: 60000,
    });
    assert.deepEqual(result, {
      appended: false,
      eventId: null,
      reason: "lock-timeout",
    });
    assert.deepEqual(readEventStoreState(fx.auditRoot, fx.sid).entries, []);
  } finally {
    rmSync(fx.root, { recursive: true, force: true });
  }
});

check("HES12. append recovers stale event-store lock", () => {
  const fx = fixture();
  try {
    const lockDir = path.join(
      eventStoreDir(fx.auditRoot, fx.sid),
      ".event-store.lock",
    );
    mkdirSync(lockDir, { recursive: true });
    const old = new Date(0);
    utimesSync(lockDir, old, old);
    const result = appendEventIfNotDeduped(fx.auditRoot, fx.sid, event(), {
      now: new Date("2026-05-08T00:10:00.000Z"),
      lockTimeoutMs: 100,
      lockStaleMs: 1,
    });
    assert.equal(result.appended, true);
    assert.equal(readEventStoreState(fx.auditRoot, fx.sid).entries.length, 1);
    assert.equal(existsSync(lockDir), false);
  } finally {
    rmSync(fx.root, { recursive: true, force: true });
  }
});
