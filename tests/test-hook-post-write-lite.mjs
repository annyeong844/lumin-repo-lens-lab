import assert from 'node:assert/strict';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';

import { processPostWriteLite } from '../_lib/hook-post-write-lite.mjs';
import {
  appendEventIfNotDeduped,
  eventStoreDir,
  readEventStoreState,
} from '../_lib/hook-event-store.mjs';
import {
  capturePreimage,
  preimagePath,
  readPreimage,
} from '../_lib/hook-preimage-store.mjs';
import { safeRepoPathForToolInput } from '../_lib/hook-path-safety.mjs';

let passed = 0;
let failed = 0;

function check(label, fn) {
  try {
    fn();
    passed++;
    console.log(`  PASS  ${label}`);
  } catch (error) {
    failed++;
    console.log(`  FAIL  ${label}\n        ${error?.message ?? error}`);
  }
}

function fixture() {
  const root = mkdtempSync(path.join(tmpdir(), 'lrl-hook-pwl-'));
  mkdirSync(path.join(root, 'src'), { recursive: true });
  writeFileSync(path.join(root, 'package.json'), '{"type":"module"}\n');
  return {
    root,
    auditRoot: path.join(root, '.audit'),
    sid: 'sid_123',
  };
}

function writeSource(root, rel, src) {
  const file = path.join(root, rel);
  mkdirSync(path.dirname(file), { recursive: true });
  writeFileSync(file, src);
  return file;
}

function captureForTool(fx, tid, rel) {
  const safe = safeRepoPathForToolInput(fx.root, rel);
  return capturePreimage({
    auditRoot: fx.auditRoot,
    sid: fx.sid,
    tid,
    safe,
    now: new Date('2026-05-08T00:00:00.000Z'),
  });
}

function editCall(tid, rel, extra = {}) {
  return {
    tool_name: 'Edit',
    tool_use_id: tid,
    tool_input: {
      file_path: rel,
      old_string: 'old',
      new_string: 'new',
    },
    ...extra,
  };
}

function payload(fx, toolCalls) {
  return {
    cwd: fx.root,
    session_id: fx.sid,
    tool_calls: toolCalls,
  };
}

check('HPWL1. capturePreimage stores typeEscapes without raw source text leakage', () => {
  const fx = fixture();
  try {
    writeSource(fx.root, 'src/a.ts', 'export const value = raw as any;\n');
    const record = captureForTool(fx, 'tool_a', 'src/a.ts');
    assert.equal(record.fingerprint.typeEscapes.length, 1);
    assert.equal(record.fingerprint.typeEscapes[0].escapeKind, 'as-any');
    assert.equal(record.fingerprint.parseError, null);
    const raw = readFileSync(preimagePath(fx.auditRoot, fx.sid, 'tool_a'), 'utf8');
    assert.equal(raw.includes('export const value = raw as any'), false);
  } finally {
    rmSync(fx.root, { recursive: true, force: true });
  }
});

check('HPWL2. processPostWriteLite appends silent-new event for new escape', () => {
  const fx = fixture();
  try {
    writeSource(fx.root, 'src/a.ts', 'export const value = raw;\n');
    captureForTool(fx, 'tool_a', 'src/a.ts');
    writeSource(fx.root, 'src/a.ts', 'export const value = raw as any;\n');

    const result = processPostWriteLite(payload(fx, [editCall('tool_a', 'src/a.ts')]), {
      now: new Date('2026-05-08T00:01:00.000Z'),
      redeliverAfterMs: 60000,
    });
    assert.equal(result.processedFiles, 1);
    assert.equal(result.appendedEventIds.length, 1);
    assert.equal(result.preimageIncompleteFiles.length, 0);
    assert.match(result.output.hookSpecificOutput.additionalContext, /AUDIT_ACK <event id>/);

    const [entry] = readEventStoreState(fx.auditRoot, fx.sid).entries;
    assert.equal(entry.kind, 'silent-new');
    assert.equal(entry.delivery_policy, 'until_ack');
    assert.equal(entry.data.file, 'src/a.ts');
    assert.equal(entry.data.escape_kind, 'as-any');
    assert.equal(entry.data.snippet, 'raw as any');
    assert.equal(entry.data.matched_line_text, 'raw as any');
    assert.match(entry.dedupe_key, /^sha256:/);
    assert.equal(entry.delivered_count, 1);
    assert.equal(readPreimage(fx.auditRoot, fx.sid, 'tool_a'), null);
  } finally {
    rmSync(fx.root, { recursive: true, force: true });
  }
});

