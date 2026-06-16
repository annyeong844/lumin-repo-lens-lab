import { describe, expect, it } from 'vitest';

import {
  sortRustHealthArtifact,
  summarizeRustHealthArtifact,
  validateRustHealthFinalArtifact,
  validateRustHealthSidecarArtifact,
} from '../_lib/rust-source-health-schema.mjs';

function artifact(overrides = {}) {
  return {
    schemaVersion: 1,
    meta: {
      producer: 'rust-source-health',
      mode: 'syntax-only',
      generated: '2026-06-16T10:00:00.000Z',
      sidecar: {
        sourceCommit: 'abc123',
        binarySha256: `sha256:${'b'.repeat(64)}`,
      },
      input: {
        pathPolicy: { include: ['**/*.rs'], exclude: ['**/target/**', '**/vendor/**'] },
      },
      runtime: { threadCount: 2, workerStackBytes: 16777216 },
      limits: ['syntax-only', 'no-type-info', 'no-trait-solving', 'no-borrow-check'],
      policy: {
        version: 'm6-rust-source-health-syntax-v1',
        thresholds: { maxFunctionLines: 80, maxImplLines: 200 },
      },
      parser: {
        kind: 'ra_ap_syntax',
        version: '0.0.337',
        editionPolicy: 'fixed',
        edition: '2021',
        editionSource: 'm6-policy-default',
      },
    },
    summary: {
      files: 1,
      skippedFiles: 1,
      parseErrorFiles: 0,
      parseErrors: 0,
      functions: 2,
      unsafeBlocks: 1,
      unsafeFunctions: 1,
      signals: 2,
      signalsByKind: { 'clone-call': 1, 'unwrap-call': 1 },
    },
    skippedFiles: [{ path: 'src/bad.rs', reason: 'invalid-utf8' }],
    files: {
      'src/lib.rs': {
        sha256: `sha256:${'a'.repeat(64)}`,
        facts: {
          items: 3,
          functions: 2,
          maxFunctionLines: 12,
          unsafeBlocks: 1,
          unsafeFunctions: 1,
        },
        signals: [
          {
            kind: 'unwrap-call',
            severity: 'review',
            claim: 'syntax-only',
            location: {
              line: 2,
              column: 3,
              endLine: 2,
              endColumn: 11,
              byteStart: 20,
              byteEnd: 28,
            },
          },
          {
            kind: 'clone-call',
            severity: 'review',
            claim: 'syntax-only',
            location: {
              line: 3,
              column: 3,
              endLine: 3,
              endColumn: 10,
              byteStart: 40,
              byteEnd: 47,
            },
          },
        ],
        parse: { ok: true, errors: [] },
        path: { classifications: ['source'], suppressed: false },
      },
    },
    ...overrides,
  };
}

