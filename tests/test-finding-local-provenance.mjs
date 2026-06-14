// Tests for finding-local provenance (v1.10.0 P1).
//
// Replaces the previous repo-global `unresolvedInternalRatio >= 0.15`
// gate that demoted EVERY finding when the repo had high resolver
// blindness. Now each finding carries its own `taintedBy` evidence, so
// findings in unaffected parts of the repo keep their tier even when
// another part of the repo is alias-blind.
//
// Two layers under test:
//   (1) _lib/finding-provenance.mjs  — pure helpers
//         specifierCouldMatchFile(spec, relFile, opts)
//         computeFindingProvenance(finding, {filesWithParseErrors,
//                                            unresolvedInternalSpecifiers,
//                                            astEvidence, astCount})
//   (2) _lib/ranking.mjs::tierForFinding — reads finding.taintedBy,
//         demotes per-finding; falls back to global
//         resolver.unresolvedRatio only when taintedBy is undefined
//         (old artifact shape).

import {
  specifierCouldMatchFile,
  computeFindingProvenance,
} from '../_lib/finding-provenance.mjs';
import { tierForFinding } from '../_lib/ranking.mjs';

let passed = 0, failed = 0;
function assert(label, ok, detail) {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else {
    failed++;
    console.log(`  FAIL  ${label}`);
    if (detail) console.log(`        ${detail}`);
  }
}

function safeAction() {
  return {
    kind: 'demote_export_declaration',
    proofComplete: true,
    actionBlockers: [],
    strongerActionBlockers: [],
  };
}

const aliasMap = {
  scopedTsconfigPaths: [{
    scopeDir: 'apps/web',
    baseUrlDir: 'apps/web',
    key: '@/*',
    matchPrefix: '@/',
    matchSuffix: '',
    targets: ['./*'],
    wildcard: true,
  }],
  scopedTsconfigBaseUrls: [{
    scopeDir: 'apps/web',
    baseUrlDir: 'apps/web',
  }],
};
const submoduleOf = (file) => file.replace(/\\/g, '/').split('/').slice(0, 2).join('/');

// ─── Layer 1: specifierCouldMatchFile ─────────────────────────

assert('S1. known alias matches only inside alias scope',
  specifierCouldMatchFile('@/components/auth-control', 'apps/web/components/auth-control.tsx', {
    aliasMap,
    fromHint: 'apps/web/page.ts',
    submoduleOf,
  }) === 'match');

assert('S2. known alias does not taint a different scope',
  specifierCouldMatchFile('@/components/auth-control', 'apps/api/components/auth-control.ts', {
    aliasMap,
    fromHint: 'apps/web/page.ts',
    submoduleOf,
  }) === 'no-match');

assert('S3. known alias does not match unrelated target in scope',
  specifierCouldMatchFile('@/components/auth-control', 'apps/web/utils/logger.ts', {
    aliasMap,
    fromHint: 'apps/web/page.ts',
    submoduleOf,
  }) === 'no-match');

assert('S4. bare specifier without slash does not match anything',
  specifierCouldMatchFile('@scope', 'any/file.ts', { aliasMap, submoduleOf }) === 'no-match');

assert('S5. unknown alias-like spec is unknown only in same submodule',
  specifierCouldMatchFile('~/config', 'apps/web/config.ts', {
    aliasMap,
    fromHint: 'apps/web/page.ts',
    submoduleOf,
  }) === 'unknown');

assert('S6. unknown alias-like spec does not taint other submodule',
  specifierCouldMatchFile('~/config', 'apps/api/config.ts', {
    aliasMap,
    fromHint: 'apps/web/page.ts',
    submoduleOf,
  }) === 'no-match');

assert('S7. baseUrl-like spec is unknown in matching baseUrl scope',
  specifierCouldMatchFile('app/_types', 'apps/web/app/_types.ts', {
    aliasMap,
    fromHint: 'apps/web/page.ts',
    submoduleOf,
  }) === 'unknown');

assert('S8. baseUrl-like spec does not taint outside matching scope',
  specifierCouldMatchFile('app/_types', 'apps/api/app/_types.ts', {
    aliasMap,
    fromHint: 'apps/web/page.ts',
    submoduleOf,
  }) === 'no-match');

assert('S9. relative specifier matches importer-normalized path',
  specifierCouldMatchFile('../components/auth-control', 'apps/web/components/auth-control.tsx', {
    fromHint: 'apps/web/pages/page.ts',
    submoduleOf,
  }) === 'match');