check('HPWL3. duplicate occurrence key merges into existing event count', () => {
  const fx = fixture();
  try {
    writeSource(fx.root, 'src/a.ts', 'export function f(raw) { return raw as any; }\n');
    captureForTool(fx, 'tool_a', 'src/a.ts');
    const beforeKey = readPreimage(fx.auditRoot, fx.sid, 'tool_a').fingerprint.typeEscapes[0].occurrenceKey;
    appendEventIfNotDeduped(fx.auditRoot, fx.sid, {
      kind: 'silent-new',
      severity: 'warn',
      ack_required: true,
      delivery_policy: 'until_ack',
      diff_key: beforeKey,
      dedupe_key: beforeKey,
      occurrence_delta: 2,
      data: {
        file: 'src/a.ts',
        line: 1,
        escape_kind: 'as-any',
        snippet: 'raw as any',
        enclosing_symbol: 'f',
        matched_line_text: 'raw as any',
      },
    });

    writeSource(fx.root, 'src/a.ts', 'export function f(raw) { return raw as any; }\nexport function g(raw) { return raw as any; }\n');
    const result = processPostWriteLite(payload(fx, [editCall('tool_a', 'src/a.ts')]), {
      maxChars: 0,
    });
    assert.equal(result.appendedEventIds.length, 1);
    const entries = readEventStoreState(fx.auditRoot, fx.sid).entries;
    assert.equal(entries.length, 2);
    assert.equal(entries.find((entry) => entry.dedupe_key === beforeKey).occurrence_count, 2);
  } finally {
    rmSync(fx.root, { recursive: true, force: true });
  }
});

check('HPWL4. same-file group uses first preimage and cleans all preimages', () => {
  const fx = fixture();
  try {
    writeSource(fx.root, 'src/a.ts', 'export const value = raw;\n');
    captureForTool(fx, 'tool_first', 'src/a.ts');
    writeSource(fx.root, 'src/a.ts', 'export const value = raw as any;\n');
    captureForTool(fx, 'tool_second', 'src/a.ts');

    const result = processPostWriteLite(payload(fx, [
      editCall('tool_first', 'src/a.ts'),
      editCall('tool_second', 'src/a.ts'),
    ]), {
      maxChars: 0,
    });
    assert.equal(result.appendedEventIds.length, 1);
    assert.equal(readPreimage(fx.auditRoot, fx.sid, 'tool_first'), null);
    assert.equal(readPreimage(fx.auditRoot, fx.sid, 'tool_second'), null);
    assert.equal(readEventStoreState(fx.auditRoot, fx.sid).entries.length, 1);
  } finally {
    rmSync(fx.root, { recursive: true, force: true });
  }
});

check('HPWL5. missing first preimage over-warns from empty baseline', () => {
  const fx = fixture();
  try {
    writeSource(fx.root, 'src/a.ts', 'export const value = raw as any;\n');
    const result = processPostWriteLite(payload(fx, [editCall('tool_missing', 'src/a.ts')]), {
      maxChars: 0,
    });
    assert.deepEqual(result.preimageIncompleteFiles, ['src/a.ts']);
    assert.equal(result.appendedEventIds.length, 1);
  } finally {
    rmSync(fx.root, { recursive: true, force: true });
  }
});

check('HPWL6. non-mutating batch creates no event-store directory', () => {
  const fx = fixture();
  try {
    writeSource(fx.root, 'src/a.ts', 'export const value = raw as any;\n');
    const result = processPostWriteLite(payload(fx, [{
      tool_name: 'Read',
      tool_use_id: 'tool_read',
      tool_input: { file_path: 'src/a.ts' },
    }]));
    assert.equal(result.processedFiles, 0);
    assert.deepEqual(result.appendedEventIds, []);
    assert.equal(existsSync(eventStoreDir(fx.auditRoot, fx.sid)), false);
  } finally {
    rmSync(fx.root, { recursive: true, force: true });
  }
});

if (failed) {
  console.error(`\n${passed} passed, ${failed} failed`);
  process.exit(1);
}

console.log(`\n${passed} passed, ${failed} failed`);
