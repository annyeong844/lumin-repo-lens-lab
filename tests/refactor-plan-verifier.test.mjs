import { execFileSync, spawnSync } from 'node:child_process';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

import { afterAll, describe, expect, it } from 'vitest';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');
const NODE = process.execPath;
const CLI = path.join(ROOT, 'test-harness/lib/verify-refactor-plan.mjs');
const CLI_URL = pathToFileURL(CLI).href;

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

function sampleFull() {
  return [
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
}

function verifyPlan(text, options = {}) {
  const code = [
    `import { verifyRefactorPlan } from ${JSON.stringify(CLI_URL)};`,
    'const payload = JSON.parse(process.env.LRL_TEST_PAYLOAD);',
    'const result = verifyRefactorPlan(payload.text, payload.options);',
    'process.stdout.write(JSON.stringify(result));',
  ].join('\n');
  const stdout = execFileSync(NODE, ['--input-type=module', '--eval', code], {
    cwd: ROOT,
    encoding: 'utf8',
    env: {
      ...process.env,
      LRL_TEST_PAYLOAD: JSON.stringify({ options, text }),
    },
  });
  return JSON.parse(stdout);
}

function runCli(args) {
  return spawnSync(NODE, [CLI, ...args], {
    cwd: ROOT,
    encoding: 'utf8',
  });
}

describe('refactor plan verifier', () => {
  const tmp = mkdtempSync(path.join(tmpdir(), 'fx-vitest-refactor-plan-verify-'));

  afterAll(() => {
    rmSync(tmp, { recursive: true, force: true });
  });

  it('accepts a valid SHORT code-change plan with pre-write handoff', () => {
    const result = verifyPlan(sampleShort(), {
      expectCodeChange: true,
      mode: 'short',
    });

    expect(result.ok).toBe(true);
    expect(result.errors).toEqual([]);
  });

  it('rejects code-changing SHORT plans without pre-write handoff', () => {
    const result = verifyPlan(
      sampleShort()
        .replace(/^Pre-write handoff:.*\n/m, '')
        .replace('Start with pre-write', 'Start with a narrow implementation check'),
      {
        expectCodeChange: true,
        mode: 'short',
      },
    );

    expect(result.ok).toBe(false);
    expect(result.errors).toEqual(
      expect.arrayContaining([expect.objectContaining({ code: 'missing-prewrite-handoff' })]),
    );
  });

  it('rejects code-changing SHORT plans without coding-agent prompt', () => {
    const result = verifyPlan(sampleShort().replace(/^Ask the coding agent:.*\n/m, ''), {
      expectCodeChange: true,
      mode: 'short',
    });

    expect(result.ok).toBe(false);
    expect(result.errors).toEqual(
      expect.arrayContaining([expect.objectContaining({ code: 'missing-coding-agent-prompt' })]),
    );
  });

  it('rejects raw JSON blocks in default chat-facing plans', () => {
    const result = verifyPlan(sampleShort('\n```json\n{"files":["src/a.ts"]}\n```\n'), {
      expectCodeChange: true,
      mode: 'short',
    });

    expect(result.ok).toBe(false);
    expect(result.errors).toEqual(
      expect.arrayContaining([expect.objectContaining({ code: 'raw-json-in-chat' })]),
    );
  });

  it('rejects discouraging tone', () => {
    const result = verifyPlan(sampleShort('\nThis code is broken and terrible.\n'), {
      expectCodeChange: true,
      mode: 'short',
    });

    expect(result.ok).toBe(false);
    expect(result.errors).toEqual(
      expect.arrayContaining([expect.objectContaining({ code: 'discouraging-tone' })]),
    );
  });

  it('rejects plans without artifact or claim-label evidence anchors', () => {
    const result = verifyPlan(sampleShort().replace('(topology.json, manifest.json)', '(the audit)'), {
      expectCodeChange: true,
      mode: 'short',
    });

    expect(result.ok).toBe(false);
    expect(result.errors).toEqual(
      expect.arrayContaining([expect.objectContaining({ code: 'missing-evidence-anchor' })]),
    );
  });

  it('accepts a FULL handoff plan with required sections', () => {
    const result = verifyPlan(sampleFull(), { mode: 'full' });

    expect(result.ok).toBe(true);
    expect(result.errors).toEqual([]);
  });

  it('CLI exits 0 for a valid plan', () => {
    const goodFile = path.join(tmp, 'good.md');
    writeFileSync(goodFile, sampleShort());

    const result = runCli(['--mode', 'short', '--expect-code-change', goodFile]);

    expect(result.status).toBe(0);
    expect(result.stdout).toContain('[verify-refactor-plan] OK');
  });

  it('CLI exits non-zero and names missing verification section', () => {
    const badFile = path.join(tmp, 'bad.md');
    writeFileSync(badFile, sampleShort().replace('**How We Verify**', '**How We Guess**'));

    const result = runCli(['--mode', 'short', '--expect-code-change', badFile]);

    expect(result.status).toBe(1);
    expect(result.stderr).toContain('verification-section');
  });
});