assert('S10. Windows-style backslash target path is normalized',
  specifierCouldMatchFile('@/components/auth-control', 'apps\\web\\components\\auth-control.tsx', {
    aliasMap,
    fromHint: 'apps/web/page.ts',
    submoduleOf,
  }) === 'match');

// ─── Layer 2: computeFindingProvenance ────────────────────────

{
  const p = computeFindingProvenance(
    { file: 'src/foo.ts', symbol: 'foo' },
    {
      filesWithParseErrors: [],
      unresolvedInternalSpecifiers: [],
      astEvidence: 'ast-ident-ref-count',
      astCount: 0,
    });
  assert('P1. no taint when clean', p.taintedBy.length === 0, `got ${JSON.stringify(p.taintedBy)}`);
  assert('P1b. resolverConfidence=high when clean', p.resolverConfidence === 'high');
  assert('P1c. parseStatus=ok when file not in error list', p.parseStatus === 'ok');
  assert('P1d. supportedBy includes ast-ident-ref-count', p.supportedBy.some((s) => s.kind === 'ast-ident-ref-count'));
}

{
  const p = computeFindingProvenance(
    { file: 'src/foo.ts', symbol: 'foo' },
    {
      filesWithParseErrors: ['src/other.ts'],   // ≠ finding.file
      unresolvedInternalSpecifiers: [],
      astEvidence: 'ast-ident-ref-count',
      astCount: 0,
    });
  assert('P2. parse-errors-elsewhere produces parse-errors-present taint',
    p.taintedBy.some((t) => t.kind === 'parse-errors-present'),
    `got ${JSON.stringify(p.taintedBy)}`);
  assert('P2b. resolverConfidence=medium when only soft taint', p.resolverConfidence === 'medium');
}

{
  const p = computeFindingProvenance(
    { file: 'apps/api/foo.ts', symbol: 'foo' },
    {
      filesWithParseErrors: ['apps/web/broken.ts'],
      unresolvedInternalSpecifiers: [],
      astEvidence: 'ast-ident-ref-count',
      astCount: 0,
      submoduleOf,
    });
  assert('P2c. parse error in unrelated submodule does not taint finding',
    p.taintedBy.length === 0,
    `got ${JSON.stringify(p.taintedBy)}`);
}

{
  const p = computeFindingProvenance(
    { file: 'apps/web/foo.ts', symbol: 'foo' },
    {
      filesWithParseErrors: ['apps/web/broken.ts'],
      unresolvedInternalSpecifiers: [],
      astEvidence: 'ast-ident-ref-count',
      astCount: 0,
      submoduleOf,
    });
  assert('P2d. parse error in same submodule remains relevant soft taint',
    p.taintedBy.some((t) => t.kind === 'parse-errors-present'),
    `got ${JSON.stringify(p.taintedBy)}`);
}

{
  const p = computeFindingProvenance(
    { file: 'src/foo.ts', symbol: 'foo' },
    {
      filesWithParseErrors: ['src/foo.ts'],    // the defining file itself
      unresolvedInternalSpecifiers: [],
      astEvidence: 'ast-ident-ref-count',
      astCount: 0,
    });
  assert('P3. defining-file-parse-error taint emitted when file in list',
    p.taintedBy.some((t) => t.kind === 'defining-file-parse-error'));
  assert('P3b. parseStatus=error when file is in the error list',
    p.parseStatus === 'error');
  assert('P3c. resolverConfidence=low for blocking taint', p.resolverConfidence === 'low');
}

{
  const p = computeFindingProvenance(
    { file: 'apps/web/components/auth-control.tsx', symbol: 'AuthControl' },
    {
      filesWithParseErrors: [],
      unresolvedInternalSpecifiers: [
        {
          specifier: '@/components/auth-control',
          consumerFile: 'apps/web/page.ts',
          fromHint: 'apps/web/page.ts',
        },
        {
          specifier: '@/other/thing',
          consumerFile: 'apps/web/page.ts',
          fromHint: 'apps/web/page.ts',
        },
      ],
      aliasMap,
      submoduleOf,
      astEvidence: 'ast-ident-ref-count',
      astCount: 0,
    });
  assert('P4. unresolved-specifier-could-match detected via scoped alias',
    p.taintedBy.some((t) => t.kind === 'unresolved-specifier-could-match'));
  const match = p.taintedBy.find((t) => t.kind === 'unresolved-specifier-could-match');
  assert('P4b. matched specifier listed', match && match.specifiers.includes('@/components/auth-control'));
  assert('P4c. non-matching specifier not listed', match && !match.specifiers.includes('@/other/thing'));
  assert('P4d. resolverConfidence=low on blocking spec match', p.resolverConfidence === 'low');
}