describe('Rust source health schema', () => {
  it('accepts a complete artifact whose summary matches the body', () => {
    expect(validateRustHealthFinalArtifact(artifact())).toEqual([]);
  });

  it('accepts sidecar artifact before wrapper provenance is injected', () => {
    const value = artifact();
    delete value.meta.generated;
    delete value.meta.sidecar;
    delete value.meta.input;
    expect(validateRustHealthSidecarArtifact(value)).toEqual([]);
    expect(validateRustHealthFinalArtifact(value)).toContain('meta.generated invalid');
  });

  it('rejects summary counts that do not match artifact body', () => {
    const value = artifact({
      summary: { ...artifact().summary, signals: 99 },
    });
    expect(validateRustHealthFinalArtifact(value)).toContain(
      'summary.signals expected 2 but found 99',
    );
  });

  it('rejects malformed thresholds, facts, and skipped file records', () => {
    const value = artifact();
    value.meta.policy.thresholds.maxFunctionLines = 0;
    value.files['src/lib.rs'].facts.functions = -1;
    value.skippedFiles = [
      { path: '', reason: 'invalid-utf8' },
      { path: 'bad.rs', reason: 'unknown-reason' },
    ];

    const problems = validateRustHealthFinalArtifact(value);
    expect(problems).toContain('policy.thresholds.maxFunctionLines invalid');
    expect(problems).toContain('files.src/lib.rs.facts.functions invalid');
    expect(problems).toContain('skippedFiles.path invalid');
    expect(problems).toContain('skippedFiles.bad.rs.reason invalid');
  });

  it('rejects malformed final path policy metadata', () => {
    const missing = artifact();
    missing.meta.input.pathPolicy = {};
    const malformed = artifact();
    malformed.meta.input.pathPolicy = { include: [123], exclude: 'target/**' };

    expect(validateRustHealthFinalArtifact(missing)).toEqual(
      expect.arrayContaining([
        'meta.input.pathPolicy.include mismatch',
        'meta.input.pathPolicy.exclude mismatch',
      ]),
    );
    expect(validateRustHealthFinalArtifact(malformed)).toEqual(
      expect.arrayContaining([
        'meta.input.pathPolicy.include mismatch',
        'meta.input.pathPolicy.exclude mismatch',
      ]),
    );
  });

  it('returns validation problems instead of throwing for malformed collection fields', () => {
    const value = artifact();
    value.skippedFiles = {};
    value.files['src/lib.rs'].signals = {};
    value.files['src/lib.rs'].parse.errors = {};

    expect(() => validateRustHealthFinalArtifact(value)).not.toThrow();
    const problems = validateRustHealthFinalArtifact(value);
    expect(problems).toContain('skippedFiles must be an array');
    expect(problems).toContain('files.src/lib.rs.signals must be an array');
    expect(problems).toContain('files.src/lib.rs.parse.errors must be an array');
  });

  it('returns validation problems instead of throwing for malformed nested entries', () => {
    const value = artifact();
    value.skippedFiles = [null];
    value.files['src/lib.rs'].signals = [null];
    value.files['src/lib.rs'].parse.errors = [null];
    value.files['src/null.rs'] = null;

    expect(() => validateRustHealthFinalArtifact(value)).not.toThrow();
    const problems = validateRustHealthFinalArtifact(value);
    expect(problems).toEqual(expect.arrayContaining([
      'skippedFiles.0 must be an object',
      'files.src/lib.rs.signals.0 must be an object',
      'files.src/lib.rs.parse.errors.0 must be an object',
      'files.src/null.rs must be an object',
    ]));
  });

  it('returns validation problems instead of throwing for non-object artifacts', () => {
    expect(() => validateRustHealthFinalArtifact(null)).not.toThrow();
    expect(() => validateRustHealthFinalArtifact(undefined)).not.toThrow();
    expect(validateRustHealthFinalArtifact(null)).toContain('artifact must be an object');
    expect(validateRustHealthFinalArtifact(undefined)).toContain('artifact must be an object');
  });

  it('rejects unsafe artifact paths and malformed path metadata', () => {
    const value = artifact({
      files: {
        '../evil.rs': {
          ...artifact().files['src/lib.rs'],
          path: { classifications: [123], suppressed: 'no' },
        },
      },
      skippedFiles: [{ path: '../evil.rs', reason: 'invalid-utf8' }],
    });
    value.summary = summarizeRustHealthArtifact(value);

    const problems = validateRustHealthFinalArtifact(value);
    expect(problems).toContain('files.../evil.rs.path key invalid');
    expect(problems).toContain('files.../evil.rs.path.classifications invalid');
    expect(problems).toContain('files.../evil.rs.path.suppressed invalid');
    expect(problems).toContain('skippedFiles.../evil.rs.path invalid');
  });

  it('rejects inconsistent parse ok and parse errors state', () => {
    const okWithErrors = artifact();
    okWithErrors.files['src/lib.rs'].parse = {
      ok: true,
      errors: [
        {
          message: 'expected expression',
          claim: 'syntax-only',
          location: {
            line: 1,
            column: 1,
            endLine: 1,
            endColumn: 1,
            byteStart: 0,
            byteEnd: 0,
          },
        },
      ],
    };
    okWithErrors.summary = summarizeRustHealthArtifact(okWithErrors);

    const notOkWithoutErrors = artifact();
    notOkWithoutErrors.files['src/lib.rs'].parse = { ok: false, errors: [] };
    notOkWithoutErrors.summary = summarizeRustHealthArtifact(notOkWithoutErrors);

    expect(validateRustHealthFinalArtifact(okWithErrors)).toContain(
      'files.src/lib.rs.parse.ok true with parse errors',
    );
    expect(validateRustHealthFinalArtifact(notOkWithoutErrors)).toContain(
      'files.src/lib.rs.parse.ok false without parse errors',
    );
  });

  it('sorts file keys, skipped files, signals, and parse errors deterministically', () => {
    const sorted = sortRustHealthArtifact({
      ...artifact(),
      skippedFiles: [
        { path: 'z.rs', reason: 'invalid-utf8' },
        { path: 'a.rs', reason: 'excluded-by-path-policy' },
      ],
      files: {
        'z.rs': {
          facts: {
            items: 0,
            functions: 0,
            maxFunctionLines: 0,
            unsafeBlocks: 0,
            unsafeFunctions: 0,
          },
          signals: [
            {
              kind: 'z',
              severity: 'review',
              claim: 'syntax-only',
              location: {
                line: 1,
                column: 10,
                endLine: 1,
                endColumn: 11,
                byteStart: 9,
                byteEnd: 10,
              },
            },
          ],
          parse: {
            ok: false,
            errors: [
              {
                message: 'late',
                claim: 'syntax-only',
                location: {
                  line: 1,
                  column: 9,
                  endLine: 1,
                  endColumn: 10,
                  byteStart: 8,
                  byteEnd: 9,
                },
              },
              {
                message: 'early',
                claim: 'syntax-only',
                location: {
                  line: 1,
                  column: 1,
                  endLine: 1,
                  endColumn: 2,
                  byteStart: 0,
                  byteEnd: 1,
                },
              },
            ],
          },
          path: { classifications: ['source'], suppressed: false },
        },
        'a.rs': {
          facts: {
            items: 0,
            functions: 0,
            maxFunctionLines: 0,
            unsafeBlocks: 0,
            unsafeFunctions: 0,
          },
          signals: [
            {
              kind: 'late',
              severity: 'review',
              claim: 'syntax-only',
              location: {
                line: 1,
                column: 21,
                endLine: 1,
                endColumn: 25,
                byteStart: 20,
                byteEnd: 24,
              },
            },
            {
              kind: 'early',
              severity: 'review',
              claim: 'syntax-only',
              location: {
                line: 1,
                column: 2,
                endLine: 1,
                endColumn: 7,
                byteStart: 1,
                byteEnd: 6,
              },
            },
          ],
          parse: { ok: true, errors: [] },
          path: { classifications: ['source'], suppressed: false },
        },
      },
    });
    expect(Object.keys(sorted.files)).toEqual(['a.rs', 'z.rs']);
    expect(sorted.skippedFiles.map((file) => file.path)).toEqual(['a.rs', 'z.rs']);
    expect(sorted.files['a.rs'].signals.map((signal) => signal.kind)).toEqual([
      'early',
      'late',
    ]);
    expect(sorted.files['z.rs'].parse.errors.map((error) => error.message)).toEqual([
      'early',
      'late',
    ]);
  });

  it('recomputes summary from artifact contents', () => {
    expect(summarizeRustHealthArtifact(artifact())).toEqual(artifact().summary);
  });
});
