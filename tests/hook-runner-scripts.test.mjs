import assert from "node:assert/strict";
import { it } from "vitest";
import { spawnSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  appendEventIfNotDeduped,
  readEventStoreState,
} from "../_lib/hook-event-store.mjs";
import { preimagePath, readPreimage } from "../_lib/hook-preimage-store.mjs";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const NODE = process.execPath;

function check(label, fn) {
  it(label, fn);
}

function fixture() {
  const root = mkdtempSync(path.join(tmpdir(), "lrl-hook-runner-"));
  mkdirSync(path.join(root, "src"), { recursive: true });
  writeFileSync(path.join(root, "package.json"), '{"type":"module"}\n');
  return {
    root,
    auditRoot: path.join(root, ".audit"),
    sid: "sid_runner",
  };
}

function writeSource(root, rel, text) {
  const file = path.join(root, rel);
  mkdirSync(path.dirname(file), { recursive: true });
  writeFileSync(file, text);
  return file;
}

function runHook(scriptRel, payloadOrRaw, { cwd = ROOT } = {}) {
  const input =
    typeof payloadOrRaw === "string"
      ? payloadOrRaw
      : JSON.stringify(payloadOrRaw);
  return spawnSync(NODE, [path.join(ROOT, scriptRel)], {
    cwd,
    input,
    encoding: "utf8",
  });
}

function parseHookJson(stdout) {
  assert.notEqual(stdout.trim(), "", "expected hook JSON output");
  return JSON.parse(stdout);
}

function preToolPayload(fx, toolUseId = "tool_runner") {
  return {
    cwd: fx.root,
    session_id: fx.sid,
    tool_name: "Edit",
    tool_use_id: toolUseId,
    tool_input: {
      file_path: "src/a.ts",
      old_string: "old",
      new_string: "new",
    },
  };
}

function postBatchPayload(fx, toolUseId = "tool_runner") {
  return {
    cwd: fx.root,
    session_id: fx.sid,
    tool_calls: [
      {
        tool_name: "Edit",
        tool_use_id: toolUseId,
        tool_input: {
          file_path: "src/a.ts",
          old_string: "old",
          new_string: "new",
        },
      },
    ],
  };
}

check("HRS1. hook runners exit 0 and stay quiet on malformed stdin", () => {
  const result = runHook("hooks/pre-tool-use.mjs", "{ bad json");
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.stdout, "");
});

check(
  "HRS2. pre-tool-use captures preimage for a mutating Edit payload",
  () => {
    const fx = fixture();
    try {
      writeSource(fx.root, "src/a.ts", "export const value = raw;\n");
      const result = runHook("hooks/pre-tool-use.mjs", preToolPayload(fx), {
        cwd: fx.root,
      });
      assert.equal(result.status, 0, result.stderr);
      assert.equal(result.stdout, "");
      const record = readPreimage(fx.auditRoot, fx.sid, "tool_runner");
      assert.equal(record?.repoRel, "src/a.ts");
      assert.equal(record?.absent, false);
      assert.equal(
        existsSync(preimagePath(fx.auditRoot, fx.sid, "tool_runner")),
        true,
      );
    } finally {
      rmSync(fx.root, { recursive: true, force: true });
    }
  },
);

check(
  "HRS3. post-tool-batch emits a delivered silent-new reminder from captured preimage",
  () => {
    const fx = fixture();
    try {
      writeSource(fx.root, "src/a.ts", "export const value = raw;\n");
      const pre = runHook("hooks/pre-tool-use.mjs", preToolPayload(fx), {
        cwd: fx.root,
      });
      assert.equal(pre.status, 0, pre.stderr);

      writeSource(fx.root, "src/a.ts", "export const value = raw as any;\n");
      const post = runHook("hooks/post-tool-batch.mjs", postBatchPayload(fx), {
        cwd: fx.root,
      });
      assert.equal(post.status, 0, post.stderr);
      const hookOutput = parseHookJson(post.stdout);
      assert.equal(
        hookOutput.hookSpecificOutput.hookEventName,
        "PostToolBatch",
      );
      assert.match(
        hookOutput.hookSpecificOutput.additionalContext,
        /AUDIT_ACK <event id>/,
      );

      const state = readEventStoreState(fx.auditRoot, fx.sid);
      assert.equal(state.entries.length, 1);
      assert.equal(state.entries[0].kind, "silent-new");
      assert.equal(state.entries[0].acknowledged, false);
      assert.equal(state.entries[0].delivered_count, 1);
      assert.equal(readPreimage(fx.auditRoot, fx.sid, "tool_runner"), null);
    } finally {
      rmSync(fx.root, { recursive: true, force: true });
    }
  },
);

check("HRS4. stop runner acknowledges AUDIT_ACK lines", () => {
  const fx = fixture();
  try {
    writeSource(fx.root, "src/a.ts", "export const value = raw;\n");
    runHook("hooks/pre-tool-use.mjs", preToolPayload(fx), { cwd: fx.root });
    writeSource(fx.root, "src/a.ts", "export const value = raw as any;\n");
    runHook("hooks/post-tool-batch.mjs", postBatchPayload(fx), {
      cwd: fx.root,
    });

    const eventId = readEventStoreState(fx.auditRoot, fx.sid).entries[0].id;
    const stop = runHook(
      "hooks/stop.mjs",
      {
        cwd: fx.root,
        session_id: fx.sid,
        last_assistant_message: `AUDIT_ACK ${eventId} fixed\n`,
      },
      { cwd: fx.root },
    );
    assert.equal(stop.status, 0, stop.stderr);
    assert.equal(stop.stdout, "");

    const entry = readEventStoreState(fx.auditRoot, fx.sid).entries[0];
    assert.equal(entry.acknowledged, true);
    assert.equal(entry.ack_source, "fixed");
  } finally {
    rmSync(fx.root, { recursive: true, force: true });
  }
});

check("HRS5. user-prompt-submit drains already due events", () => {
  const fx = fixture();
  try {
    appendEventIfNotDeduped(
      fx.auditRoot,
      fx.sid,
      {
        kind: "silent-new",
        severity: "warn",
        ack_required: true,
        delivery_policy: "until_ack",
        diff_key:
          "sha256:1111111111111111111111111111111111111111111111111111111111111111",
        dedupe_key:
          "sha256:1111111111111111111111111111111111111111111111111111111111111111",
        occurrence_delta: 1,
        data: {
          file: "src/a.ts",
          line: 1,
          escape_kind: "as-any",
          snippet: "raw as any",
          enclosing_symbol: "top-level",
          matched_line_text: "raw as any",
        },
      },
      {
        now: new Date("2026-05-08T00:00:00.000Z"),
      },
    );

    const prompt = runHook(
      "hooks/user-prompt-submit.mjs",
      {
        cwd: fx.root,
        session_id: fx.sid,
      },
      { cwd: fx.root },
    );
    assert.equal(prompt.status, 0, prompt.stderr);
    const hookOutput = parseHookJson(prompt.stdout);
    assert.equal(
      hookOutput.hookSpecificOutput.hookEventName,
      "UserPromptSubmit",
    );
    assert.match(
      hookOutput.hookSpecificOutput.additionalContext,
      /AUDIT_ACK <event id>/,
    );
    assert.equal(
      readEventStoreState(fx.auditRoot, fx.sid).entries[0].delivered_count,
      1,
    );
  } finally {
    rmSync(fx.root, { recursive: true, force: true });
  }
});