{
  const p = computeFindingProvenance(
    { file: 'apps/web/config.ts', symbol: 'config' },
    {
      filesWithParseErrors: [],
      unresolvedInternalSpecifiers: [{
        specifier: '~/config',
        consumerFile: 'apps/web/page.ts',
        fromHint: 'apps/web/page.ts',
      }],
      aliasMap,
      submoduleOf,
      astEvidence: 'ast-ident-ref-count',
      astCount: 0,
    });
  const unknown = p.taintedBy.find((t) => t.kind === 'unresolved-specifier-could-match-unknown');
  assert('P4e. unknown alias in same submodule emits weak unresolved taint',
    !!unknown,
    `got ${JSON.stringify(p.taintedBy)}`);
  assert('P4f. weak unresolved taint records consumer file',
    unknown?.consumerFile === 'apps/web/page.ts',
    `got ${JSON.stringify(unknown)}`);
  assert('P4g. weak unresolved taint is medium confidence, not low',
    p.resolverConfidence === 'medium',
    `got ${p.resolverConfidence}`);
}

{
  const p = computeFindingProvenance(
    { file: 'apps/api/config.ts', symbol: 'config' },
    {
      filesWithParseErrors: [],
      unresolvedInternalSpecifiers: [{
        specifier: '~/config',
        consumerFile: 'apps/web/page.ts',
        fromHint: 'apps/web/page.ts',
      }],
      aliasMap,
      submoduleOf,
      astEvidence: 'ast-ident-ref-count',
      astCount: 0,
    });
  assert('P4h. unknown alias from other submodule does not taint finding',
    p.taintedBy.length === 0,
    `got ${JSON.stringify(p.taintedBy)}`);
}

{
  // The core P1 win: two findings in the same repo, only one is
  // tainted by the repo's unresolved specs. The second finding must
  // remain clean even though the repo globally has unresolved imports.
  const unresolvedInRepo = [{
    specifier: '@/components/auth-control',
    consumerFile: 'apps/web/page.ts',
    fromHint: 'apps/web/page.ts',
  }];
  const fileA = 'apps/web/components/auth-control.tsx';   // affected
  const fileB = 'apps/web/utils/logger.ts';               // not affected

  const pA = computeFindingProvenance(
    { file: fileA, symbol: 'AuthControl' },
    { filesWithParseErrors: [], unresolvedInternalSpecifiers: unresolvedInRepo,
      aliasMap, submoduleOf, astEvidence: 'ast-ident-ref-count', astCount: 0 });
  const pB = computeFindingProvenance(
    { file: fileB, symbol: 'log' },
    { filesWithParseErrors: [], unresolvedInternalSpecifiers: unresolvedInRepo,
      aliasMap, submoduleOf, astEvidence: 'ast-ident-ref-count', astCount: 0 });

  assert('P5. affected file is tainted', pA.taintedBy.some((t) => t.kind === 'unresolved-specifier-could-match'));
  assert('P5b. unaffected file stays clean (the P1 win)',
    pB.taintedBy.length === 0,
    `expected empty, got ${JSON.stringify(pB.taintedBy)}`);
}

{
  const p = computeFindingProvenance(
    { file: 'packages/prisma/index.ts', symbol: 'PrismaEnums' },
    {
      filesWithParseErrors: [],
      unresolvedInternalSpecifiers: [{
        specifier: '@scope/prisma/enums',
        consumerFile: 'apps/web/page.ts',
        reason: 'workspace-generated-artifact-missing',
        hint: 'generated-artifact-missing',
        targetCandidates: ['packages/prisma/enums.ts', 'packages/prisma/enums/index.ts'],
        generatedArtifact: {
          policyVersion: 'generated-artifact-policy-v1',
          matchedPackage: '@scope/prisma',
          targetSubpath: 'enums',
          generatorFamily: 'prisma',
          confidence: 'strong',
        },
      }],
      aliasMap,
      submoduleOf,
      astEvidence: 'ast-ident-ref-count',
      astCount: 0,
    });

  const generatedTaint = p.taintedBy.find((t) => t.kind === 'generated-artifact-missing-relevant');
  assert('P6. generated artifact miss in candidate package emits relevant soft taint',
    generatedTaint?.specifier === '@scope/prisma/enums' &&
      generatedTaint?.matchedPackage === '@scope/prisma' &&
      generatedTaint?.targetSubpath === 'enums' &&
      generatedTaint?.impact === 'provider-surface-unresolved',
    JSON.stringify(p.taintedBy));
  assert('P6b. generated artifact relevant taint lowers resolver confidence to medium',
    p.resolverConfidence === 'medium',
    `got ${p.resolverConfidence}`);
}

