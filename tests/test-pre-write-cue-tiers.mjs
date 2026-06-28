import {
  classifyPreWriteCues,
  CUE_TIERS,
  UNAVAILABLE_STATUS,
} from '../_lib/pre-write-cue-tiers.mjs';
import { tokenizePreWrite } from '../_lib/pre-write-token-policy.mjs';

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

function findCard(result, identity) {
  return result.cueCards.find((card) => card.candidate?.identity === identity);
}

{
  const result = classifyPreWriteCues({
    lookups: [{
      kind: 'name',
      intentName: 'formatDate',
      result: 'EXISTS',
      identities: [{
        identity: 'src/date.ts::formatDate',
        ownerFile: 'src/date.ts',
        exportedName: 'formatDate',
        fanIn: 3,
        fanInConfidence: 'grounded',
        citations: [`[grounded, symbols.json.fanInByIdentity['src/date.ts::formatDate'] = 3]`],
      }],
      nearNames: [],
      semanticHints: [],
      suppressedSemanticHints: [],
    }],
    intent: { names: ['formatDate'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
  });
  const card = findCard(result, 'src/date.ts::formatDate');
  assert('T1. exact symbol identity creates a cue card',
    !!card, JSON.stringify(result, null, 2));
  assert('T1b. exact symbol cue is SAFE_CUE claim-only',
    card.cues.some((cue) =>
      cue.cueTier === CUE_TIERS.SAFE &&
      cue.safeMeaning === 'claim-only' &&
      cue.evidenceLane === 'exact-symbol' &&
      cue.notSafeFor.includes('semantic-equivalence')),
    JSON.stringify(card, null, 2));
}

{
  const result = classifyPreWriteCues({
    lookups: [{
      kind: 'shape',
      result: 'SIGNATURE_MATCH',
      shapeHash: 'sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc',
      shapeHashSource: 'functionSignature',
      signature: '(string):string',
      matches: [{
        identity: 'src/user-a.ts::normalizeUserName',
        ownerFile: 'src/user-a.ts',
        exportedName: 'normalizeUserName',
        localName: 'normalizeUserName',
        visibility: 'file-local',
        exported: false,
        confidence: 'high',
      }],
      citations: ['[grounded, function-clones.json facts[] matched 1 identities]'],
    }],
    intent: { names: ['normalizeUserName'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
  });
  const card = findCard(result, 'src/user-a.ts::normalizeUserName');
  assert('T2c. file-local function signature match creates a review cue card',
    !!card && card.renderTier === CUE_TIERS.AGENT_REVIEW,
    JSON.stringify(result, null, 2));
  assert('T2d. file-local function signature match is not promoted to SAFE_CUE',
    card.cues.some((cue) =>
      cue.cueTier === CUE_TIERS.AGENT_REVIEW &&
      cue.evidenceLane === 'function-signature' &&
      cue.evidence?.[0]?.visibility === 'file-local') &&
      !card.cues.some((cue) => cue.cueTier === CUE_TIERS.SAFE),
    JSON.stringify(card, null, 2));
}

{
  const result = classifyPreWriteCues({
    lookups: [{
      kind: 'shape',
      result: 'SIGNATURE_MATCH',
      shapeHash: 'sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd',
      shapeHashSource: 'functionSignature',
      signature: '(string):string',
      matches: [{
        identity: 'src/default-fn.ts::default',
        ownerFile: 'src/default-fn.ts',
        exportedName: 'default',
        localName: 'normalizePayload',
        visibility: 'exported',
        exported: true,
        confidence: 'high',
      }],
      citations: ['[grounded, function-clones.json facts[] matched 1 identities]'],
    }],
    intent: { names: ['normalizePayload'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
  });
  const card = findCard(result, 'src/default-fn.ts::default');
  const cue = card?.cues?.find((entry) => entry.evidenceLane === 'function-signature');
  assert('T2e. exported default function signature match remains SAFE_CUE',
    card?.renderTier === CUE_TIERS.SAFE &&
      cue?.cueTier === CUE_TIERS.SAFE &&
      cue.evidence?.[0]?.visibility === 'exported' &&
      cue.evidence?.[0]?.localName === 'normalizePayload',
    JSON.stringify(result, null, 2));
}

{
  const result = classifyPreWriteCues({
    lookups: [{
      kind: 'shape',
      result: 'SIGNATURE_MATCH',
      shapeHash: 'sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
      shapeHashSource: 'functionSignature',
      signature: '<S,U>((S)=>U):(S)=>U',
      matches: [{
        identity: 'src/shallow.ts::useShallow',
        ownerFile: 'src/shallow.ts',
        exportedName: 'useShallow',
        confidence: 'medium',
      }],
      citations: ['[grounded, function-clones.json facts[] matched 1 identities]'],
    }],
    intent: { names: ['composeProjection'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
  });
  const card = findCard(result, 'src/shallow.ts::useShallow');
  assert('T2. function signature match creates cue card',
    !!card, JSON.stringify(result, null, 2));
  assert('T2b. function signature cue is SAFE_CUE',
    card.cues.some((cue) =>
      cue.cueTier === CUE_TIERS.SAFE &&
      cue.evidenceLane === 'function-signature' &&
      cue.claim === 'same normalized function signature'),
    JSON.stringify(card, null, 2));
}

{
  const result = classifyPreWriteCues({
    lookups: [{
      kind: 'name',
      intentName: 'useShallowFromState',
      result: 'NOT_OBSERVED',
      identities: [],
      nearNames: [{ name: 'useShallow', ownerFile: 'src/shallow.ts', distance: 2 }],
      semanticHints: [],
      suppressedSemanticHints: [],
      citations: ['[degraded, fuzzy-name match; search hint only]'],
    }, {
      kind: 'shape',
      result: 'SIGNATURE_MATCH',
      shapeHash: 'sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb',
      shapeHashSource: 'functionSignature',
      signature: '<S,U>((S)=>U):(S)=>U',
      matches: [{
        identity: 'src/shallow.ts::useShallow',
        ownerFile: 'src/shallow.ts',
        exportedName: 'useShallow',
      }],
      citations: ['[grounded, function-clones.json facts[] matched 1 identities]'],
    }],
    intent: { names: ['useShallowFromState'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
  });
  const card = findCard(result, 'src/shallow.ts::useShallow');
  const tiers = new Set(card?.cues?.map((cue) => cue.cueTier));
  assert('T3. one candidate can carry SAFE_CUE and AGENT_REVIEW_CUE together',
    tiers.has(CUE_TIERS.SAFE) && tiers.has(CUE_TIERS.AGENT_REVIEW),
    JSON.stringify(card, null, 2));
  assert('T3b. mixed candidate renderTier is AGENT_REVIEW_CUE',
    card?.renderTier === CUE_TIERS.AGENT_REVIEW,
    JSON.stringify(card, null, 2));
}

{
  const result = classifyPreWriteCues({
    lookups: [{
      kind: 'name',
      intentName: 'handleBulkDelete',
      result: 'NOT_OBSERVED',
      identities: [],
      nearNames: [{
        name: 'handleDelete',
        ownerFile: 'src/event-dispatcher.ts',
        identity: 'src/event-dispatcher.ts::TaskControlEventDispatcher#handleDelete',
        className: 'TaskControlEventDispatcher',
        distance: 4,
        matchedField: 'classMethodIndex',
      }],
      semanticHints: [],
      suppressedSemanticHints: [],
      citations: ['[degraded, class method search hint only]'],
    }],
    intent: { names: ['handleBulkDelete'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
  });
  const card = findCard(result, 'src/event-dispatcher.ts::TaskControlEventDispatcher#handleDelete');
  const cue = card?.cues?.[0];
  assert('T3c. class method near-name creates review cue card',
    !!card &&
    cue?.cueTier === CUE_TIERS.AGENT_REVIEW &&
    cue.evidenceLane === 'class-method-name' &&
    cue.claim === 'near class method name',
    JSON.stringify(result, null, 2));
  assert('T3d. class method cue cites classMethodIndex, not defIndex',
    cue?.evidence?.[0]?.matchedField === 'classMethodIndex' &&
    cue?.evidence?.[0]?.candidateIdentity === 'src/event-dispatcher.ts::TaskControlEventDispatcher#handleDelete',
    JSON.stringify(cue, null, 2));
}

{
  const result = classifyPreWriteCues({
    lookups: [{
      kind: 'name',
      intentName: 'createLogger',
      result: 'NOT_OBSERVED',
      identities: [],
      nearNames: [],
      semanticHints: [],
      suppressedSemanticHints: [{
        name: 'createStore',
        ownerFile: 'src/store.ts',
        matchedTokens: ['create'],
        reason: 'domain-token-overlap',
        candidateCount: 2,
      }, {
        name: 'createJSONStorage',
        ownerFile: 'src/storage.ts',
        matchedTokens: ['create'],
        reason: 'domain-token-overlap',
        candidateCount: 2,
      }],
    }],
    intent: { names: ['createLogger'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
  });
  assert('T4. weak common token candidates become suppressed cues',
    result.suppressedCues.length === 2 &&
    result.suppressedCues.every((cue) =>
      cue.cueTier === CUE_TIERS.MUTED &&
      cue.reason === 'domain-token-overlap' &&
      cue.tokenPolicyVersion === 'prewrite-token-policy-v1'),
    JSON.stringify(result.suppressedCues, null, 2));
}

{
  const result = classifyPreWriteCues({
    lookups: [{
      kind: 'name',
      intentName: 'searchUser',
      result: 'NOT_OBSERVED',
      identities: [],
      nearNames: [],
      semanticHints: [],
      suppressedNearNames: [{
        name: 'fetchUser',
        ownerFile: 'src/services/user.ts',
        matchedTokens: ['user'],
        distance: 3,
        reason: 'near-distance-exceeded',
        locality: { sameDir: true, sameFile: false },
        candidateCount: 1,
      }],
      suppressedSemanticHints: [{
        name: 'fetchUser',
        ownerFile: 'src/services/user.ts',
        matchedTokens: ['user'],
        score: 1,
        reason: 'single-non-weak-token-only',
        locality: { sameDir: true, sameFile: false },
        candidateCount: 1,
      }],
    }],
    intent: { names: ['searchUser'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
  });
  assert('T4b. suppressed near/semantic diagnostics stay muted and do not create cards',
    result.cueCards.length === 0 &&
      result.suppressedCues.length === 2 &&
      result.suppressedCues.some((cue) =>
        cue.evidenceLane === 'near-name' &&
        cue.reason === 'near-distance-exceeded' &&
        cue.distance === 3 &&
        cue.locality?.sameDir === true) &&
      result.suppressedCues.some((cue) =>
        cue.evidenceLane === 'intent-token' &&
        cue.reason === 'single-non-weak-token-only' &&
        cue.score === 1 &&
        cue.locality?.sameDir === true),
    JSON.stringify(result, null, 2));
}

{
  const result = classifyPreWriteCues({
    lookups: [{
      kind: 'name',
      intentName: 'searchUser',
      result: 'NOT_OBSERVED',
      identities: [],
      nearNames: [],
      semanticHints: [],
      suppressedNearNames: [{
        name: 'fetchUser',
        ownerFile: 'src/services/user.ts',
        matchedTokens: ['user'],
        distance: 3,
        reason: 'near-distance-exceeded',
        locality: { sameDir: true, sameFile: false },
        candidateCount: 1,
      }],
      suppressedSemanticHints: [{
        name: 'fetchUser',
        ownerFile: 'src/services/user.ts',
        matchedTokens: ['user'],
        score: 1,
        reason: 'single-non-weak-token-only',
        locality: { sameDir: true, sameFile: false },
        candidateCount: 1,
      }],
      serviceOperationSiblingPolicy: {
        policyId: 'prewrite-service-operation-sibling-cue',
        policyVersion: 'prewrite-service-operation-sibling-cue-v1',
        promoted: [{
          identity: 'src/services/user.ts::fetchUser',
          name: 'fetchUser',
          ownerFile: 'src/services/user.ts',
          operationFamily: 'read-query',
          sharedDomainTokens: ['user'],
          supportingReasons: ['near-distance-exceeded', 'single-non-weak-token-only'],
          locality: { sameDir: true, sameFile: false },
          signatureSupport: { status: 'unavailable', reason: 'no-signature-facts' },
        }],
        muted: [],
      },
    }],
    intent: { names: ['searchUser'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
  });
  const card = findCard(result, 'src/services/user.ts::fetchUser');
  const cue = card?.cues?.find((entry) => entry.evidenceLane === 'service-operation-sibling');
  assert('T4c. promoted service-operation sibling creates review cue card',
    !!card &&
      card.renderTier === CUE_TIERS.AGENT_REVIEW &&
      cue?.cueTier === CUE_TIERS.AGENT_REVIEW &&
      cue.claim === 'related service operation sibling' &&
      cue.confidence === 'heuristic-review',
    JSON.stringify(result, null, 2));
  assert('T4d. service-operation sibling cue copies policy evidence without safe claims',
    cue?.evidence?.[0]?.artifact === 'pre-write-advisory.json' &&
      cue.evidence[0].matchedField === 'lookups[].serviceOperationSiblingPolicy.promoted' &&
      cue.evidence[0].policyId === 'prewrite-service-operation-sibling-cue' &&
      cue.evidence[0].policyVersion === 'prewrite-service-operation-sibling-cue-v1' &&
      cue.evidence[0].candidateIdentity === 'src/services/user.ts::fetchUser' &&
      cue.evidence[0].operationFamily === 'read-query' &&
      cue.evidence[0].sharedDomainTokens?.[0] === 'user' &&
      !card.cues.some((entry) => entry.cueTier === CUE_TIERS.SAFE),
    JSON.stringify(card, null, 2));
  assert('T4e. service-operation sibling cue preserves original suppressed diagnostics',
    result.suppressedCues.length === 2 &&
      result.suppressedCues.some((entry) => entry.reason === 'near-distance-exceeded') &&
      result.suppressedCues.some((entry) => entry.reason === 'single-non-weak-token-only'),
    JSON.stringify(result.suppressedCues, null, 2));
}

{
  const result = classifyPreWriteCues({
    lookups: [{
      kind: 'name',
      intentName: 'createUser',
      result: 'NOT_OBSERVED',
      identities: [],
      nearNames: [],
      semanticHints: [],
      suppressedNearNames: [],
      suppressedSemanticHints: [],
      serviceOperationSiblingPolicy: {
        policyId: 'prewrite-service-operation-sibling-cue',
        policyVersion: 'prewrite-service-operation-sibling-cue-v1',
        promoted: [],
        muted: [{
          identity: 'src/services/user.ts::fetchUser',
          name: 'fetchUser',
          ownerFile: 'src/services/user.ts',
          reason: 'service-sibling-operation-family-mismatch',
          operationFamily: 'read-query',
          sharedDomainTokens: ['user'],
        }],
      },
    }],
    intent: { names: ['createUser'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
  });
  assert('T4f. muted service-operation sibling stays out of cueCards',
    result.cueCards.length === 0 &&
      result.suppressedCues.some((entry) =>
        entry.evidenceLane === 'service-operation-sibling' &&
        entry.reason === 'service-sibling-operation-family-mismatch' &&
        entry.identity === 'src/services/user.ts::fetchUser'),
    JSON.stringify(result, null, 2));
}

{
  const result = classifyPreWriteCues({
    lookups: [{
      kind: 'name',
      intentName: 'searchUser',
      result: 'NOT_OBSERVED',
      identities: [],
      nearNames: [],
      semanticHints: [],
      suppressedNearNames: [],
      suppressedSemanticHints: [],
      serviceOperationSiblingPolicy: {
        policyId: 'prewrite-service-operation-sibling-cue',
        policyVersion: 'prewrite-service-operation-sibling-cue-v1',
        promoted: [{
          identity: 'src/event-dispatcher.ts::TaskControlEventDispatcher#searchUser',
          name: 'searchUser',
          ownerFile: 'src/event-dispatcher.ts',
          matchedField: 'classMethodIndex',
          operationFamily: 'read-query',
          sharedDomainTokens: ['user'],
        }, {
          identity: 'dist/generated/user.ts::fetchUser',
          name: 'fetchUser',
          ownerFile: 'dist/generated/user.ts',
          operationFamily: 'read-query',
          sharedDomainTokens: ['user'],
        }],
        muted: [],
      },
    }],
    intent: { names: ['searchUser'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
  });
  assert('T4g. service-operation adapter does not render class methods or generated paths',
    result.cueCards.length === 0 &&
      result.suppressedCues.some((entry) =>
        entry.evidenceLane === 'service-operation-sibling' &&
        entry.reason === 'service-sibling-class-method-lane') &&
      result.suppressedCues.some((entry) =>
        entry.reason === 'policy-excluded' &&
        entry.policyReason === 'path:dist'),
    JSON.stringify(result, null, 2));
}

{
  const result = classifyPreWriteCues({
    lookups: [{
      kind: 'name',
      intentName: 'searchWorld',
      result: 'NOT_OBSERVED',
      identities: [],
      nearNames: [],
      semanticHints: [],
      suppressedNearNames: [],
      suppressedSemanticHints: [],
      serviceOperationSiblingPolicy: {
        policyId: 'prewrite-service-operation-sibling-cue',
        policyVersion: 'prewrite-service-operation-sibling-cue-v1',
        promoted: [],
        muted: [],
      },
      localOperationSiblingPolicy: {
        policyId: 'prewrite-local-operation-sibling',
        policyVersion: 'prewrite-local-operation-sibling-v1',
        status: 'complete',
        promoted: [{
          identity: 'src/repository.ts::createRepository#getWorld',
          name: 'getWorld',
          ownerFile: 'src/repository.ts',
          matchedField: 'preWriteLocalOperationIndex',
          surfaceKind: 'nested-local-operation',
          containerName: 'createRepository',
          containerKind: 'function-declaration',
          operationFamily: 'read-query',
          sharedDomainTokens: ['world'],
          supportingReasons: ['local-operation-same-file-domain-overlap'],
          locality: { sameDir: true, sameFile: true },
          eligibleForDeadExportRanking: false,
          eligibleForSafeFix: false,
        }],
        muted: [],
      },
    }],
    intent: { names: ['searchWorld'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
  });
  const card = findCard(result, 'src/repository.ts::createRepository#getWorld');
  const cue = card?.cues?.find((entry) => entry.evidenceLane === 'local-operation-sibling');
  assert('T4h. promoted local-operation sibling creates review cue card',
    !!card &&
      card.renderTier === CUE_TIERS.AGENT_REVIEW &&
      cue?.cueTier === CUE_TIERS.AGENT_REVIEW &&
      cue.claim === 'related local service operation' &&
      cue.confidence === 'heuristic-review',
    JSON.stringify(result, null, 2));
  assert('T4i. local-operation cue copies policy evidence without safe claims',
    cue?.evidence?.[0]?.artifact === 'pre-write-advisory.json' &&
      cue.evidence[0].matchedField === 'lookups[].localOperationSiblingPolicy.promoted' &&
      cue.evidence[0].policyId === 'prewrite-local-operation-sibling' &&
      cue.evidence[0].policyVersion === 'prewrite-local-operation-sibling-v1' &&
      cue.evidence[0].candidateIdentity === 'src/repository.ts::createRepository#getWorld' &&
      cue.evidence[0].containerName === 'createRepository' &&
      cue.evidence[0].surfaceKind === 'nested-local-operation' &&
      cue.evidence[0].operationFamily === 'read-query' &&
      cue.evidence[0].sharedDomainTokens?.[0] === 'world' &&
      cue.evidence[0].supportingReasons?.[0] === 'local-operation-same-file-domain-overlap' &&
      cue.evidence[0].locality?.sameFile === true &&
      !card.cues.some((entry) => entry.cueTier === CUE_TIERS.SAFE),
    JSON.stringify(card, null, 2));
}

{
  const result = classifyPreWriteCues({
    lookups: [{
      kind: 'name',
      intentName: 'deleteWorld',
      result: 'NOT_OBSERVED',
      identities: [],
      nearNames: [],
      semanticHints: [],
      suppressedNearNames: [],
      suppressedSemanticHints: [],
      localOperationSiblingPolicy: {
        policyId: 'prewrite-local-operation-sibling',
        policyVersion: 'prewrite-local-operation-sibling-v1',
        status: 'complete',
        promoted: [],
        muted: [{
          identity: 'src/repository.ts::createRepository#getWorld',
          name: 'getWorld',
          ownerFile: 'src/repository.ts',
          reason: 'local-operation-operation-family-mismatch',
          matchedField: 'preWriteLocalOperationIndex',
          surfaceKind: 'nested-local-operation',
          containerName: 'createRepository',
          containerKind: 'function-declaration',
          operationFamily: 'read-query',
          sharedDomainTokens: ['world'],
          locality: { sameDir: true, sameFile: true },
        }],
      },
    }],
    intent: { names: ['deleteWorld'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
  });
  assert('T4j. muted local-operation sibling stays out of cueCards',
    result.cueCards.length === 0 &&
      result.suppressedCues.some((entry) =>
        entry.evidenceLane === 'local-operation-sibling' &&
        entry.reason === 'local-operation-operation-family-mismatch' &&
        entry.identity === 'src/repository.ts::createRepository#getWorld'),
    JSON.stringify(result, null, 2));
}

{
  const result = classifyPreWriteCues({
    lookups: [{
      kind: 'shape',
      shape: { fields: [], typeLiteral: '<S>(selector: (state: S) => S) => S' },
      result: 'UNAVAILABLE',
      reason: 'missing-artifact',
      artifact: 'function-clones.json',
      citations: ['[확인 불가, function-clones.json absent]'],
    }],
    intent: { names: [], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
  });
  assert('T5. unavailable evidence is separate from suppressed cues',
    result.unavailableEvidence.length === 1 &&
    result.unavailableEvidence[0].status === UNAVAILABLE_STATUS &&
    result.suppressedCues.length === 0,
    JSON.stringify(result, null, 2));
  assert('T5b. unavailable evidence preserves lookup reason and artifact',
    result.unavailableEvidence[0].reason === 'missing-artifact' &&
    result.unavailableEvidence[0].artifact === 'function-clones.json',
    JSON.stringify(result.unavailableEvidence, null, 2));
}

{
  const result = classifyPreWriteCues({
    lookups: [{
      kind: 'name',
      intentName: 'generatedHelper',
      result: 'EXISTS',
      identities: [{
        identity: 'dist/generated.ts::generatedHelper',
        ownerFile: 'dist/generated.ts',
        exportedName: 'generatedHelper',
        policyExcluded: true,
        policyReason: 'generated-output',
        citations: ['[grounded, exact symbol exists]'],
      }],
      nearNames: [],
      semanticHints: [],
      suppressedSemanticHints: [],
    }],
    intent: { names: ['generatedHelper'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
  });
  assert('T6. policy-excluded exact evidence stays out of cueCards',
    result.cueCards.length === 0,
    JSON.stringify(result, null, 2));
  assert('T6b. policy-excluded exact evidence remains in suppressed cues',
    result.suppressedCues.some((cue) =>
      cue.reason === 'policy-excluded' &&
      cue.policyReason === 'generated-output' &&
      cue.originalCueTier === CUE_TIERS.SAFE &&
      cue.claim === 'exact exported symbol exists' &&
      cue.evidence?.[0]?.artifact === 'symbols.json'),
    JSON.stringify(result.suppressedCues, null, 2));
}

{
  const result = classifyPreWriteCues({
    lookups: [{
      kind: 'file',
      intentFile: 'src/logger.ts',
      result: 'FILE_EXISTS',
    }],
    intent: { names: [], shapes: [], files: ['src/logger.ts'], dependencies: [], plannedTypeEscapes: [] },
  });
  const card = findCard(result, 'src/logger.ts::__file__');
  assert('T7. exact file exists creates SAFE_CUE',
    card?.cues?.some((cue) =>
      cue.cueTier === CUE_TIERS.SAFE &&
      cue.evidenceLane === 'exact-file' &&
      cue.claim === 'exact file exists'),
    JSON.stringify(result, null, 2));
}

{
  assert('T8. token policy preserves class/process/status stems',
    tokenizePreWrite('className').includes('class') &&
    tokenizePreWrite('processConfig').includes('process') &&
    tokenizePreWrite('statusCheck').includes('status') &&
    tokenizePreWrite('analysisReport').includes('analysis'),
    JSON.stringify({
      className: tokenizePreWrite('className'),
      processConfig: tokenizePreWrite('processConfig'),
      statusCheck: tokenizePreWrite('statusCheck'),
      analysisReport: tokenizePreWrite('analysisReport'),
    }, null, 2));
}

{
  const result = classifyPreWriteCues({
    lookups: [{
      kind: 'inline-pattern',
      result: 'INLINE_PATTERN_MATCH',
      groups: [{
        patternHash: 'sha256:catch-destroy',
        kind: 'catch-block',
        size: 4,
        ownerFiles: ['src/server.ts'],
        occurrences: [
          { file: 'src/server.ts', line: 498, endLine: 500 },
          { file: 'src/server.ts', line: 577, endLine: 579 },
        ],
        reviewReason: 'same normalized catch block; verify socket ownership before extracting',
      }],
    }],
    intent: { names: ['writeOrDestroyConnection'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
  });
  const card = findCard(result, 'inline-pattern:sha256:catch-destroy');
  const cue = card?.cues?.[0];
  assert('T9. inline pattern match creates review cue card',
    !!card && cue?.cueTier === CUE_TIERS.AGENT_REVIEW,
    JSON.stringify(result, null, 2));
  assert('T9b. inline pattern cue is review-only structured evidence',
    cue?.evidenceLane === 'inline-extraction' &&
    cue?.claim === 'repeated inline statement pattern' &&
    cue?.evidence?.[0]?.artifact === 'inline-patterns.json' &&
    cue?.evidence?.[0]?.occurrenceCount === 4,
    JSON.stringify(cue, null, 2));
}

{
  const result = classifyPreWriteCues({
    lookups: [{
      kind: 'inline-pattern',
      result: 'UNAVAILABLE',
      reason: 'missing-artifact',
      artifact: 'inline-patterns.json',
      citations: ['[확인 불가, inline-patterns.json absent]'],
    }],
    intent: { names: [], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
  });
  assert('T10. missing inline-patterns artifact becomes unavailable evidence',
    result.unavailableEvidence.length === 1 &&
    result.unavailableEvidence[0].evidenceLane === 'inline-extraction' &&
    result.unavailableEvidence[0].status === UNAVAILABLE_STATUS &&
    result.unavailableEvidence[0].reason === 'missing-artifact' &&
    result.suppressedCues.length === 0,
    JSON.stringify(result, null, 2));
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
