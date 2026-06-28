import assert from "node:assert/strict";
import { it } from "vitest";

import {
  isSafeId,
  safeSessionId,
  safeToolUseId,
} from "../_lib/hook-id-safety.mjs";

function check(label, fn) {
  it(label, fn);
}

check("HID1. safe ids allow only compact opaque tokens", () => {
  assert.equal(isSafeId("abc_123-XYZ"), true);
  assert.equal(isSafeId("a".repeat(128)), true);
  assert.equal(isSafeId("a".repeat(129)), false);
  assert.equal(isSafeId("../bad"), false);
  assert.equal(isSafeId("bad space"), false);
  assert.equal(isSafeId(null), false);
});

check(
  "HID2. session id uses safe explicit id or deterministic transcript fallback",
  () => {
    assert.equal(safeSessionId({ session_id: "sid_123" }), "sid_123");
    assert.match(
      safeSessionId({ transcript_path: "/tmp/transcript.jsonl" }),
      /^sid_[a-f0-9]{16}$/,
    );
    assert.equal(safeSessionId({ session_id: "../bad" }), "default-session");
  },
);

check("HID3. tool use id uses safe explicit id", () => {
  assert.equal(safeToolUseId({ tool_use_id: "tool_123" }), "tool_123");
  assert.notEqual(
    safeToolUseId({ tool_use_id: "../bad", tool_name: "Read" }),
    "../bad",
  );
});

check(
  "HID4. fallback tool id is deterministic and ignores hook event metadata",
  () => {
    const payloadA = {
      hook_event_name: "PreToolUse",
      call_index: 1,
      tool_name: "Write",
      tool_input: {
        file_path: "src/a.ts",
        content: 'const secret = "do-not-leak";\n',
      },
    };
    const payloadB = {
      hook_event_name: "PostToolBatch",
      call_index: 99,
      tool_name: "Write",
      tool_input: {
        file_path: "src/a.ts",
        content: 'const secret = "do-not-leak";\n',
      },
    };
    assert.equal(safeToolUseId(payloadA), safeToolUseId(payloadB));
    assert.match(safeToolUseId(payloadA), /^tool_[a-f0-9]{16}$/);
  },
);

check(
  "HID5. fallback tool id hashes content-bearing fields without raw content",
  () => {
    const payload = {
      tool_name: "Edit",
      tool_input: {
        file_path: "src/a.ts",
        old_string: "old secret raw text",
        new_string: "new secret raw text",
      },
    };
    const id = safeToolUseId(payload);
    assert.match(id, /^tool_[a-f0-9]{16}$/);
    assert.equal(id.includes("old secret raw text"), false);
    assert.equal(id.includes("new secret raw text"), false);

    const changed = safeToolUseId({
      ...payload,
      tool_input: {
        ...payload.tool_input,
        new_string: "different raw text",
      },
    });
    assert.notEqual(id, changed);
  },
);
