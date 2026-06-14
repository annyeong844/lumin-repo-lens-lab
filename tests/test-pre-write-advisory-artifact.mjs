// Tests for _lib/pre-write-artifact.mjs — P1-1 step 5.4.
//
// Pinning rules from docs/history/phases/p1/p1-1.md §4.5 + §5.4:
//   - generateInvocationId() shape: `YYYY-MM-DDTHH-mm-ssZ-<6-char-random>`
//   - hashIntent(intent) is sha256 hex; normalization sorts keys recursively.
//   - Calling with same intent twice → same hash (deterministic).
//   - writeAdvisory writes BOTH paths: latest.json + <invocationId>.json,
//     content identical byte-for-byte.
//   - capabilities block copied from symbols.meta.supports; absent →
//     capabilities: null AND failures[] includes {kind: 'capabilities-missing'}.
//   - Atomic write: temp-file + rename so a crash mid-write leaves no
//     partial file at the target path.

import { existsSync, readFileSync, mkdtempSync, rmSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import {
  generateInvocationId,
  hashIntent,
  writeAdvisory,
} from '../_lib/pre-write-artifact.mjs';

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

// ═══ generateInvocationId ═══

{
  const id = generateInvocationId();
  assert('T1. invocationId is a string',
    typeof id === 'string');
  assert('T1b. invocationId matches YYYY-MM-DDTHH-mm-ssZ-<6-char-random>',
    /^\d{4}-\d{2}-\d{2}T\d{2}-\d{2}-\d{2}Z-[a-z0-9]{6}$/.test(id),
    `id=${id}`);
}

{
  // Two back-to-back generations produce different IDs thanks to the
  // random suffix (even if the timestamp is identical).
  const a = generateInvocationId();
  const b = generateInvocationId();
  assert('T2. two invocationIds are different (random suffix)',
    a !== b, `a=${a}, b=${b}`);
}

// ═══ hashIntent — determinism + normalization ═══

{
  const intent = {
    names: ['formatDate'],
    shapes: [],
    files: [],
    dependencies: [],
    plannedTypeEscapes: [],
  };
  const h1 = hashIntent(intent);
  const h2 = hashIntent(intent);
  assert('T3. hashIntent is deterministic (same input → same hash)',
    h1 === h2);
  assert('T3b. hashIntent returns 64-char lowercase hex',
    /^[a-f0-9]{64}$/.test(h1), `h1=${h1}`);
}

{
  // Key-order independence: same content with keys in different order
  // must produce the same hash.
  const a = {
    names: ['x'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [],
  };
  const b = {
    plannedTypeEscapes: [], dependencies: [], files: [], shapes: [], names: ['x'],
  };
  assert('T4. hashIntent ignores top-level key order',
    hashIntent(a) === hashIntent(b));
}

{
  // Nested key order also ignored.
  const a = {
    names: [], shapes: [], files: [], dependencies: [],
    plannedTypeEscapes: [
      { escapeKind: 'as-any', locationHint: 'x', reason: 'y' },
    ],
  };
  const b = {
    names: [], shapes: [], files: [], dependencies: [],
    plannedTypeEscapes: [
      { reason: 'y', locationHint: 'x', escapeKind: 'as-any' },
    ],
  };
  assert('T5. hashIntent ignores nested key order',
    hashIntent(a) === hashIntent(b));
}

{
  // Different content → different hash.
  const a = { names: ['foo'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] };
  const b = { names: ['bar'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] };
  assert('T6. hashIntent differs when content differs',
    hashIntent(a) !== hashIntent(b));
}

// ═══ writeAdvisory — dual write + content integrity ═══

{
  const dir = mkdtempSync(path.join(tmpdir(), 'pw-artifact-'));
  try {
    const invocationId = '2026-04-20T12-30-00Z-abc123';
    const advisory = {
      invocationId,
      intentHash: 'dummy-hash',
      intent: { names: ['x'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
      lookups: [],
      boundaryChecks: [],
      drift: [],
      capabilities: { anyContamination: false, identityFanIn: true, reExportRecords: 'file-level' },
      failures: [],
    };
    writeAdvisory(dir, advisory);

    const latest = path.join(dir, 'pre-write-advisory.latest.json');
    const specific = path.join(dir, `pre-write-advisory.${invocationId}.json`);

    assert('T7. latest.json exists', existsSync(latest));
    assert('T7b. invocation-specific json exists', existsSync(specific));

    const latestText = readFileSync(latest, 'utf8');
    const specificText = readFileSync(specific, 'utf8');
    assert('T7c. both files contain identical bytes', latestText === specificText);

    const parsed = JSON.parse(latestText);
    assert('T7d. parsed invocationId matches', parsed.invocationId === invocationId);
    assert('T7e. parsed capabilities copied through',
      parsed.capabilities?.identityFanIn === true);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

// ═══ Atomic write — no temp file left behind ═══

{
  const dir = mkdtempSync(path.join(tmpdir(), 'pw-atomic-'));
  try {
    const invocationId = '2026-04-20T12-31-00Z-def456';
    writeAdvisory(dir, {
      invocationId,
      intentHash: 'h',
      intent: {},
      lookups: [],
      boundaryChecks: [],
      drift: [],
      capabilities: null,
      failures: [],
    });
    const names = readdirSync(dir);
    const tempLeftovers = names.filter((n) => n.startsWith('.') || n.endsWith('.tmp') || n.includes('.tmp.'));
    assert('T8. no temporary files left behind after write',
      tempLeftovers.length === 0,
      `leftovers=${tempLeftovers.join(', ')}`);
    assert('T8b. exactly 2 advisory files written (latest + invocationId)',
      names.filter((n) => n.startsWith('pre-write-advisory.')).length === 2,
      `files=${names.join(', ')}`);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

// ═══ Repeated writes update latest; preserve specific files ═══

{
  const dir = mkdtempSync(path.join(tmpdir(), 'pw-multi-'));
  try {
    const id1 = '2026-04-20T12-32-00Z-aaa111';
    const id2 = '2026-04-20T12-33-00Z-bbb222';
    writeAdvisory(dir, {
      invocationId: id1, intentHash: 'h1', intent: { names: ['first'] },
      lookups: [], boundaryChecks: [], drift: [], capabilities: null, failures: [],
    });
    writeAdvisory(dir, {
      invocationId: id2, intentHash: 'h2', intent: { names: ['second'] },
      lookups: [], boundaryChecks: [], drift: [], capabilities: null, failures: [],
    });

    const latest = JSON.parse(readFileSync(path.join(dir, 'pre-write-advisory.latest.json'), 'utf8'));
    assert('T9. latest.json reflects the MOST RECENT invocation',
      latest.invocationId === id2 && latest.intentHash === 'h2');

    // Both invocation-specific files remain on disk.
    assert('T9b. first invocation-specific file preserved',
      existsSync(path.join(dir, `pre-write-advisory.${id1}.json`)));
    assert('T9c. second invocation-specific file present',
      existsSync(path.join(dir, `pre-write-advisory.${id2}.json`)));
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

// ═══ capabilities-missing failure entry ═══

{
  const dir = mkdtempSync(path.join(tmpdir(), 'pw-caps-missing-'));
  try {
    const invocationId = '2026-04-20T12-34-00Z-ccc333';
    const advisory = {
      invocationId,
      intentHash: 'h',
      intent: {},
      lookups: [],
      boundaryChecks: [],
      drift: [],
      capabilities: null,   // caller explicitly marks absent
      failures: [{ kind: 'capabilities-missing', reason: 'symbols.meta.supports not found in symbols.json' }],
    };
    writeAdvisory(dir, advisory);
    const parsed = JSON.parse(readFileSync(path.join(dir, 'pre-write-advisory.latest.json'), 'utf8'));
    assert('T10. capabilities:null round-trips through write',
      parsed.capabilities === null);
    assert('T10b. failures[] with capabilities-missing kind preserved',
      Array.isArray(parsed.failures) &&
      parsed.failures.some((f) => f.kind === 'capabilities-missing'));
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

// ═══ hashIntent vs. writeAdvisory round-trip ═══

{
  const dir = mkdtempSync(path.join(tmpdir(), 'pw-hash-roundtrip-'));
  try {
    const intent = {
      names: ['formatDate'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [],
    };
    const intentHash = hashIntent(intent);
    const invocationId = '2026-04-20T12-35-00Z-ddd444';
    writeAdvisory(dir, {
      invocationId, intentHash, intent,
      lookups: [], boundaryChecks: [], drift: [], capabilities: null, failures: [],
    });
    const parsed = JSON.parse(readFileSync(path.join(dir, 'pre-write-advisory.latest.json'), 'utf8'));
    assert('T11. intentHash written to artifact equals hashIntent(intent) directly',
      parsed.intentHash === intentHash);
    assert('T11b. re-hashing the round-tripped intent yields the same hash',
      hashIntent(parsed.intent) === intentHash);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