{
  const p = computeFindingProvenance(
    { file: 'packages/ui/Button.tsx', symbol: 'Button' },
    {
      filesWithParseErrors: [],
      unresolvedInternalSpecifiers: [{
        specifier: '@scope/prisma/enums',
        consumerFile: 'packages/prisma/client.ts',
        reason: 'workspace-generated-artifact-missing',
        hint: 'generated-artifact-missing',
        generatedArtifact: {
          policyVersion: 'generated-artifact-policy-v1',
          matchedPackage: '@scope/prisma',
          packageRoot: 'packages/prisma',
          targetSubpath: 'enums',
          generatorFamily: 'prisma',
          confidence: 'strong',
        },
      }],
      aliasMap,
      submoduleOf,
      astEvidence: 'ast-ident-ref-count',
      astCount: 0,
    });

  assert('P7. unrelated generated artifact miss does not taint another package',
    p.taintedBy.length === 0 && p.resolverConfidence === 'high',
    JSON.stringify(p));
}

{
  const p = computeFindingProvenance(
    { file: 'apps/web/app/_types.ts', symbol: 'LayoutProps' },
    {
      filesWithParseErrors: [],
      unresolvedInternalSpecifiers: [{
        specifier: '@scope/prisma/enums',
        consumerFile: 'apps/web/page.ts',
        reason: 'workspace-generated-artifact-missing',
        hint: 'generated-artifact-missing',
        targetCandidates: ['packages/prisma/enums.ts', 'packages/prisma/enums/index.ts'],
        generatedArtifact: {
          policyVersion: 'generated-artifact-policy-v1',
          matchedPackage: '@scope/prisma',
          targetSubpath: 'enums',
          generatorFamily: 'prisma',
          confidence: 'strong',
        },
      }],
      aliasMap,
      submoduleOf,
      astEvidence: 'ast-ident-ref-count',
      astCount: 0,
    });

  assert('P8. generated provider miss in consumer submodule alone stays clean',
    p.taintedBy.length === 0 &&
      p.resolverConfidence === 'high',
    JSON.stringify(p.taintedBy));
}

{
  const p = computeFindingProvenance(
    { file: 'packages/prisma/model.ts', symbol: 'ModelName' },
    {
      filesWithParseErrors: [],
      unresolvedInternalSpecifiers: [],
      aliasMap,
      submoduleOf,
      astEvidence: 'ast-ident-ref-count',
      astCount: 0,
      generatedConsumerBlindZones: [{
        reason: 'generated-consumer-blind-zone',
        sourceReason: 'workspace-generated-artifact-missing',
        specifier: '@scope/prisma/enums',
        consumerFile: 'apps/web/page.ts',
        matchedPackage: '@scope/prisma',
        targetSubpath: 'enums',
        candidatePath: 'packages/prisma/generated/enums.ts',
        scopePackageRoot: 'packages/prisma',
        status: 'missing',
        mode: 'default',
      }],
    });

  assert('P9. generated consumer blind zone emits consumer-surface taint',
    p.taintedBy.some((t) =>
      t.kind === 'generated-artifact-missing-relevant' &&
      t.reason === 'generated-consumer-blind-zone' &&
      t.impact === 'consumer-surface-unresolved' &&
      t.relevance === 'generated-consumer-scope') &&
      p.resolverConfidence === 'medium',
    JSON.stringify(p.taintedBy));
}

// ─── Layer 3: tierForFinding honors finding.taintedBy ─────────

const strongEvidence = {
  runtime: { status: 'dead-confirmed', grounding: 'grounded', hitsInSymbol: 0 },
  staleness: { tier: 'fossil', grounding: 'grounded' },
  resolver: { unresolvedRatio: 0.05 },  // clean globally
};

{
  const f = { file: 'src/a.ts', line: 1, symbol: 'a', bucket: 'C', taintedBy: [], safeAction: safeAction() };
  const { tier } = tierForFinding(f, strongEvidence);
  assert('T1. empty taintedBy + strong evidence → SAFE_FIX', tier === 'SAFE_FIX');
}

