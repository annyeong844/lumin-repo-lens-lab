// Tests for _lib/pre-write-render.mjs — P1-1 step 5.5.
//
// Six golden fixtures pinning the advisory Markdown + JSON per
// docs/history/phases/p1/p1-1.md §5.5:
//   A — EXISTS + grounded fan-in + clean
//   B — EXISTS + severely-any-contaminated (grounded measurements + warn-on-reuse)
//   C — NOT_OBSERVED + near-name hints (under "Search hints" sub-section)
//   D — CANONICAL_EXISTS_AST_ABSENT (canonical + [확인 불가], NO "CANONICAL DRIFT:")
//   E — EXISTS_MULTIPLE (two identities, both rendered side-by-side)
//   F — non-empty plannedTypeEscapes (one as-unknown-as-T item, 5 fields)
//
// Plus claim-bearing citation coverage assertions across all fixtures.

import { renderMarkdown, renderJson } from '../_lib/pre-write-render.mjs';

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

// ── Claim-bearing citation coverage helper ──────────────────
//
// Extract lines that are claim-bearing (bullets / table rows declaring
// a lookup result or planned escape). Non-claim lines (headers, blanks,
// horizontal rules, pure prose) are exempt.
//
// Approach: a "claim-bearing line" is any bullet line (`- ...`) that
// mentions one of the result keywords. The test asserts each such line
// contains a citation label.

