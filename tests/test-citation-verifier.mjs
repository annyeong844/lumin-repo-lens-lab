// Rule 1 citation verifier: grounded labels must be falsifiable against
// audit JSON artifacts.

import { execFileSync, spawnSync } from 'node:child_process';
import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { verifyGroundedCitations } from '../test-harness/lib/verify-citations.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');
const CLI = path.join(ROOT, 'test-harness/lib/verify-citations.mjs');
const NODE = process.execPath;
const TMP = mkdtempSync(path.join(tmpdir(), 'fx-citation-verify-'));
const OUT = path.join(TMP, 'audit');
const REPO = path.join(TMP, 'repo');

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

try {
  mkdirSync(OUT, { recursive: true });
  mkdirSync(REPO, { recursive: true });
  writeFileSync(path.join(OUT, 'topology.json'), JSON.stringify({
    summary: { sccCount: 0 },
    nodes: {
      'src/a.ts': { loc: 12 },
    },
    largestFiles: [
      { file: 'src/a.ts', loc: 12 },
      { file: 'src/b.ts', loc: 8 },
    ],
  }, null, 2));
  writeFileSync(path.join(OUT, 'checklist-facts.json'), JSON.stringify({
    A2_function_size: {
      buckets: { big: 4, medium: 4, small: 491 },
    },
  }, null, 2));
  writeFileSync(path.join(OUT, 'symbols.json'), JSON.stringify({
    fanInByIdentity: {
      'src/utils/date.ts::formatDate': 8,
    },
    deadProdList: [{ name: 'a' }, { name: 'b' }, { name: 'c' }],
  }, null, 2));
  writeFileSync(path.join(REPO, 'package.json'), JSON.stringify({
    dependencies: { dayjs: '1.0.0' },
  }, null, 2));

  {
    const text = [
      '- Cycles are clear [grounded, topology.json.summary.sccCount = 0]',
      "- Fan-in is known [grounded, symbols.json.fanInByIdentity['src/utils/date.ts::formatDate'] = 8]",
      '- Length works [grounded, symbols.json.deadProdList.length = 3]',
      '- Object literals work [grounded, checklist-facts.json.A2_function_size.buckets = {big: 4, medium: 4, small: 491}]',
      "- Root package fallback works [grounded, package.json.dependencies['dayjs'] = '1.0.0']",
    ].join('\n');
    const result = verifyGroundedCitations(text, { artifactsDir: OUT, rootDir: REPO });
    assert('C1. valid scalar, bracket, length, object, and root package citations pass',
      result.ok && result.checked === 5 && result.citationsFound === 5,
      JSON.stringify(result));
  }

  {
    const result = verifyGroundedCitations(
      'Wrong value [grounded, topology.json.summary.sccCount = 1]',
      { artifactsDir: OUT },
    );
    assert('C2. value mismatch fails',
      !result.ok && result.failures.some((e) => e.code === 'value-mismatch'),
      JSON.stringify(result));
  }

  {
    const result = verifyGroundedCitations(
      'Unfalsifiable [grounded, source: topology.json]',
      { artifactsDir: OUT },
    );
    assert('C3. grounded citation without path=value fails',
      !result.ok && result.failures.some((e) => e.code === 'unfalsifiable-grounded-citation'),
      JSON.stringify(result));
  }

  {
    const result = verifyGroundedCitations(
      'Missing path [grounded, topology.json.summary.nope = 0]',
      { artifactsDir: OUT },
    );
    assert('C4. missing artifact path fails',
      !result.ok && result.failures.some((e) => e.code === 'artifact-path-missing'),
      JSON.stringify(result));
  }

  {
    const result = verifyGroundedCitations(
      'Placeholder [grounded, topology.json.summary.sccCount = N]',
      { artifactsDir: OUT },
    );
    assert('C5. placeholder values fail',
      !result.ok && result.failures.some((e) => e.code === 'expected-value-uncheckable'),
      JSON.stringify(result));
  }

  {
    const result = verifyGroundedCitations(
      'Extra clause [grounded, topology.json.summary.sccCount = 0, lens = runtime]',
      { artifactsDir: OUT },
    );
    assert('C6. first assignment passes while trailing clause is warned',
      result.ok &&
      result.checked === 1 &&
      result.warnings.some((e) => e.code === 'trailing-unverified-clause'),
      JSON.stringify(result));
  }

  {
    const good = path.join(TMP, 'good.md');
    writeFileSync(good, 'OK [grounded, topology.json.summary.sccCount = 0]\n');
    const stdout = execFileSync(NODE, [CLI, '--artifacts', OUT, good], {
      cwd: ROOT,
      encoding: 'utf8',
    });
    assert('C7. CLI exits 0 for valid citation',
      stdout.includes('[verify-citations] OK'),
      stdout);
  }

  {
    const bad = path.join(TMP, 'bad.md');
    writeFileSync(bad, 'Bad [grounded, topology.json.summary.sccCount = 9]\n');
    const result = spawnSync(NODE, [CLI, '--artifacts', OUT, bad], {
      cwd: ROOT,
      encoding: 'utf8',
    });
    assert('C8. CLI exits 1 for mismatched citation',
      result.status === 1 && result.stderr.includes('value-mismatch'),
      `${result.status}\n${result.stdout}\n${result.stderr}`);
  }

  {
    const result = spawnSync(NODE, [CLI, '--artifacts', OUT, '-'], {
      cwd: ROOT,
      input: 'STDIN [grounded, topology.json.largestFiles[0].loc = 12]\n',
      encoding: 'utf8',
    });
    assert('C9. CLI reads Markdown from stdin',
      result.status === 0 && result.stdout.includes('checked 1/1'),
      `${result.status}\n${result.stdout}\n${result.stderr}`);
  }
} finally {
  rmSync(TMP, { recursive: true, force: true });
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