{
  const f = {
    file: 'src/a.ts', line: 1, symbol: 'a', bucket: 'C',
    safeAction: safeAction(),
    taintedBy: [{
      kind: 'unresolved-specifier-could-match',
      specifiers: ['@/components/a'], total: 1, effect: '...',
    }],
  };
  const { tier, reason } = tierForFinding(f, strongEvidence);
  assert('T2. unresolved-specifier-could-match → DEGRADED (overrides strong evidence)',
    tier === 'DEGRADED', `got ${tier} (${reason})`);
  assert('T2b. reason surfaces the matching specifier',
    typeof reason === 'string' && reason.includes('@/components/a'));
}

{
  const f = {
    file: 'src/a.ts', line: 1, symbol: 'a', bucket: 'C',
    safeAction: safeAction(),
    taintedBy: [{
      kind: 'unresolved-specifier-could-match-unknown',
      specifiers: ['~/a'], consumerFile: 'src/consumer.ts', total: 1, effect: '...',
    }],
  };
  const { tier, reason } = tierForFinding(f, strongEvidence);
  assert('T2c. unresolved-specifier-could-match-unknown demotes SAFE_FIX → REVIEW_FIX',
    tier === 'REVIEW_FIX', `got ${tier} (${reason})`);
}

{
  const f = {
    file: 'src/a.ts', line: 1, symbol: 'a', bucket: 'C',
    safeAction: safeAction(),
    taintedBy: [{
      kind: 'defining-file-parse-error', file: 'src/a.ts', effect: '...',
    }],
  };
  const { tier } = tierForFinding(f, strongEvidence);
  assert('T3. defining-file-parse-error → DEGRADED', tier === 'DEGRADED');
}

{
  const f = {
    file: 'src/a.ts', line: 1, symbol: 'a', bucket: 'C',
    safeAction: safeAction(),
    taintedBy: [{
      kind: 'parse-errors-present', scope: 'repo-wide',
      affected: 2, sample: ['src/other.ts'], effect: '...',
    }],
  };
  const { tier, reason } = tierForFinding(f, strongEvidence);
  assert('T4. parse-errors-present demotes SAFE_FIX → REVIEW_FIX (soft taint)',
    tier === 'REVIEW_FIX', `got ${tier} (${reason})`);
  assert('T4b. reason mentions parse-errors-elsewhere',
    typeof reason === 'string' && reason.includes('parse-errors-elsewhere'));
}

{
  const f = {
    file: 'packages/prisma/index.ts', line: 1, symbol: 'PrismaEnums', bucket: 'C',
    safeAction: safeAction(),
    taintedBy: [{
      kind: 'generated-artifact-missing-relevant',
      specifier: '@scope/prisma/enums',
      matchedPackage: '@scope/prisma',
      targetSubpath: 'enums',
      impact: 'provider-surface-unresolved',
      effect: '...',
    }],
  };
  const { tier, reason } = tierForFinding(f, strongEvidence);
  assert('T4c. relevant generated artifact miss demotes SAFE_FIX → REVIEW_FIX',
    tier === 'REVIEW_FIX', `got ${tier} (${reason})`);
  assert('T4d. reason mentions generated-artifact-missing',
    typeof reason === 'string' && reason.includes('generated-artifact-missing'));
}

{
  // The user's main complaint fix: in repos with high global ratio, a
  // CLEAN finding (no taint) must NOT be demoted. Use per-finding taint.
  const f = { file: 'src/clean.ts', line: 1, symbol: 'x', bucket: 'C', taintedBy: [], safeAction: safeAction() };
  const evidence = {
    runtime: { status: 'dead-confirmed', grounding: 'grounded', hitsInSymbol: 0 },
    staleness: { tier: 'fossil', grounding: 'grounded' },
    resolver: { unresolvedRatio: 0.45 },  // high global but this finding is clean
  };
  const { tier } = tierForFinding(f, evidence);
  assert('T5. clean finding in high-global-ratio repo → SAFE_FIX (the P1 win)',
    tier === 'SAFE_FIX');
}

{
  // Backward compat: legacy finding without `taintedBy` falls back to
  // the global ratio gate.
  const f = { file: 'src/a.ts', line: 1, symbol: 'a', bucket: 'C' }; // no taintedBy
  const evidence = {
    runtime: { status: 'dead-confirmed', grounding: 'grounded', hitsInSymbol: 0 },
    staleness: { tier: 'fossil', grounding: 'grounded' },
    resolver: { unresolvedRatio: 0.45 },
  };
  const { tier, reason } = tierForFinding(f, evidence);
  assert('T6. legacy finding (no taintedBy) falls back to global ratio gate',
    tier === 'DEGRADED', `got ${tier} (${reason})`);
  assert('T6b. reason mentions resolver-blind fallback',
    typeof reason === 'string' && reason.includes('resolver-blind'));
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
