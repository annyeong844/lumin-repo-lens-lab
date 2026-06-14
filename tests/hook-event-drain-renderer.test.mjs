import assert from "node:assert/strict";
import { it } from "vitest";
import { existsSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import { drainDueEventReminders } from "../_lib/hook-event-drain.mjs";
import {
  appendEventIfNotDeduped,
  eventStoreDir,
  readEventStoreState,
} from "../_lib/hook-event-store.mjs";
import { renderEventReminderContext } from "../_lib/hook-event-renderer.mjs";

function check(label, fn) {
  it(label, fn);
}

function fixture() {
  const root = mkdtempSync(path.join(tmpdir(), "lrl-hook-drain-"));
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
      enclosing_symbol: "parseValue",
      matched_line_text: "const value = raw as any",
    },
    ...overrides,
  };
}

function snapshot(overrides = {}) {
  return {
    id: "evt_a",
    active: true,
    session_id: "sid_123",
    kind: "silent-new",
    severity: "warn",
    ack_required: true,
    delivery_policy: "until_ack",
    diff_key: "diff_a",
    dedupe_key: "dedupe_a",
    data: {
      file: "src/a.ts",
      line: 12,
      escape_kind: "as-any",
      snippet: "value as any",
      enclosing_symbol: "parseValue",
      matched_line_text: "const value = raw as any",
    },
    created_at: "2026-05-08T00:00:00.000Z",
    first_seen_at: "2026-05-08T00:00:00.000Z",
    last_seen_at: "2026-05-08T00:00:00.000Z",
    occurrence_count: 1,
    delivered_count: 0,
    delivered_at: null,
    next_redeliver_at: null,
    acknowledged: false,
    acknowledged_at: null,
    ack_source: null,
    archived_at: null,
    archive_reason: null,
    ...overrides,
  };
}

check("HED1. renderer returns empty output for empty input", () => {
  const rendered = renderEventReminderContext([]);
  assert.deepEqual(rendered, { text: "", eventIds: [], omittedCount: 0 });
});

check("HED2. renderer includes silent-new evidence and ACK instruction", () => {
  const rendered = renderEventReminderContext([snapshot()], { maxChars: 2048 });
  assert.match(
    rendered.text,
    /\[audit · observed in this\/previous tool batch\]/,
  );
  assert.match(rendered.text, /src\/a\.ts:12/);
  assert.match(rendered.text, /value as any/);
  assert.match(rendered.text, /Event id evt_a/);
  assert.match(
    rendered.text,
    /AUDIT_ACK <event id> <intentional\|fixed\|noted>/,
  );
  assert.deepEqual(rendered.eventIds, ["evt_a"]);
  assert.equal(rendered.omittedCount, 0);
});

check(
  "HED3. aggregate renderer uses matching-escapes wording instead of exact line assertion",
  () => {
    const rendered = renderEventReminderContext([
      snapshot({
        id: "evt_many",
        occurrence_count: 3,
        data: {
          ...snapshot().data,
          line: 40,
          enclosing_symbol: "loadConfig",
        },
      }),
    ]);
    assert.match(rendered.text, /3 matching escapes near `loadConfig`/);
    assert.doesNotMatch(rendered.text, /src\/a\.ts:40 — silent-new/);
  },
);

check(
  "HED4. renderer budget omits whole event blocks and reports omittedCount",
  () => {
    const rendered = renderEventReminderContext(
      [
        snapshot({ id: "evt_one", created_at: "2026-05-08T00:00:00.000Z" }),
        snapshot({
          id: "evt_two",
          created_at: "2026-05-08T00:00:01.000Z",
          data: {
            ...snapshot().data,
            file: "src/b.ts",
            line: 20,
            snippet: "other as any",
          },
        }),
      ],
      { maxChars: 440 },
    );
    assert.ok(rendered.text.length <= 520, rendered.text);
    assert.match(rendered.text, /evt_one/);
    assert.doesNotMatch(rendered.text, /evt_two/);
    assert.deepEqual(rendered.eventIds, ["evt_one"]);
    assert.equal(rendered.omittedCount, 1);
  },
);

check(
  "HED5. drain missing store emits nothing and creates no event-store directory",
  () => {
    const fx = fixture();
    try {
      const result = drainDueEventReminders(fx.auditRoot, fx.sid);
      assert.deepEqual(result, {
        emitted: false,
        output: null,
        eventIds: [],
        omittedCount: 0,
      });
      assert.equal(
        existsSync(path.join(fx.auditRoot, "sessions", fx.sid, "event-store")),
        false,
      );
    } finally {
      rmSync(fx.root, { recursive: true, force: true });
    }
  },
);

check(
  "HED6. drain due event returns hook output and marks event delivered before return",
  () => {
    const fx = fixture();
    try {
      const appended = appendEventIfNotDeduped(fx.auditRoot, fx.sid, event(), {
        now: new Date("2026-05-08T00:00:00.000Z"),
      });
      const result = drainDueEventReminders(fx.auditRoot, fx.sid, {
        hookEventName: "PreToolUse",
        now: new Date("2026-05-08T00:01:00.000Z"),
        redeliverAfterMs: 60000,
      });
      assert.equal(result.emitted, true);
      assert.equal(
        result.output.hookSpecificOutput.hookEventName,
        "PreToolUse",
      );
      assert.match(
        result.output.hookSpecificOutput.additionalContext,
        new RegExp(appended.eventId),
      );
      assert.deepEqual(result.eventIds, [appended.eventId]);

      const [entry] = readEventStoreState(fx.auditRoot, fx.sid).entries;
      assert.equal(entry.delivered_count, 1);
      assert.equal(entry.delivered_at, "2026-05-08T00:01:00.000Z");
      assert.equal(entry.next_redeliver_at, "2026-05-08T00:02:00.000Z");
    } finally {
      rmSync(fx.root, { recursive: true, force: true });
    }
  },
);

check("HED7. second drain before next_redeliver_at emits nothing", () => {
  const fx = fixture();
  try {
    appendEventIfNotDeduped(fx.auditRoot, fx.sid, event(), {
      now: new Date("2026-05-08T00:00:00.000Z"),
    });
    const first = drainDueEventReminders(fx.auditRoot, fx.sid, {
      now: new Date("2026-05-08T00:01:00.000Z"),
      redeliverAfterMs: 60000,
    });
    const second = drainDueEventReminders(fx.auditRoot, fx.sid, {
      now: new Date("2026-05-08T00:01:30.000Z"),
      redeliverAfterMs: 60000,
    });
    assert.equal(first.emitted, true);
    assert.deepEqual(second, {
      emitted: false,
      output: null,
      eventIds: [],
      omittedCount: 0,
    });
    const [entry] = readEventStoreState(fx.auditRoot, fx.sid).entries;
    assert.equal(entry.delivered_count, 1);
  } finally {
    rmSync(fx.root, { recursive: true, force: true });
  }
});