const RESULT_KEYWORDS = [
  'EXISTS',
  'EXISTS_MULTIPLE',
  'CANONICAL_EXISTS',
  'NOT_OBSERVED',
  'Planned type escapes',
  'escapes planned',
];
const CITATION_RE = /\[(grounded|grounded structural|degraded|확인 불가)/;

function claimBearingLines(md) {
  return md
    .split('\n')
    .filter((line) => line.trim().length > 0)
    .filter((line) => !line.startsWith('#'))                       // headers
    .filter((line) => !/^-{3,}$/.test(line.trim()))                 // horizontal rules
    .filter((line) => RESULT_KEYWORDS.some((k) => line.includes(k)));
}

function assertAllClaimsCited(md, fixtureName) {
  const claims = claimBearingLines(md);
  const uncited = claims.filter((line) => !CITATION_RE.test(line));
  // The claim bullet line itself might carry the citation on a sub-line
  // (e.g. `  [grounded, ...]` indented under it). Widen: treat a claim
  // as cited if it OR the next non-blank line carries a citation label.
  const lines = md.split('\n');
  const reallyUncited = uncited.filter((claim) => {
    const idx = lines.indexOf(claim);
    for (let j = idx + 1; j < lines.length && j < idx + 5; j++) {
      if (CITATION_RE.test(lines[j])) return false;
      if (/^#/.test(lines[j])) break;  // next section
    }
    return true;
  });
  assert(`${fixtureName}: every claim-bearing line is cited`,
    reallyUncited.length === 0,
    `uncited=${JSON.stringify(reallyUncited)}`);
}

// ═══ Fixture A: EXISTS + grounded fan-in + clean ═══

{
  const advisory = {
    invocationId: 'test-A',
    intentHash: 'hash-A',
    intent: { names: ['formatDate'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
    lookups: [{
      kind: 'name',
      intentName: 'formatDate',
      result: 'EXISTS',
      identities: [{
        identity: 'src/utils/date.ts::formatDate',
        ownerFile: 'src/utils/date.ts',
        exportedName: 'formatDate',
        fanIn: 8,
        fanInConfidence: 'grounded',
        fanInSpace: { value: 7, type: 1, broad: 0 },
        fanInSpaceConfidence: 'grounded',
        anyContamination: { state: 'clean' },
        resolverConfidence: 'high',
        citations: [
          `[grounded, symbols.json.fanInByIdentity['src/utils/date.ts::formatDate'] = 8]`,
          `[grounded, symbols.json.fanInByIdentitySpace['src/utils/date.ts::formatDate'] = {"value":7,"type":1,"broad":0}]`,
        ],
      }],
      canonicalClaim: null,
      canonicalAstStatus: 'not-consulted',
      nearNames: [],
      citations: [`[grounded, symbols.json.fanInByIdentity['src/utils/date.ts::formatDate'] = 8]`],
    }],
    boundaryChecks: [], drift: [], capabilities: null, failures: [],
  };

  const md = renderMarkdown(advisory);
  assert('A1. fixture A contains "pre-write advisory" title',
    md.includes('pre-write advisory'));
  assert('A2. fixture A contains "EXISTS" row for formatDate',
    /EXISTS.*formatDate|formatDate.*EXISTS/.test(md));
  assert('A3. fixture A contains ownerFile path',
    md.includes('src/utils/date.ts'));
  assert('A4. fixture A cites fanInByIdentity grounded',
    md.includes(`symbols.json.fanInByIdentity['src/utils/date.ts::formatDate']`));
  assert('A4b. fixture A renders value/type/broad fan-in space',
    md.includes('fan-in 8 (value 7, type 1, broad 0)'),
    md);
  assert('A4c. fixture A cites fanInByIdentitySpace grounded',
    md.includes(`symbols.json.fanInByIdentitySpace['src/utils/date.ts::formatDate']`));
  assertAllClaimsCited(md, 'A5');
}

// ═══ Fixture B: EXISTS + severely-any-contaminated ═══

{
  const advisory = {
    invocationId: 'test-B',
    intentHash: 'hash-B',
    intent: { names: ['UserData'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
    lookups: [{
      kind: 'name',
      intentName: 'UserData',
      result: 'EXISTS',
      identities: [{
        identity: 'src/types/User.ts::UserData',
        ownerFile: 'src/types/User.ts',
        exportedName: 'UserData',
        fanIn: 4,
        fanInConfidence: 'grounded',
        anyContamination: {
          state: 'severely-any-contaminated',
          labels: ['has-any', 'any-contaminated', 'severely-any-contaminated'],
          measurements: { totalFields: 7, anyFields: 6, anyFieldRatio: 0.85 },
          recommendation: {
            action: 'warn-on-reuse',
            confidence: 'low',
            reason: 'severely-any-contaminated semantic reuse caution',
          },
        },
        resolverConfidence: 'high',
        citations: [
          `[grounded, symbols.json.fanInByIdentity['src/types/User.ts::UserData'] = 4]`,
          `[grounded, anyContamination.label = 'severely-any-contaminated', measurements = {"totalFields":7,"anyFields":6,"anyFieldRatio":0.85}]`,
        ],
      }],
      canonicalClaim: null,
      canonicalAstStatus: 'not-consulted',
      nearNames: [],
      citations: [],
    }],
    boundaryChecks: [], drift: [], capabilities: null, failures: [],
  };

  const md = renderMarkdown(advisory);
  assert('B1. fixture B renders under "any-contaminated" section',
    md.includes('Already exists — but any-contaminated'));
  assert('B2. fixture B surfaces raw measurements (anyFieldRatio 0.85)',
    md.includes('0.85'));
  assert('B3. fixture B separates recommendation caution from measurement confidence',
    md.includes('[recommendation: warn-on-reuse, confidence: low') &&
      md.includes('[grounded, anyContamination.label') &&
      !md.includes('[degraded, any-contaminated'),
    md);
  assert('B4. severely-any-contaminated label present in rendered output',
    md.includes('severely-any-contaminated'));
  assertAllClaimsCited(md, 'B5');
}

// ═══ Fixture C: NOT_OBSERVED + near-name hints ═══

{
  const advisory = {
    invocationId: 'test-C',
    intentHash: 'hash-C',
    intent: { names: ['formatTimestamp'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
    lookups: [{
      kind: 'name',
      intentName: 'formatTimestamp',
      result: 'NOT_OBSERVED',
      identities: [],
      canonicalClaim: null,
      canonicalAstStatus: 'not-consulted',
      nearNames: [
        { name: 'formatDate', ownerFile: 'src/utils/date.ts', distance: 2 },
        { name: 'formatDateTime', ownerFile: 'src/utils/date.ts', distance: 2 },
        { name: 'formatTimeAgo', ownerFile: 'src/utils/time.ts', distance: 2 },
      ],
      citations: [
        `[확인 불가, scan range: symbols.json.defIndex does not contain 'formatTimestamp'; near-name hints emitted]`,
        `[degraded, fuzzy-name match; source: symbols.json.defIndex name scan — search hint only, NOT a grounded reuse claim]`,
      ],
    }],
    boundaryChecks: [], drift: [], capabilities: null, failures: [],
  };

  const md = renderMarkdown(advisory);
  assert('C1. fixture C contains NOT_OBSERVED for formatTimestamp',
    md.includes('NOT_OBSERVED'));
  assert('C2. fixture C has "Search hints (not reuse candidates)" sub-section',
    md.includes('Search hints (not reuse candidates)'));
  assert('C3. fixture C lists near-name formatDate',
    md.includes('formatDate'));

  // CRITICAL: near-name rows do NOT appear under "Already exists".
  //
  // Assert structurally: split by H3 headers and verify that the
  // "Already exists" section body does not mention any near-name.
  const sections = md.split(/^### /m);
  const alreadyExistsSection = sections.find((s) => s.startsWith('Already exists (reuse candidates)'));
  assert('C4. "Already exists (reuse candidates)" section does NOT contain near-name rows',
    !alreadyExistsSection ||
    (!alreadyExistsSection.includes('formatDate') &&
     !alreadyExistsSection.includes('formatDateTime') &&
     !alreadyExistsSection.includes('formatTimeAgo')),
    `Already exists section: ${alreadyExistsSection?.slice(0, 300)}`);

  // Near-names must carry "search hint only" citation wording.
  assert('C5. near-name citation includes "search hint only"',
    md.includes('search hint only'));
  assertAllClaimsCited(md, 'C6');
}

// ═══ Fixture D: CANONICAL_EXISTS_AST_ABSENT (no CANONICAL DRIFT:) ═══

{
  const advisory = {
    invocationId: 'test-D',
    intentHash: 'hash-D',
    intent: { names: ['TokenKind'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
    lookups: [{
      kind: 'name',
      intentName: 'TokenKind',
      result: 'CANONICAL_EXISTS_AST_ABSENT',
      identities: [],
      canonicalClaim: {
        ownerFile: 'src/auth/token.ts',
        line: 7,
        file: 'canonical/type-ownership.md',
        section: 'Single owner (strong)',
      },
      canonicalAstStatus: 'ast-absent',
      nearNames: [],
      citations: [
        `[grounded, canonical/type-ownership.md:L7 declares owner 'src/auth/token.ts' for 'TokenKind']`,
        `[확인 불가, scan range: current AST does not observe 'TokenKind' under TS/JS production scope]`,
      ],
    }],
    boundaryChecks: [], drift: [], capabilities: null, failures: [],
  };

  const md = renderMarkdown(advisory);
  assert('D1. fixture D cites canonical declaration',
    md.includes(`canonical/type-ownership.md:L7`));
  assert('D2. fixture D cites AST absent via [확인 불가]',
    md.includes('확인 불가'));

  // P1-1 contract: CANONICAL_EXISTS_AST_DISAGREE and AST_ABSENT must NOT
  // use the literal "CANONICAL DRIFT:" phrase. That's P1-3 territory.
  assert('D3. Markdown does NOT contain literal "CANONICAL DRIFT:"',
    !md.includes('CANONICAL DRIFT:'));
  assertAllClaimsCited(md, 'D4');
}

// ═══ Fixture E: EXISTS_MULTIPLE (two identities side-by-side) ═══

{
  const advisory = {
    invocationId: 'test-E',
    intentHash: 'hash-E',
    intent: { names: ['User'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
    lookups: [{
      kind: 'name',
      intentName: 'User',
      result: 'EXISTS_MULTIPLE',
      identities: [
        {
          identity: 'apps/admin/types.ts::User',
          ownerFile: 'apps/admin/types.ts',
          exportedName: 'User',
          fanIn: 5,
          fanInConfidence: 'grounded',
          anyContamination: { state: 'clean' },
          resolverConfidence: 'high',
          citations: [`[grounded, symbols.json.fanInByIdentity['apps/admin/types.ts::User'] = 5]`],
        },
        {
          identity: 'apps/blog/types.ts::User',
          ownerFile: 'apps/blog/types.ts',
          exportedName: 'User',
          fanIn: 2,
          fanInConfidence: 'grounded',
          anyContamination: { state: 'clean' },
          resolverConfidence: 'high',
          citations: [`[grounded, symbols.json.fanInByIdentity['apps/blog/types.ts::User'] = 2]`],
        },
      ],
      canonicalClaim: null,
      canonicalAstStatus: 'not-consulted',
      nearNames: [],
      citations: [],
    }],
    boundaryChecks: [], drift: [], capabilities: null, failures: [],
  };

  const md = renderMarkdown(advisory);
  assert('E1. EXISTS_MULTIPLE header or marker present',
    md.includes('EXISTS_MULTIPLE'));
  assert('E2. BOTH owner files rendered',
    md.includes('apps/admin/types.ts') && md.includes('apps/blog/types.ts'));
  assert('E3. per-identity fan-ins preserved (= 5 and = 2)',
    md.includes('= 5]') && md.includes('= 2]'));
  assertAllClaimsCited(md, 'E4');
}

// ═══ Fixture F: non-empty plannedTypeEscapes ═══

{
  const advisory = {
    invocationId: 'test-F',
    intentHash: 'hash-F',
    intent: {
      names: [], shapes: [], files: [], dependencies: [],
      plannedTypeEscapes: [{
        escapeKind: 'as-unknown-as-T',
        locationHint: 'src/vendor/wrapper.ts::adaptResponse',
        codeShape: 'response as unknown as ThirdPartyShape',
        reason: 'upstream SDK lacks type exports',
        alternativeConsidered: 'unknown + decoder; rejected because runtime validation library not yet approved',
      }],
    },
    lookups: [],
    boundaryChecks: [], drift: [], capabilities: null, failures: [],
  };

  const md = renderMarkdown(advisory);
  assert('F1. "Planned type escapes" section present',
    md.includes('Planned type escapes (from Step 2 intent)'));
  assert('F2. escapeKind "as-unknown-as-T" rendered',
    md.includes('as-unknown-as-T'));
  assert('F3. locationHint rendered',
    md.includes('src/vendor/wrapper.ts::adaptResponse'));
  assert('F4. codeShape rendered',
    md.includes('response as unknown as ThirdPartyShape'));
  assert('F5. reason rendered',
    md.includes('upstream SDK lacks type exports'));
  assert('F6. alternativeConsidered rendered (at least the prefix)',
    md.includes('unknown + decoder'));
  assert('F7. grounded citation on the planned escape (intent extracted at pre-write Step 2)',
    md.includes('[grounded, intent extracted at pre-write Step 2'));

  // With empty `names`, "Already exists" section should be omitted
  // (no body means no header).
  assert('F8. Already exists section omitted when names list empty',
    !md.includes('### Already exists (reuse candidates)'));
  assertAllClaimsCited(md, 'F9');
}

// ═══ Empty-list planned escapes ═══

{
  const advisory = {
    invocationId: 'test-empty-plan',
    intentHash: 'hash-emp',
    intent: { names: [], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
    lookups: [],
    boundaryChecks: [], drift: [], capabilities: null, failures: [],
  };
  const md = renderMarkdown(advisory);
  assert('G1. empty planned escapes renders the default zero-planned text',
    md.includes('0 escapes planned'));
  // Canonical empty-list text references fact-model.md §3.9 enumeration.
  assert('G2. empty-list text references all 11 escapeKinds or fact-model §3.9',
    (md.includes('explicit-any') && md.includes('ts-expect-error')) ||
    md.includes('fact-model.md §3.9'),
    `Markdown fragment: ${md.slice(md.indexOf('Planned type escapes'))}`);
}

// ═══ Fixture G: NEW_FILE + NOT_EVALUATED boundary ═══

{
  const advisory = {
    invocationId: 'test-G',
    intentHash: 'hash-G',
    intent: { names: [], shapes: [], files: ['src/utils/time.ts'], dependencies: [], plannedTypeEscapes: [] },
    lookups: [{
      kind: 'file',
      intentFile: 'src/utils/time.ts',
      result: 'NEW_FILE',
      loc: null,
      inboundFanIn: null,
      inboundFanInConfidence: 'unavailable',
      submodule: 'src',
      boundary: { status: 'NOT_EVALUATED', rule: null },
      tags: [],
      citations: [
        `[grounded, topology.json.nodes does not contain 'src/utils/time.ts'; topology.meta.complete = true; symbols.filesWithParseErrors does not list it]`,
        `[확인 불가, reason: P1-2 intent carries no planned from→to edge; boundary rules consulted only when endpoints are known (P1-3)]`,
      ],
    }],
    boundaryChecks: [], drift: [], capabilities: null, failures: [],
  };
  const md = renderMarkdown(advisory);
  assert('G1. New code candidates section present',
    md.includes('### New code candidates'));
  assert('G2. NEW_FILE row mentions the intent file',
    md.includes('src/utils/time.ts'));
  assert('G3. NEW_FILE grounded topology citation rendered',
    md.includes('topology.meta.complete = true'));
  assert('G4. boundary sub-line cites "not evaluated"',
    /boundary.*not.evaluated|not.evaluated.*boundary/i.test(md));
  assert('G5. "CANONICAL DRIFT:" literal never appears',
    !md.includes('CANONICAL DRIFT:'));
}

// ═══ Fixture H: FILE_STATUS_UNKNOWN (topology absent) ═══

{
  const advisory = {
    invocationId: 'test-H',
    intentHash: 'hash-H',
    intent: { names: [], shapes: [], files: ['src/utils/time.ts'], dependencies: [], plannedTypeEscapes: [] },
    lookups: [{
      kind: 'file',
      intentFile: 'src/utils/time.ts',
      result: 'FILE_STATUS_UNKNOWN',
      loc: null,
      inboundFanIn: null,
      inboundFanInConfidence: 'unavailable',
      submodule: null,
      boundary: { status: 'NOT_EVALUATED', rule: null },
      tags: [],
      citations: [
        `[확인 불가, reason: topology absent and symbols.defIndex has no entry; file existence cannot be grounded]`,
      ],
    }],
    boundaryChecks: [], drift: [], capabilities: null, failures: [],
  };
  const md = renderMarkdown(advisory);
  assert('H1. FILE_STATUS_UNKNOWN row rendered under New code candidates',
    md.includes('### New code candidates') && md.includes('FILE_STATUS_UNKNOWN'));
  assert('H2. [확인 불가] citation present',
    md.includes('확인 불가'));
  assert('H3. literal "NEW_FILE" claim NEVER appears in this fixture',
    !md.includes('NEW_FILE'));
}

// ═══ Fixture I: DEPENDENCY_AVAILABLE (under Already exists, NOT New code) ═══

{
  const advisory = {
    invocationId: 'test-I',
    intentHash: 'hash-I',
    intent: { names: [], shapes: [], files: [], dependencies: ['dayjs'], plannedTypeEscapes: [] },
    lookups: [{
      kind: 'dependency',
      depName: 'dayjs',
      declaredIn: 'dependencies',
      result: 'DEPENDENCY_AVAILABLE',
      existingImports: {
        examples: [
          { file: 'src/a.ts', fromSpec: 'dayjs' },
          { file: 'src/b.ts', fromSpec: 'dayjs/plugin/utc' },
        ],
        observedImportCount: 2,
        countConfidence: 'grounded',
      },
      citations: [
        `[grounded, package.json.dependencies['dayjs'] = '1.0.0']`,
        `[grounded, symbols.json.dependencyImportConsumers fromSpec matches 'dayjs' → 2 observed static-import consumers]`,
      ],
    }],
    boundaryChecks: [], drift: [], capabilities: null, failures: [],
  };
  const md = renderMarkdown(advisory);
  assert('I1. DEPENDENCY_AVAILABLE rendered under Already exists',
    md.includes('### Already exists (reuse candidates)') && md.includes('DEPENDENCY_AVAILABLE'));
  // Pinning: dependency-available NEVER appears under New code candidates.
  const sections = md.split(/^### /m);
  const newCodeSection = sections.find((s) => s.startsWith('New code candidates'));
  assert('I2. "DEPENDENCY_AVAILABLE" does NOT appear under New code candidates',
    !newCodeSection || !newCodeSection.includes('DEPENDENCY_AVAILABLE'));
  assert('I3. package.json citation rendered',
    md.includes(`package.json.dependencies['dayjs']`));
}

// ═══ Fixture I2: declared dependency, import graph unavailable ═══

{
  const advisory = {
    invocationId: 'test-I2',
    intentHash: 'hash-I2',
    intent: { names: [], shapes: [], files: [], dependencies: ['dayjs'], plannedTypeEscapes: [] },
    lookups: [{
      kind: 'dependency',
      depName: 'dayjs',
      declaredIn: 'dependencies',
      result: 'DEPENDENCY_AVAILABLE_IMPORT_GRAPH_UNAVAILABLE',
      existingImports: {
        examples: [],
        observedImportCount: null,
        countConfidence: 'unavailable',
        unavailableReason: 'symbols.json absent',
      },
      citations: [
        `[grounded, package.json.dependencies['dayjs'] = '1.0.0']`,
        `[확인 불가, reason: symbols.json absent; observed static-import consumer count unavailable for 'dayjs']`,
      ],
    }],
    boundaryChecks: [], drift: [], capabilities: null, failures: [],
  };
  const md = renderMarkdown(advisory);
  assert('I2a. unavailable import graph renders as unavailable, not zero',
    md.includes('import graph unavailable') && !md.includes('0 observed consumer'),
    md);
  assert('I2b. unavailable dependency lookup keeps [확인 불가] citation',
    md.includes('확인 불가') && md.includes('symbols.json absent'),
    md);
}

// ═══ Fixture J: shape UNAVAILABLE under Watch-for ═══

{
  const advisory = {
    invocationId: 'test-J',
    intentHash: 'hash-J',
    intent: { names: [], shapes: [{ fields: ['year', 'month'] }], files: [], dependencies: [], plannedTypeEscapes: [] },
    lookups: [{
      kind: 'shape',
      shape: { fields: ['year', 'month'] },
      result: 'UNAVAILABLE',
      citations: [
        `[확인 불가, shape-index.json absent; run build-shape-index.mjs to enable P4 shape-hash lookup]`,
      ],
    }],
    boundaryChecks: [], drift: [], capabilities: null, failures: [],
  };
  const md = renderMarkdown(advisory);
  assert('J1. Watch-for section rendered',
    md.includes('### Watch-for'));
  assert('J2. shape-hash + P4 substrings both present',
    md.includes('shape-hash') && md.includes('P4'));
  assert('J3. [확인 불가] citation on the shape row',
    md.includes('확인 불가'));
}

// ═══ Fixture K: multi-item planned escapes, grouping cosmetic ═══

{
  const plannedTypeEscapes = [
    { escapeKind: 'as-any', locationHint: 'src/a.ts::fn1', reason: 'first' },
    { escapeKind: 'ts-expect-error', locationHint: 'src/b.ts::fn2', reason: 'second' },
    { escapeKind: 'as-any', locationHint: 'src/c.ts::fn3', reason: 'third' },
  ];
  const advisory = {
    invocationId: 'test-K',
    intentHash: 'hash-K',
    intent: { names: [], shapes: [], files: [], dependencies: [], plannedTypeEscapes },
    lookups: [],
    boundaryChecks: [], drift: [], capabilities: null, failures: [],
  };
  const md = renderMarkdown(advisory);
  const json = renderJson(advisory);

  assert('K1. all three escape items render',
    md.includes('fn1') && md.includes('fn2') && md.includes('fn3'));

  // PINNING: JSON artifact preserves the ORIGINAL input order — so P2
  // 1:1 comparison against caller-supplied order works regardless of
  // any Markdown-level grouping.
  assert('K2. renderJson preserves original plannedTypeEscapes order',
    json.intent.plannedTypeEscapes[0].locationHint === 'src/a.ts::fn1' &&
    json.intent.plannedTypeEscapes[1].locationHint === 'src/b.ts::fn2' &&
    json.intent.plannedTypeEscapes[2].locationHint === 'src/c.ts::fn3');
}

// ═══ Fixture L: FILE_EXISTS with high inbound fan-in → Watch-for hub ═══

{
  const advisory = {
    invocationId: 'test-L',
    intentHash: 'hash-L',
    intent: { names: [], shapes: [], files: ['src/old/orphan.ts'], dependencies: [], plannedTypeEscapes: [] },
    lookups: [{
      kind: 'file',
      intentFile: 'src/old/orphan.ts',
      result: 'FILE_EXISTS',
      loc: 120,
      inboundFanIn: 14,
      inboundFanInConfidence: 'grounded',
      submodule: 'src',
      boundary: { status: 'NOT_EVALUATED', rule: null },
      tags: [],
      citations: [
        `[grounded, topology.json.nodes['src/old/orphan.ts'] present, loc = 120]`,
        `[grounded, topology.json.edges inbound count for 'src/old/orphan.ts' = 14]`,
      ],
    }],
    boundaryChecks: [], drift: [], capabilities: null, failures: [],
  };
  const md = renderMarkdown(advisory);
  assert('L1. file appears under Already exists (reuse candidates)',
    md.includes('### Already exists (reuse candidates)') && md.includes('src/old/orphan.ts'));
  assert('L2. Watch-for hub signal also present',
    md.includes('### Watch-for'));
  assert('L3. hub citation includes count AND threshold',
    /inboundFanIn\s*=\s*14/.test(md) && /threshold\s*=\s*10/.test(md),
    `md excerpt: ${md.slice(md.indexOf('Watch-for'), md.indexOf('Watch-for') + 600)}`);
}

// ═══ Fixture L2: NEW_FILE with domain cluster → Watch-for domain signal ═══

{
  const advisory = {
    invocationId: 'test-L2',
    intentHash: 'hash-L2',
    intent: { names: [], shapes: [], files: ['lib/cardNewsService.js'], dependencies: [], plannedTypeEscapes: [] },
    lookups: [{
      kind: 'file',
      intentFile: 'lib/cardNewsService.js',
      result: 'NEW_FILE',
      loc: null,
      inboundFanIn: 0,
      inboundFanInConfidence: 'grounded',
      submodule: 'lib',
      boundary: { status: 'NOT_EVALUATED', rule: null },
      tags: [],
      domainCluster: {
        kind: 'DOMAIN_CLUSTER_DETECTED',
        directory: 'lib',
        basenamePrefix: 'cardNews',
        prefixPath: 'lib/cardNews',
        matchCount: 3,
        totalLoc: 240,
        examples: [
          { file: 'lib/cardNewsGenerator.js', loc: 120 },
          { file: 'lib/cardNewsJobStore.js', loc: 40 },
          { file: 'lib/cardNewsPlanner.js', loc: 80 },
        ],
        omittedCount: 0,
        citations: [
          `[grounded, topology.json.nodes matched 3 files with prefix 'lib/cardNews*']`,
        ],
      },
      citations: [
        `[grounded, topology.json.nodes does not contain 'lib/cardNewsService.js'; topology.meta.complete = true; symbols.filesWithParseErrors does not list it]`,
      ],
    }],
    boundaryChecks: [], drift: [], capabilities: null, failures: [],
  };
  const md = renderMarkdown(advisory);
  assert('L2a. NEW_FILE remains under New code candidates',
    md.includes('### New code candidates') && md.includes('NEW_FILE') && md.includes('lib/cardNewsService.js'));
  assert('L2b. domain cluster also appears under Watch-for',
    md.includes('### Watch-for') && md.includes('DOMAIN_CLUSTER_DETECTED') && md.includes('lib/cardNews*'));
  assert('L2c. existing sibling examples render',
    md.includes('lib/cardNewsGenerator.js') && md.includes('lib/cardNewsPlanner.js'));
}

// ═══ Fixture L3: capability-absent is one global note, not per row noise ═══

{
  const advisory = {
    invocationId: 'test-L3',
    intentHash: 'hash-L3',
    intent: { names: ['Logger'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
    lookups: [{
      kind: 'name',
      intentName: 'Logger',
      result: 'EXISTS_MULTIPLE',
      identities: [
        {
          identity: 'src/a.ts::Logger',
          ownerFile: 'src/a.ts',
          exportedName: 'Logger',
          fanIn: 1,
          fanInConfidence: 'grounded',
          anyContamination: { state: 'capability-absent' },
          resolverConfidence: 'high',
          citations: [
            `[grounded, symbols.json.fanInByIdentity['src/a.ts::Logger'] = 1]`,
            `[확인 불가, reason: producer did not emit anyContamination capability (symbols.meta.supports.anyContamination !== true)]`,
          ],
        },
        {
          identity: 'src/b.ts::Logger',
          ownerFile: 'src/b.ts',
          exportedName: 'Logger',
          fanIn: 0,
          fanInConfidence: 'grounded',
          anyContamination: { state: 'capability-absent' },
          resolverConfidence: 'high',
          citations: [
            `[grounded, symbols.json.fanInByIdentity['src/b.ts::Logger'] = 0]`,
            `[확인 불가, reason: producer did not emit anyContamination capability (symbols.meta.supports.anyContamination !== true)]`,
          ],
        },
      ],
      canonicalClaim: null,
      canonicalAstStatus: 'not-consulted',
      nearNames: [],
      citations: [],
    }],
    boundaryChecks: [], drift: [], capabilities: { identityFanIn: true, anyContamination: false }, failures: [],
  };
  const md = renderMarkdown(advisory);
  const count = (md.match(/producer did not emit anyContamination capability/g) ?? []).length;
  assert('L3a. capability-absent note renders once, not once per identity',
    count === 1,
    `count=${count}\n${md}`);
  assert('L3b. fan-in citations still render for both identities',
    md.includes(`symbols.json.fanInByIdentity['src/a.ts::Logger']`) &&
    md.includes(`symbols.json.fanInByIdentity['src/b.ts::Logger']`));
}

// ═══ Fixture M: Canonical drift section (P1-3) ═══

{
  const advisory = {
    invocationId: 'test-M',
    intentHash: 'hash-M',
    intent: { names: ['User'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
    lookups: [{
      kind: 'name',
      intentName: 'User',
      result: 'CANONICAL_EXISTS_AST_DISAGREE',
      identities: [{
        identity: 'apps/legacy/user.ts::User',
        ownerFile: 'apps/legacy/user.ts',
        exportedName: 'User',
        fanIn: 1,
        fanInConfidence: 'grounded',
        anyContamination: { state: 'clean' },
        resolverConfidence: 'high',
        citations: [],
      }],
      canonicalClaim: {
        ownerFile: 'src/models/User.ts',
        line: 42,
        file: 'canonical/type-ownership.md',
        section: 'Single owner (strong)',
      },
      canonicalAstStatus: 'owner-disagrees',
      nearNames: [],
      citations: [],
    }],
    boundaryChecks: [],
    drift: [{
      intentName: 'User',
      canonicalOwner: 'src/models/User.ts',
      canonicalFile: 'canonical/type-ownership.md',
      canonicalLine: 42,
      astOwners: ['apps/legacy/user.ts'],
      kind: 'owner-disagrees',
    }],
    capabilities: null,
    failures: [],
  };

  const md = renderMarkdown(advisory);

  assert('M1. "### Canonical drift" section rendered',
    md.includes('### Canonical drift'));
  assert('M2. "CANONICAL DRIFT:" literal appears in the drift section',
    md.includes('CANONICAL DRIFT:'));
  assert('M3. Both owners cited (canonical src/models/User.ts + AST apps/legacy/user.ts)',
    md.includes('src/models/User.ts') && md.includes('apps/legacy/user.ts'));
  assert('M4. Canonical line number cited',
    md.includes(':L42') || md.includes('L42'));

  // Structural pinning: "CANONICAL DRIFT:" appears ONLY under "### Canonical drift".
  const sections = md.split(/^### /m);
  const driftSection = sections.find((s) => s.startsWith('Canonical drift'));
  assert('M5. "CANONICAL DRIFT:" is contained within the Canonical drift section',
    driftSection && driftSection.includes('CANONICAL DRIFT:'));

  const nonDriftSections = sections.filter((s, i) => i > 0 && !s.startsWith('Canonical drift'));
  const literalInOtherSections = nonDriftSections.some((s) => s.includes('CANONICAL DRIFT:'));
  assert('M6. "CANONICAL DRIFT:" appears ONLY in the drift section (not in Already exists etc)',
    !literalInOtherSections,
    `other section contained the literal: ${nonDriftSections.find((s) => s.includes('CANONICAL DRIFT:'))?.slice(0, 200)}`);
}

// ═══ Fixture M-absent: drift with ast-absent (empty astOwners) ═══

{
  const advisory = {
    invocationId: 'test-M-absent',
    intentHash: 'hash-Ma',
    intent: { names: ['GoneType'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
    lookups: [{
      kind: 'name',
      intentName: 'GoneType',
      result: 'CANONICAL_EXISTS_AST_ABSENT',
      identities: [],
      canonicalClaim: {
        ownerFile: 'src/types/gone.ts',
        line: 7,
        file: 'canonical/type-ownership.md',
        section: 'Single owner (strong)',
      },
      canonicalAstStatus: 'ast-absent',
      nearNames: [],
      citations: [],
    }],
    boundaryChecks: [],
    drift: [{
      intentName: 'GoneType',
      canonicalOwner: 'src/types/gone.ts',
      canonicalFile: 'canonical/type-ownership.md',
      canonicalLine: 7,
      astOwners: [],
      kind: 'ast-absent',
    }],
    capabilities: null,
    failures: [],
  };

  const md = renderMarkdown(advisory);
  assert('Ma1. ast-absent drift renders CANONICAL DRIFT:',
    md.includes('### Canonical drift') && md.includes('CANONICAL DRIFT:'));
  assert('Ma2. ast-absent drift cites "not observed" / "ast" language',
    /not observed|AST does not observe|ast-absent|현재 AST에 없음/i.test(md),
    `md drift section: ${md.slice(md.indexOf('Canonical drift'))}`);
}

// ═══ Fixture M-empty: empty drift → section omitted ═══

{
  const advisory = {
    invocationId: 'test-M-empty',
    intentHash: 'hash-Me',
    intent: { names: ['formatDate'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
    lookups: [{
      kind: 'name',
      intentName: 'formatDate',
      result: 'EXISTS',
      identities: [{
        identity: 'src/utils/date.ts::formatDate',
        ownerFile: 'src/utils/date.ts',
        exportedName: 'formatDate',
        fanIn: 8,
        fanInConfidence: 'grounded',
        anyContamination: { state: 'clean' },
        resolverConfidence: 'high',
        citations: [],
      }],
      canonicalClaim: null,
      canonicalAstStatus: 'not-consulted',
      nearNames: [],
      citations: [],
    }],
    boundaryChecks: [],
    drift: [],  // empty
    capabilities: null,
    failures: [],
  };
  const md = renderMarkdown(advisory);
  assert('Me1. empty drift → "### Canonical drift" section omitted',
    !md.includes('### Canonical drift'));
  assert('Me2. "CANONICAL DRIFT:" literal absent when drift empty',
    !md.includes('CANONICAL DRIFT:'));
}

// ═══ renderJson round-trip ═══

{
  const advisory = {
    invocationId: 'json-test',
    intentHash: 'h',
    intent: { names: ['x'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
    intentWarnings: [
      { kind: 'missing-intent-key-defaulted', key: 'dependencies', action: 'defaulted-to-empty-array' },
      { kind: 'missing-intent-key-defaulted', key: 'plannedTypeEscapes', action: 'defaulted-to-empty-array' },
    ],
    lookups: [],
    boundaryChecks: [], drift: [], capabilities: null, failures: [],
  };
  const md = renderMarkdown(advisory);
  const json = renderJson(advisory);
  assert('J1. renderJson returns an object (not string)',
    typeof json === 'object' && json !== null);
  assert('J2. renderJson preserves invocationId',
    json.invocationId === 'json-test');
  assert('J3. renderJson preserves intent',
    Array.isArray(json.intent?.names) && json.intent.names.includes('x'));
  assert('J4. renderJson includes empty-array defaults for optional collections',
    Array.isArray(json.lookups) && Array.isArray(json.boundaryChecks) && Array.isArray(json.drift));
  assert('J5. renderJson preserves intentWarnings',
    Array.isArray(json.intentWarnings) && json.intentWarnings.length === 2,
    JSON.stringify(json.intentWarnings));
  assert('J6. renderMarkdown suppresses benign compact intent defaults',
    !md.includes('### Intent schema notes') &&
    !md.includes('Missing top-level intent keys defaulted') &&
    !md.includes('pre-write intent schema normalization'),
    md);
}

// ═══ Fixture L4: semantic intent-token hints render under Search hints ═══

{
  const advisory = {
    invocationId: 'test-L4',
    intentHash: 'hash-L4',
    intent: {
      names: ['loadArtifactJson'],
      nameDeclarations: [{
        name: 'loadArtifactJson',
        kind: 'function',
        why: 'load a JSON artifact file with existence check',
      }],
      shapes: [], files: [], dependencies: [], plannedTypeEscapes: [],
    },
    lookups: [{
      kind: 'name',
      intentName: 'loadArtifactJson',
      result: 'NOT_OBSERVED',
      identities: [],
      canonicalClaim: null,
      canonicalAstStatus: 'not-consulted',
      nearNames: [],
      semanticHints: [
        {
          name: 'loadIfExists',
          ownerFile: '_lib/artifacts.mjs',
          matchedTokens: ['artifact', 'exist'],
          score: 2,
        },
      ],
      citations: [
        `[degraded, intent-token match; source: symbols.json.defIndex plus intent.name/intent.why tokens — search hint only, NOT a grounded reuse claim]`,
      ],
    }],
    boundaryChecks: [], drift: [], capabilities: null, failures: [],
  };
  const md = renderMarkdown(advisory);
  assert('L4a. semantic hint routes to Search hints',
    md.includes('### Search hints') && md.includes('loadIfExists'));
  assert('L4b. semantic hint remains degraded search hint, not reuse claim',
    md.includes('intent-token match') && md.includes('NOT a grounded reuse claim'));
  assert('L4c. matched tokens render for reviewer inspection',
    md.includes('matched tokens: `artifact`, `exist`'));
}

// ═══ Cue-tier JSON fields ═══

{
  const advisory = {
    invocationId: 'cue-json-test',
    intentHash: 'cue-hash',
    intent: { names: ['createLogger'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
    lookups: [],
    cueCards: [{
      candidate: {
        identity: 'src/logger.ts::createLogger',
        ownerFile: 'src/logger.ts',
        exportedName: 'createLogger',
      },
      renderTier: 'SAFE_CUE',
      cues: [{
        cueTier: 'SAFE_CUE',
        safeMeaning: 'claim-only',
        notSafeFor: ['semantic-equivalence', 'auto-reuse', 'auto-fix'],
        evidenceLane: 'exact-symbol',
        claim: 'exact exported symbol exists',
        confidence: 'grounded',
        evidence: [{ artifact: 'symbols.json', matchedField: 'defIndex' }],
      }],
    }],
    suppressedCues: [{
      cueTier: 'MUTED_CUE',
      evidenceLane: 'intent-token',
      reason: 'domain-token-overlap',
      tokens: ['create'],
      candidateCount: 2,
      tokenizerVersion: 'camel-snake-kebab-digit-v1',
      tokenPolicyVersion: 'prewrite-token-policy-v1',
    }],
    unavailableEvidence: [{
      evidenceLane: 'function-signature',
      status: 'UNAVAILABLE',
      reason: 'missing-artifact',
      artifact: 'function-clones.json',
    }],
    cuePolicy: {
      tokenizerVersion: 'camel-snake-kebab-digit-v1',
      tokenPolicyVersion: 'prewrite-token-policy-v1',
      weakCommonTokens: ['create'],
    },
    boundaryChecks: [],
    drift: [],
    capabilities: null,
    failures: [],
  };
  const json = renderJson(advisory);
  assert('N1. renderJson preserves cueCards',
    json.cueCards?.[0]?.cues?.[0]?.cueTier === 'SAFE_CUE',
    JSON.stringify(json.cueCards));
  assert('N2. renderJson preserves suppressedCues',
    json.suppressedCues?.[0]?.reason === 'domain-token-overlap',
    JSON.stringify(json.suppressedCues));
  assert('N3. renderJson preserves unavailableEvidence',
    json.unavailableEvidence?.[0]?.status === 'UNAVAILABLE',
    JSON.stringify(json.unavailableEvidence));
  assert('N4. renderJson preserves cuePolicy',
    json.cuePolicy?.tokenPolicyVersion === 'prewrite-token-policy-v1',
    JSON.stringify(json.cuePolicy));
}

// ═══ Cue-tier Markdown sections and wording guard ═══

{
  const advisory = {
    invocationId: 'cue-md-test',
    intentHash: 'cue-md-hash',
    intent: { names: ['useShallowFromState'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
    lookups: [],
    cueCards: [{
      candidate: {
        identity: 'src/shallow.ts::useShallow',
        ownerFile: 'src/shallow.ts',
        exportedName: 'useShallow',
      },
      renderTier: 'AGENT_REVIEW_CUE',
      cues: [{
        cueTier: 'SAFE_CUE',
        safeMeaning: 'claim-only',
        notSafeFor: ['semantic-equivalence', 'auto-reuse', 'auto-fix'],
        evidenceLane: 'function-signature',
        claim: 'same normalized function signature',
        confidence: 'grounded',
        evidence: [{ artifact: 'function-clones.json', matchedField: 'normalizedSignatureHash' }],
      }, {
        cueTier: 'AGENT_REVIEW_CUE',
        evidenceLane: 'near-name',
        claim: 'near exported name',
        confidence: 'heuristic-review',
        evidence: [{ artifact: 'symbols.json', matchedField: 'defIndex', distance: 2 }],
      }],
    }],
    suppressedCues: [{
      cueTier: 'MUTED_CUE',
      evidenceLane: 'intent-token',
      reason: 'domain-token-overlap',
      tokens: ['use'],
      candidateCount: 1,
      tokenizerVersion: 'camel-snake-kebab-digit-v1',
      tokenPolicyVersion: 'prewrite-token-policy-v1',
    }],
    unavailableEvidence: [{
      evidenceLane: 'shape-hash',
      status: 'UNAVAILABLE',
      reason: 'missing-artifact',
      artifact: 'shape-index.json',
    }],
    cuePolicy: { tokenizerVersion: 'camel-snake-kebab-digit-v1', tokenPolicyVersion: 'prewrite-token-policy-v1' },
    boundaryChecks: [],
    drift: [],
    capabilities: null,
    failures: [],
  };
  const md = renderMarkdown(advisory);
  assert('O1. Grounded facts section renders SAFE_CUE claim',
    md.includes('### Grounded facts') &&
    md.includes('same normalized function signature') &&
    md.includes('src/shallow.ts::useShallow'),
    md);
  assert('O2. Agent review cues section renders review cue separately',
    md.includes('### Agent review cues') &&
    md.includes('near exported name'),
    md);
  assert('O3. Muted cue details are not rendered by default',
    !md.includes('domain-token-overlap') && !md.includes('Muted noise'),
    md);
  assert('O4. Unavailable evidence section renders lane status',
    md.includes('### Unavailable evidence') &&
    md.includes('shape-index.json'),
    md);
  assert('O5. renderer avoids semantic/reuse-forcing wording',
    !/does the same thing|semantically equivalent|reuse this/i.test(md),
    md);
}

{
  const advisory = {
    invocationId: 'cue-md-dedupe-test',
    intentHash: 'cue-md-dedupe-hash',
    intent: { names: ['formatDate'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
    lookups: [{
      kind: 'name',
      intentName: 'formatDate',
      result: 'EXISTS',
      identities: [{
        identity: 'src/date.ts::formatDate',
        ownerFile: 'src/date.ts',
        exportedName: 'formatDate',
      }],
      nearNames: [],
      semanticHints: [],
      suppressedSemanticHints: [],
    }],
    cueCards: [{
      candidate: {
        identity: 'src/date.ts::formatDate',
        ownerFile: 'src/date.ts',
        exportedName: 'formatDate',
      },
      renderTier: 'SAFE_CUE',
      cues: [{
        cueTier: 'SAFE_CUE',
        safeMeaning: 'claim-only',
        notSafeFor: ['semantic-equivalence', 'auto-reuse', 'auto-fix'],
        evidenceLane: 'exact-symbol',
        claim: 'exact exported symbol exists',
        confidence: 'grounded',
        evidence: [{ artifact: 'symbols.json', matchedField: 'defIndex' }],
      }],
    }],
    suppressedCues: [],
    unavailableEvidence: [],
    cuePolicy: { tokenizerVersion: 'camel-snake-kebab-digit-v1', tokenPolicyVersion: 'prewrite-token-policy-v1' },
    boundaryChecks: [],
    drift: [],
    capabilities: null,
    failures: [],
  };
  const md = renderMarkdown(advisory);
  assert('O6. exact cue-covered candidate appears once in default Markdown',
    (md.match(/src\/date\.ts::formatDate/g) ?? []).length === 1,
    md);
}

{
  const advisory = {
    invocationId: 'service-operation-cue-md-test',
    intentHash: 'service-operation-cue-md-hash',
    intent: { names: ['searchUser'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
    lookups: [],
    cueCards: [{
      candidate: {
        identity: 'src/services/user.ts::fetchUser',
        ownerFile: 'src/services/user.ts',
        exportedName: 'fetchUser',
      },
      renderTier: 'AGENT_REVIEW_CUE',
      cues: [{
        cueTier: 'AGENT_REVIEW_CUE',
        evidenceLane: 'service-operation-sibling',
        claim: 'related service operation sibling',
        confidence: 'heuristic-review',
        evidence: [{
          artifact: 'pre-write-advisory.json',
          matchedField: 'lookups[].serviceOperationSiblingPolicy.promoted',
          policyId: 'prewrite-service-operation-sibling-cue',
          policyVersion: 'prewrite-service-operation-sibling-cue-v1',
          candidateIdentity: 'src/services/user.ts::fetchUser',
          operationFamily: 'read-query',
          sharedDomainTokens: ['user'],
          locality: { sameDir: true, sameFile: false },
          supportingReasons: ['near-distance-exceeded', 'single-non-weak-token-only'],
        }],
      }],
    }],
    suppressedCues: [{
      cueTier: 'MUTED_CUE',
      evidenceLane: 'service-operation-sibling',
      reason: 'service-sibling-operation-family-mismatch',
      identity: 'src/services/user.ts::fetchUser',
      policyVersion: 'prewrite-service-operation-sibling-cue-v1',
    }],
    unavailableEvidence: [],
    cuePolicy: { tokenizerVersion: 'camel-snake-kebab-digit-v1', tokenPolicyVersion: 'prewrite-token-policy-v1' },
    boundaryChecks: [],
    drift: [],
    capabilities: null,
    failures: [],
  };
  const md = renderMarkdown(advisory);
  assert('P2b. service operation cue renders explicit review wording',
    md.includes('Review related service operation: `fetchUser` in `src/services/user.ts`.') &&
      md.includes('shared domain tokens: `user`; operation family: `read-query`; locality: sameDir.') &&
      md.includes('supporting suppressed reasons: `near-distance-exceeded`, `single-non-weak-token-only`.'),
    md);
  assert('P2b. service operation cue cites the policy evidence path',
    md.includes('pre-write-advisory.json / lookups[].serviceOperationSiblingPolicy.promoted') &&
      md.includes('policy prewrite-service-operation-sibling-cue-v1'),
    md);
  assert('P2b. muted service operation details remain hidden by default',
    !md.includes('service-sibling-operation-family-mismatch'),
    md);
  assert('P2b. service operation renderer avoids strong action wording',
    !/\b(reuse|equivalent|safe|exists|should call|blocking failure)\b/i.test(md),
    md);
}

{
  const advisory = {
    invocationId: 'evidence-availability-test',
    intentHash: 'evidence-availability-hash',
    intent: { names: ['KlarnaPaymentService'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
    evidenceAvailability: {
      status: 'missing',
      freshAudit: false,
      output: '.audit',
      guidance: 'Run a baseline audit with the same `--output`, or rerun pre-write without `--no-fresh-audit`.',
      artifacts: [
        {
          artifact: 'symbols.json',
          status: 'missing',
          requiredFor: ['names'],
          reason: 'symbols.json missing in .audit',
        },
      ],
    },
    lookups: [],
    cueCards: [],
    suppressedCues: [],
    unavailableEvidence: [],
    cuePolicy: null,
    boundaryChecks: [],
    drift: [],
    capabilities: null,
    failures: [],
  };
  const md = renderMarkdown(advisory);
  const json = renderJson(advisory);
  assert('O7. evidence availability warning renders before lookup sections',
    md.includes('### Evidence availability') &&
      md.includes('symbols.json') &&
      md.includes('same `--output`') &&
      md.includes('not grounded absence'),
    md);
  assert('O8. evidenceAvailability survives JSON render',
    json.evidenceAvailability?.status === 'missing' &&
      json.evidenceAvailability?.artifacts?.[0]?.artifact === 'symbols.json',
    JSON.stringify(json.evidenceAvailability));
}

{
  const advisory = {
    invocationId: 'local-operation-cue-md-test',
    intentHash: 'local-operation-cue-md-hash',
    intent: { names: ['searchWorld'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
    lookups: [],
    cueCards: [{
      candidate: {
        identity: 'src/repository.ts::createRepository#getWorld',
        ownerFile: 'src/repository.ts',
        exportedName: 'getWorld',
      },
      renderTier: 'AGENT_REVIEW_CUE',
      cues: [{
        cueTier: 'AGENT_REVIEW_CUE',
        evidenceLane: 'local-operation-sibling',
        claim: 'related local service operation',
        confidence: 'heuristic-review',
        evidence: [{
          artifact: 'pre-write-advisory.json',
          matchedField: 'lookups[].localOperationSiblingPolicy.promoted',
          policyId: 'prewrite-local-operation-sibling',
          policyVersion: 'prewrite-local-operation-sibling-v1',
          candidateIdentity: 'src/repository.ts::createRepository#getWorld',
          surfaceKind: 'nested-local-operation',
          containerName: 'createRepository',
          containerKind: 'function-declaration',
          operationFamily: 'read-query',
          sharedDomainTokens: ['world'],
          locality: { sameDir: true, sameFile: true },
          supportingReasons: ['local-operation-same-file-domain-overlap'],
        }],
      }],
    }],
    suppressedCues: [{
      cueTier: 'MUTED_CUE',
      evidenceLane: 'local-operation-sibling',
      reason: 'local-operation-operation-family-mismatch',
      identity: 'src/repository.ts::createRepository#getWorld',
      policyVersion: 'prewrite-local-operation-sibling-v1',
    }],
    unavailableEvidence: [],
    cuePolicy: { tokenizerVersion: 'camel-snake-kebab-digit-v1', tokenPolicyVersion: 'prewrite-token-policy-v1' },
    boundaryChecks: [],
    drift: [],
    capabilities: null,
    failures: [],
  };
  const md = renderMarkdown(advisory);
  assert('P2b local. local operation cue renders explicit review wording',
    md.includes('Review related local service operation: `getWorld` inside `createRepository` in `src/repository.ts`.') &&
      md.includes('shared domain tokens: `world`; operation family: `read-query`; locality: sameFile, sameDir.') &&
      md.includes('supporting local-operation reasons: `local-operation-same-file-domain-overlap`.') &&
      !md.includes('supporting local-operation reasons: `unknown`.'),
    md);
  assert('P2b local. local operation cue cites the policy evidence path',
    md.includes('pre-write-advisory.json / lookups[].localOperationSiblingPolicy.promoted') &&
      md.includes('policy prewrite-local-operation-sibling-v1'),
    md);
  assert('P2b local. muted local operation details remain hidden by default',
    !md.includes('local-operation-operation-family-mismatch'),
    md);
  assert('P2b local. local operation renderer avoids strong action wording',
    !/\b(reuse|equivalent|safe|exists|should call|blocking failure)\b/i.test(md),
    md);
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
