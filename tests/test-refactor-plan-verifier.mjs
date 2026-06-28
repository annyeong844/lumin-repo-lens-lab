// Refactor-plan output verifier: turns the template self-check into
// executable maintainer validation for sample model outputs.

import { execFileSync, spawnSync } from 'node:child_process';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { verifyRefactorPlan } from '../test-harness/lib/verify-refactor-plan.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');
const NODE = process.execPath;
const CLI = path.join(ROOT, 'test-harness/lib/verify-refactor-plan.mjs');
const TMP = mkdtempSync(path.join(tmpdir(), 'fx-refactor-plan-verify-'));

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

function sampleShort(extra = '') {
  return `**What Already Works**
- Dependencies are acyclic and parser confidence is high (topology.json, manifest.json).

**Next Slice**
Smooth the self-audit path so generated mirrors do not double-count the tool itself.
Touch \`audit-repo.mjs\` and scan-scope helpers. Leave dead-export classification alone.
Pre-write handoff: files=[audit-repo.mjs,_lib/collect-files.mjs], names=[], dependencies=[], shapes=[], plannedTypeEscapes=[].
Ask the coding agent: "Please make only this slice: smooth the self-audit path. Start with pre-write, touch only audit-repo.mjs and scan-scope helpers, leave dead-export classification alone, then run npm test and a quick self-audit."

**How We Verify**
Run \`npm test\`, then rerun a quick self-audit.
Success means file count and top fan-in stay near the focused baseline.

**After That**
- Clarify \`--exclude\` help text.
- Re-run \`check-canon\`.
${extra}`;
}

try {
  {
    const result = verifyRefactorPlan(sampleShort(), { mode: 'short', expectCodeChange: true });
    assert('R1. valid SHORT plan with code change + pre-write handoff passes',
      result.ok,
      JSON.stringify(result.errors));
  }

  {
    const result = verifyRefactorPlan(
      sampleShort()
        .replace(/^Pre-write handoff:.*\n/m, '')
        .replace('Start with pre-write', 'Start with a narrow implementation check'),
      {
      mode: 'short',
      expectCodeChange: true,
      },
    );
    assert('R2. code-changing SHORT plan without pre-write handoff fails',
      !result.ok && result.errors.some((e) => e.code === 'missing-prewrite-handoff'),
      JSON.stringify(result.errors));
  }

  {
    const result = verifyRefactorPlan(sampleShort().replace(/^Ask the coding agent:.*\n/m, ''), {
      mode: 'short',
      expectCodeChange: true,
    });
    assert('R2b. code-changing SHORT plan without coding-agent prompt fails',
      !result.ok && result.errors.some((e) => e.code === 'missing-coding-agent-prompt'),
      JSON.stringify(result.errors));
  }

  {
    const result = verifyRefactorPlan(sampleShort('\n```json\n{"files":["src/a.ts"]}\n```\n'), {
      mode: 'short',
      expectCodeChange: true,
    });
    assert('R3. raw JSON block in default chat plan fails',
      !result.ok && result.errors.some((e) => e.code === 'raw-json-in-chat'),
      JSON.stringify(result.errors));
  }

  {
    const result = verifyRefactorPlan(sampleShort('\nThis code is broken and terrible.\n'), {
      mode: 'short',
      expectCodeChange: true,
    });
    assert('R4. discouraging tone fails',
      !result.ok && result.errors.some((e) => e.code === 'discouraging-tone'),
      JSON.stringify(result.errors));
  }

  {
    const withoutEvidence = sampleShort().replace('(topology.json, manifest.json)', '(the audit)');
    const result = verifyRefactorPlan(withoutEvidence, { mode: 'short', expectCodeChange: true });
    assert('R5. missing artifact or claim-label evidence anchor fails',
      !result.ok && result.errors.some((e) => e.code === 'missing-evidence-anchor'),
      JSON.stringify(result.errors));
  }

  {
    const full = [
      '## What is already working',
      'topology.json confirms the baseline.',
      '## Goal in plain language',
      'Make one small change.',
      '## Evidence snapshot',
      'manifest.json parse errors: 0.',
      '## Phase map',
      'One phase.',
      '## Phase 1 slice spec',
      'Files in scope.',
      '## Phase 1 quick-audit scope',
      'Focused scan range.',
      '## Acceptance and verification',
      'Run npm test.',
      '## Risks and leave-alone list',
      'Leave unrelated cleanup alone.',
      '## Closeout loop',
      'Compare planned vs actual.',
    ].join('\n\n');
    const result = verifyRefactorPlan(full, { mode: 'full' });
    assert('R6. FULL handoff plan sections pass',
      result.ok,
      JSON.stringify(result.errors));
  }

  {
    const goodFile = path.join(TMP, 'good.md');
    writeFileSync(goodFile, sampleShort());
    const ok = execFileSync(NODE, [CLI, '--mode', 'short', '--expect-code-change', goodFile], {
      cwd: ROOT,
      encoding: 'utf8',
    });
    assert('R7. CLI exits 0 for valid plan',
      ok.includes('[verify-refactor-plan] OK'),
      ok);
  }

  {
    const badFile = path.join(TMP, 'bad.md');
    writeFileSync(badFile, sampleShort().replace('**How We Verify**', '**How We Guess**'));
    const bad = spawnSync(NODE, [CLI, '--mode', 'short', '--expect-code-change', badFile], {
      cwd: ROOT,
      encoding: 'utf8',
    });
    assert('R8. CLI exits non-zero and names missing verification section',
      bad.status === 1 && bad.stderr.includes('verification-section'),
      `${bad.status}\n${bad.stdout}\n${bad.stderr}`);
  }
} finally {
  rmSync(TMP, { recursive: true, force: true });
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
