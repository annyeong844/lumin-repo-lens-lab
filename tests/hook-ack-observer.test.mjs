import assert from "node:assert/strict";
import { it } from "vitest";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import {
  observeStopAcknowledgements,
  parseAuditAckLines,
} from "../_lib/hook-ack-observer.mjs";
import {
  appendEventIfNotDeduped,
  readEventStoreState,
} from "../_lib/hook-event-store.mjs";

function check(label, fn) {
  it(label, fn);
}

function fixture() {
  const root = mkdtempSync(path.join(tmpdir(), "lrl-hook-ack-"));
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

check("HAO1. parseAuditAckLines parses a valid standalone ACK line", () => {
  assert.deepEqual(
    parseAuditAckLines("ok\nAUDIT_ACK evt_abc123 fixed\nthanks"),
    [{ eventId: "evt_abc123", ackSource: "fixed", line: 2 }],
  );
});

check("HAO2. parseAuditAckLines ignores invalid ACK intent", () => {
  assert.deepEqual(parseAuditAckLines("AUDIT_ACK evt_abc123 maybe"), []);
});

check("HAO3. parseAuditAckLines ignores ACK inside closed fenced code", () => {
  const text = [
    "```",
    "AUDIT_ACK evt_abc123 fixed",
    "```",
    "AUDIT_ACK evt_real noted",
  ].join("\n");
  assert.deepEqual(parseAuditAckLines(text), [
    { eventId: "evt_real", ackSource: "noted", line: 4 },
  ]);
});

check(
  "HAO4. parseAuditAckLines ignores ACK inside unclosed fence through EOF",
  () => {
    assert.deepEqual(
      parseAuditAckLines("before\n```md\nAUDIT_ACK evt_abc123 fixed"),
      [],
    );
  },
);

check("HAO5. parseAuditAckLines ignores ACK inside inline backticks", () => {
  assert.deepEqual(parseAuditAckLines("`AUDIT_ACK evt_abc123 fixed`"), []);
});

check("HAO6. parseAuditAckLines ignores ACK inside indented code block", () => {
  assert.deepEqual(parseAuditAckLines("    AUDIT_ACK evt_abc123 fixed"), []);
});

check(
  "HAO7. parseAuditAckLines ignores ACK inside leading-space blockquote",
  () => {
    assert.deepEqual(parseAuditAckLines("  > AUDIT_ACK evt_abc123 fixed"), []);
  },
);

check(
  "HAO8. observeStopAcknowledgements marks matching event acknowledged",
  () => {
    const fx = fixture();
    try {
      const appended = appendEventIfNotDeduped(fx.auditRoot, fx.sid, event(), {
        now: new Date("2026-05-08T00:00:00.000Z"),
      });
      const result = observeStopAcknowledgements(
        fx.auditRoot,
        fx.sid,
        { last_assistant_message: `AUDIT_ACK ${appended.eventId} intentional` },
        { now: new Date("2026-05-08T00:01:00.000Z") },
      );
      assert.deepEqual(result, {
        observed: 1,
        acknowledged: 1,
        ignored: 0,
        eventIds: [appended.eventId],
      });
      const [entry] = readEventStoreState(fx.auditRoot, fx.sid).entries;
      assert.equal(entry.acknowledged, true);
      assert.equal(entry.ack_source, "intentional");
      assert.equal(entry.acknowledged_at, "2026-05-08T00:01:00.000Z");
    } finally {
      rmSync(fx.root, { recursive: true, force: true });
    }
  },
);

check(
  "HAO9. observer prefers last_assistant_message over transcriptText fallback",
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
      const result = observeStopAcknowledgements(
        fx.auditRoot,
        fx.sid,
        { last_assistant_message: `AUDIT_ACK ${first.eventId} fixed` },
        { transcriptText: `AUDIT_ACK ${second.eventId} fixed` },
      );
      assert.deepEqual(result.eventIds, [first.eventId]);
      const entries = readEventStoreState(fx.auditRoot, fx.sid).entries;
      assert.equal(
        entries.find((entry) => entry.id === first.eventId).acknowledged,
        true,
      );
      assert.equal(
        entries.find((entry) => entry.id === second.eventId).acknowledged,
        false,
      );
    } finally {
      rmSync(fx.root, { recursive: true, force: true });
    }
  },
);

check(
  "HAO10. observer uses transcriptText fallback when last_assistant_message is absent",
  () => {
    const fx = fixture();
    try {
      const appended = appendEventIfNotDeduped(fx.auditRoot, fx.sid, event());
      const result = observeStopAcknowledgements(
        fx.auditRoot,
        fx.sid,
        {},
        { transcriptText: `AUDIT_ACK ${appended.eventId} noted` },
      );
      assert.equal(result.acknowledged, 1);
      assert.equal(
        readEventStoreState(fx.auditRoot, fx.sid).entries[0].ack_source,
        "noted",
      );
    } finally {
      rmSync(fx.root, { recursive: true, force: true });
    }
  },
);

check("HAO11. observer handles unsafe session id without acknowledging", () => {
  const fx = fixture();
  try {
    const result = observeStopAcknowledgements(fx.auditRoot, "../bad", {
      last_assistant_message: "AUDIT_ACK evt_abc123 fixed",
    });
    assert.deepEqual(result, {
      observed: 1,
      acknowledged: 0,
      ignored: 1,
      eventIds: [],
    });
  } finally {
    rmSync(fx.root, { recursive: true, force: true });
  }
});
