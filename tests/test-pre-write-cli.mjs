// CLI smoke test for pre-write.mjs — P1-1 step 5.6.
//
// Nine assertions from docs/history/phases/p1/p1-1.md §5.6:
//   1. Exit code 0 on happy path.
//   2. stdout contains the "pre-write advisory" header.
//   3. stdout contains an EXISTS row for the intent name.
//   4. pre-write-advisory.latest.json exists.
//   5. pre-write-advisory.<invocationId>.json exists with same content.
//   6. intentHash is a 64-char sha256 hex string.
//   7. Path with spaces works.
//   8. symbols.json absent → exit 0 with failures + [확인 불가].
//   9. Missing --intent or malformed intent → non-zero exit, error on stderr.

import { execSync, execFileSync, spawnSync } from 'node:child_process';
import { writeFileSync, readFileSync, mkdirSync, mkdtempSync, rmSync, existsSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');
const NODE = process.execPath;
const PREWRITE = path.join(DIR, 'pre-write.mjs');

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

function write(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
}

function buildFixture(fxDir) {
  write(fxDir, 'package.json', JSON.stringify({ name: 'pw-fx', type: 'module' }));
  write(fxDir, 'src/a.ts', 'export const formatDate = (d) => d.toString();\n');
  write(fxDir, 'src/b.ts', "import { formatDate } from './a';\nexport const useFmt = () => formatDate(new Date());\n");
}

function runBuildSymbols(fxDir, outDir) {
  execFileSync(NODE, [path.join(DIR, 'build-symbol-graph.mjs'), '--root', fxDir, '--output', outDir], {
    stdio: ['ignore', 'pipe', 'pipe'],
  });
}

function runPreWrite(fxDir, outDir, intentPath) {
  return execFileSync(NODE, [PREWRITE, '--root', fxDir, '--output', outDir, '--intent', intentPath], {
    stdio: ['ignore', 'pipe', 'pipe'],
  }).toString('utf8');
}

// ═══ Happy path — assertions 1-6 ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'pw-cli-happy-'));
  const out = mkdtempSync(path.join(tmpdir(), 'pw-cli-happy-out-'));
  try {
    buildFixture(fx);
    runBuildSymbols(fx, out);

    const intent = {
      names: ['formatDate'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [],
    };
    const intentPath = path.join(out, 'intent.json');
    writeFileSync(intentPath, JSON.stringify(intent));

    const stdout = runPreWrite(fx, out, intentPath);

    assert('T1. exit code 0 (implicit — execFileSync would have thrown)', true);
    assert('T2. stdout contains "pre-write advisory" header',
      stdout.includes('pre-write advisory'));
    assert('T3. stdout contains grounded cue for formatDate',
      stdout.includes('### Grounded facts') && stdout.includes('formatDate'));

    const latest = path.join(out, 'pre-write-advisory.latest.json');
    assert('T4. pre-write-advisory.latest.json exists',
      existsSync(latest));

    const invFiles = readdirSync(out).filter((n) =>
      n.startsWith('pre-write-advisory.') && !n.endsWith('.latest.json'));
    assert('T5a. at least one invocation-specific advisory file exists',
      invFiles.length === 1);
    const specific = path.join(out, invFiles[0]);
    const latestContent = readFileSync(latest, 'utf8');
    const specificContent = readFileSync(specific, 'utf8');
    assert('T5b. latest and invocation-specific contents match',
      latestContent === specificContent);

    const parsed = JSON.parse(latestContent);
    assert('T6. intentHash is 64-char sha256 hex',
      typeof parsed.intentHash === 'string' && /^[a-f0-9]{64}$/.test(parsed.intentHash),
      `intentHash=${parsed.intentHash}`);
    assert('T6b. capabilities copied from symbols.meta.supports',
      parsed.capabilities?.identityFanIn === true);
    assert('T6c. advisory JSON exposes invocation-specific post-write handoff path',
      path.basename(parsed.artifactPaths?.invocationSpecific ?? '') === invFiles[0] &&
      path.basename(parsed.artifactPaths?.latest ?? '') === 'pre-write-advisory.latest.json',
      JSON.stringify(parsed.artifactPaths));
    assert('T6d. stdout prints invocation-specific --pre-write-advisory handoff',
      stdout.includes('--pre-write-advisory') && stdout.includes(invFiles[0]) &&
      !stdout.includes('--pre-write-advisory pre-write-advisory.latest.json'),
      stdout);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ Path-with-spaces — assertion 7 ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'pw-cli-rich-intent-'));
  const out = mkdtempSync(path.join(tmpdir(), 'pw-cli-rich-intent-out-'));
  try {
    buildFixture(fx);
    runBuildSymbols(fx, out);
    const intent = {
      names: [
        { name: 'formatTimestamp', kind: 'function', why: 'new display helper' },
      ],
      shapes: [
        {
          name: 'TimestampViewModel',
          typeLiteral: '{ label: string; iso: string; timezone: string }',
          why: 'view model contract',
        },
      ],
      files: ['src/features/time/format-timestamp.ts'],
      dependencies: [
        { specifier: 'date-fns', why: 'timestamp formatting' },
      ],
      plannedTypeEscapes: [],
    };
    const intentPath = path.join(out, 'intent-rich.json');
    writeFileSync(intentPath, JSON.stringify(intent));
    runPreWrite(fx, out, intentPath);
    const parsed = JSON.parse(readFileSync(path.join(out, 'pre-write-advisory.latest.json'), 'utf8'));
    assert('T6c. rich intent object entries pass CLI validation and normalize lookup arrays',
      parsed.intent.names.includes('formatTimestamp') &&
      parsed.intent.dependencies.includes('date-fns') &&
      parsed.intent.shapes[0].typeLiteral.includes('timezone') &&
      parsed.intent.shapes[0].fields.length === 0,
      JSON.stringify(parsed.intent));
    assert('T6d. rich intent preserves self-declaration why metadata in advisory JSON',
      parsed.intent.nameDeclarations?.[0]?.why === 'new display helper' &&
      parsed.intent.dependencyDeclarations?.[0]?.why === 'timestamp formatting' &&
      parsed.intent.shapes[0].why === 'view model contract',
      JSON.stringify(parsed.intent));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

{
  const parent = mkdtempSync(path.join(tmpdir(), 'pw-cli-space-'));
  // Put fixture + output inside a subdirectory whose name contains a
  // space, so every path argument has to survive shell quoting.
  const fx = path.join(parent, 'my fixture');
  const out = path.join(parent, 'my output');
  mkdirSync(fx, { recursive: true });
  mkdirSync(out, { recursive: true });
  try {
    buildFixture(fx);
    runBuildSymbols(fx, out);
    const intent = { names: ['formatDate'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] };
    const intentPath = path.join(out, 'intent.json');
    writeFileSync(intentPath, JSON.stringify(intent));
    const stdout = runPreWrite(fx, out, intentPath);
    assert('T7. path-with-spaces fixture works end-to-end',
      stdout.includes('### Grounded facts') && stdout.includes('formatDate'));
    assert('T7b. latest.json written under space-path',
      existsSync(path.join(out, 'pre-write-advisory.latest.json')));
  } finally {
    rmSync(parent, { recursive: true, force: true });
  }
}

// ═══ symbols.json absent + --no-fresh-audit — assertion 8 ═══
//
// P1-3 cold-cache auto-runs producers when artifacts are missing by
// default. `--no-fresh-audit` opts out: cold-cache is skipped and the
// advisory degrades to [확인 불가] — which is the behavior T8 originally
// pinned (P1-1 was warm-cache-only).

{
  const fx = mkdtempSync(path.join(tmpdir(), 'pw-cli-nosym-'));
  const out = mkdtempSync(path.join(tmpdir(), 'pw-cli-nosym-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'nosym' }));
    const intent = { names: ['anything'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] };
    const intentPath = path.join(out, 'intent.json');
    writeFileSync(intentPath, JSON.stringify(intent));

    // Pass --no-fresh-audit explicitly; cold-cache will NOT spawn.
    const stdout = execFileSync(NODE, [PREWRITE,
      '--root', fx, '--output', out, '--intent', intentPath, '--no-fresh-audit',
    ], { stdio: ['ignore', 'pipe', 'pipe'] }).toString('utf8');

    assert('T8. exit 0 when symbols.json absent + --no-fresh-audit',
      true);  // execFileSync would have thrown on nonzero exit
    assert('T8b. Markdown includes [확인 불가] when symbols absent + --no-fresh-audit',
      stdout.includes('확인 불가'),
      `stdout excerpt: ${stdout.slice(0, 400)}`);

    // The advisory JSON should carry a symbols-missing failure entry.
    const latest = path.join(out, 'pre-write-advisory.latest.json');
    const parsed = JSON.parse(readFileSync(latest, 'utf8'));
    assert('T8c. advisory JSON lists symbols-missing failure under --no-fresh-audit',
      Array.isArray(parsed.failures) &&
      parsed.failures.some((f) => f.kind === 'symbols-missing'),
      `failures=${JSON.stringify(parsed.failures)}`);
    assert('T8c2. advisory JSON marks required evidence unavailable',
      parsed.evidenceAvailability?.status === 'missing' &&
      parsed.evidenceAvailability?.freshAudit === false &&
      parsed.evidenceAvailability?.artifacts?.some((entry) =>
        entry.artifact === 'symbols.json' &&
        entry.status === 'missing' &&
        entry.requiredFor.includes('names')),
      JSON.stringify(parsed.evidenceAvailability, null, 2));
    assert('T8c3. Markdown tells user to reuse same output or allow cold-cache',
      stdout.includes('Evidence availability') &&
      stdout.includes('symbols.json') &&
      stdout.includes('same `--output`') &&
      stdout.includes('not grounded absence'),
      stdout);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ dependency intent + symbols.json absent must not claim grounded zero ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'pw-cli-nodepsym-'));
  const out = mkdtempSync(path.join(tmpdir(), 'pw-cli-nodepsym-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({
      name: 'nodepsym',
      dependencies: { dayjs: '1.0.0' },
    }));
    const intent = { names: [], shapes: [], files: [], dependencies: ['dayjs'], plannedTypeEscapes: [] };
    const intentPath = path.join(out, 'intent.json');
    writeFileSync(intentPath, JSON.stringify(intent));

    const stdout = execFileSync(NODE, [PREWRITE,
      '--root', fx, '--output', out, '--intent', intentPath, '--no-fresh-audit',
    ], { stdio: ['ignore', 'pipe', 'pipe'] }).toString('utf8');

    assert('T8f. dependency lookup without symbols reports import graph unavailable',
      stdout.includes('DEPENDENCY_AVAILABLE_IMPORT_GRAPH_UNAVAILABLE') &&
      stdout.includes('import graph unavailable'),
      `stdout excerpt: ${stdout.slice(0, 500)}`);
    assert('T8g. dependency lookup without symbols does not render grounded zero consumers',
      !stdout.includes('0 observed consumer'),
      `stdout excerpt: ${stdout.slice(0, 500)}`);

    const parsed = JSON.parse(readFileSync(path.join(out, 'pre-write-advisory.latest.json'), 'utf8'));
    const depLookup = parsed.lookups.find((l) => l.kind === 'dependency');
    assert('T8h. advisory JSON marks dependency countConfidence unavailable',
      depLookup?.existingImports?.observedImportCount === null &&
      depLookup?.existingImports?.countConfidence === 'unavailable',
      JSON.stringify(depLookup));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ dependency intent + fresh symbols reports observed static package imports ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'pw-cli-dep-consumer-'));
  const out = mkdtempSync(path.join(tmpdir(), 'pw-cli-dep-consumer-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({
      name: 'dep-consumer',
      type: 'module',
      dependencies: { dayjs: '1.0.0' },
    }));
    write(fx, 'src/use.ts', "import dayjs from 'dayjs';\nexport const today = () => dayjs().format('YYYY-MM-DD');\n");
    const intent = { names: [], shapes: [], files: [], dependencies: ['dayjs'], plannedTypeEscapes: [] };
    const intentPath = path.join(out, 'intent.json');
    writeFileSync(intentPath, JSON.stringify(intent));

    const stdout = execFileSync(NODE, [
      PREWRITE, '--root', fx, '--output', out, '--intent', intentPath,
    ], { stdio: ['ignore', 'pipe', 'pipe'] }).toString('utf8');

    assert('T8i. fresh dependency lookup reports DEPENDENCY_AVAILABLE',
      stdout.includes('DEPENDENCY_AVAILABLE') &&
      !stdout.includes('DEPENDENCY_AVAILABLE_IMPORT_GRAPH_UNAVAILABLE'),
      `stdout excerpt: ${stdout.slice(0, 700)}`);

    const symbols = JSON.parse(readFileSync(path.join(out, 'symbols.json'), 'utf8'));
    assert('T8j. symbols.json advertises dependencyImportConsumers capability',
      symbols.meta?.supports?.dependencyImportConsumers === true &&
      Array.isArray(symbols.dependencyImportConsumers),
      JSON.stringify(symbols.meta?.supports));
    assert('T8k. symbols.json records dayjs static package consumer',
      symbols.dependencyImportConsumers.some((c) =>
        c.file === 'src/use.ts' && c.fromSpec === 'dayjs' && c.depRoot === 'dayjs'),
      JSON.stringify(symbols.dependencyImportConsumers));

    const parsed = JSON.parse(readFileSync(path.join(out, 'pre-write-advisory.latest.json'), 'utf8'));
    const depLookup = parsed.lookups.find((l) => l.kind === 'dependency');
    assert('T8l. advisory JSON grounds observed dependency consumer count',
      depLookup?.result === 'DEPENDENCY_AVAILABLE' &&
      depLookup?.existingImports?.observedImportCount === 1 &&
      depLookup?.existingImports?.countConfidence === 'grounded',
      JSON.stringify(depLookup));
    assert('T8m. dependency citation names dependencyImportConsumers',
      depLookup?.citations?.some((c) => /symbols\.json\.dependencyImportConsumers/.test(c)),
      JSON.stringify(depLookup?.citations));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ compact intent defaults missing top-level arrays without noisy markdown ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'pw-cli-compact-intent-'));
  const out = mkdtempSync(path.join(tmpdir(), 'pw-cli-compact-intent-out-'));
  try {
    buildFixture(fx);
    const intent = { files: ['src/new-helper.ts'] };
    const intentPath = path.join(out, 'intent.json');
    writeFileSync(intentPath, JSON.stringify(intent));

    const stdout = execFileSync(NODE, [
      PREWRITE, '--root', fx, '--output', out, '--intent', intentPath,
    ], { stdio: ['ignore', 'pipe', 'pipe'] }).toString('utf8');

    assert('T8n. compact intent omits benign schema default notes',
      !stdout.includes('Intent schema notes') &&
      !stdout.includes('Missing top-level intent keys defaulted'),
      `stdout excerpt: ${stdout.slice(0, 700)}`);

    const parsed = JSON.parse(readFileSync(path.join(out, 'pre-write-advisory.latest.json'), 'utf8'));
    assert('T8o. compact intent JSON normalizes all five top-level arrays',
      Array.isArray(parsed.intent.names) &&
      Array.isArray(parsed.intent.shapes) &&
      Array.isArray(parsed.intent.files) &&
      Array.isArray(parsed.intent.dependencies) &&
      Array.isArray(parsed.intent.plannedTypeEscapes) &&
      parsed.intent.files.includes('src/new-helper.ts'),
      JSON.stringify(parsed.intent));
    assert('T8p. compact intent JSON preserves missing-key warnings',
      parsed.intentWarnings?.length === 4 &&
      parsed.intentWarnings.every((w) => w.kind === 'missing-intent-key-defaulted'),
      JSON.stringify(parsed.intentWarnings));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ cold-cache scan scope propagation ═══
{
  const fx = mkdtempSync(path.join(tmpdir(), 'pw-cli-cold-scope-'));
  const out = mkdtempSync(path.join(tmpdir(), 'pw-cli-cold-scope-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'cold-scope', type: 'module' }));
    write(fx, 'src/prod.ts', 'export const prodOnly = 1;\n');
    write(fx, 'src/prod.test.ts', 'export const testOnly = 1;\n');
    const intent = {
      names: ['prodOnly'],
      shapes: [],
      files: [],
      dependencies: [],
      plannedTypeEscapes: [],
    };
    const intentPath = path.join(out, 'intent.json');
    writeFileSync(intentPath, JSON.stringify(intent));

    execFileSync(NODE, [
      PREWRITE,
      '--root', fx,
      '--output', out,
      '--intent', intentPath,
      '--production',
    ], { stdio: ['ignore', 'pipe', 'pipe'] });

    const symbols = JSON.parse(readFileSync(path.join(out, 'symbols.json'), 'utf8'));
    const defFiles = Object.keys(symbols.defIndex ?? {});

    assert('T8d. cold-cache preflight forwards --production to build-symbol-graph',
      !defFiles.some((f) => f.includes('.test.')),
      JSON.stringify(defFiles));
    assert('T8e. names-only cold-cache does not run triage-repo',
      !existsSync(path.join(out, 'triage.json')));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ Missing --intent / malformed intent — assertion 9 ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'pw-cli-bad-'));
  const out = mkdtempSync(path.join(tmpdir(), 'pw-cli-bad-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'bad' }));

    // Case A: missing --intent flag.
    let caughtMissing = false;
    try {
      execFileSync(NODE, [PREWRITE, '--root', fx, '--output', out], {
        stdio: ['ignore', 'pipe', 'pipe'],
      });
    } catch (e) {
      caughtMissing = true;
      const err = (e.stderr?.toString?.() ?? '') + (e.stdout?.toString?.() ?? '');
      assert('T9a. missing --intent produces a helpful stderr message',
        /intent/i.test(err), `err=${err.slice(0, 200)}`);
    }
    assert('T9b. missing --intent exits non-zero',
      caughtMissing === true);

    // Case B: malformed intent JSON.
    const badIntentPath = path.join(out, 'bad-intent.json');
    writeFileSync(badIntentPath, '{ this is not valid json');
    let caughtMalformed = false;
    try {
      execFileSync(NODE, [PREWRITE, '--root', fx, '--output', out, '--intent', badIntentPath], {
        stdio: ['ignore', 'pipe', 'pipe'],
      });
    } catch (e) {
      caughtMalformed = true;
      const err = (e.stderr?.toString?.() ?? '');
      assert('T9c. malformed intent JSON exits non-zero with parse-error message',
        /parse/i.test(err) || /JSON/i.test(err),
        `err=${err.slice(0, 200)}`);
    }
    assert('T9d. malformed intent JSON exits non-zero',
      caughtMalformed === true);

    // Case C: schema-valid JSON, schema-invalid intent.
    const schemaBadPath = path.join(out, 'schema-bad.json');
    writeFileSync(schemaBadPath, JSON.stringify({
      names: 'formatDate',  // string instead of array
      shapes: [], files: [], dependencies: [], plannedTypeEscapes: [],
    }));
    let caughtSchema = false;
    try {
      execFileSync(NODE, [PREWRITE, '--root', fx, '--output', out, '--intent', schemaBadPath], {
        stdio: ['ignore', 'pipe', 'pipe'],
      });
    } catch (e) {
      caughtSchema = true;
      const err = (e.stderr?.toString?.() ?? '');
      assert('T9e. schema-invalid intent exits non-zero with errorPath citation',
        err.includes('names') || err.includes('errorPath'),
        `err=${err.slice(0, 200)}`);
    }
    assert('T9f. schema-invalid intent exits non-zero',
      caughtSchema === true);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}


// ═══ P1-2 — file / dependency / shape lookups end-to-end ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'pw-cli-p12-'));
  const out = mkdtempSync(path.join(tmpdir(), 'pw-cli-p12-out-'));
  try {
    buildFixture(fx);
    // Also need a topology artifact — run measure-topology.
    execFileSync(NODE, [path.join(DIR, 'build-symbol-graph.mjs'), '--root', fx, '--output', out], { stdio: ['ignore', 'pipe', 'pipe'] });
    execFileSync(NODE, [path.join(DIR, 'measure-topology.mjs'), '--root', fx, '--output', out], { stdio: ['ignore', 'pipe', 'pipe'] });

    // Intent exercising all four paths: 1 name (EXISTS), 1 new file,
    // 1 new dependency, 1 shape.
    const intent = {
      names: ['formatDate'],
      shapes: [{ fields: ['year', 'month'] }],
      files: ['src/utils/time.ts'],
      dependencies: ['dayjs'],
      plannedTypeEscapes: [],
    };
    const intentPath = path.join(out, 'intent.json');
    writeFileSync(intentPath, JSON.stringify(intent));
    const stdout = runPreWrite(fx, out, intentPath);

    // Each of four sections present.
    assert('P12.T1. Grounded facts section present (formatDate)',
      stdout.includes('### Grounded facts') && stdout.includes('formatDate'));
    assert('P12.T2. New code candidates section present (NEW_FILE + NEW_PACKAGE)',
      stdout.includes('### New code candidates'));
    assert('P12.T3. Unavailable evidence section present (shape UNAVAILABLE)',
      stdout.includes('### Unavailable evidence'));

    // Specific intent items rendered.
    assert('P12.T4. NEW_FILE row for src/utils/time.ts',
      stdout.includes('NEW_FILE') && stdout.includes('src/utils/time.ts'));
    assert('P12.T5. NEW_PACKAGE row for dayjs',
      stdout.includes('NEW_PACKAGE') && stdout.includes('dayjs'));
    assert('P12.T6. shape-hash + P4 citation present',
      stdout.includes('shape-hash') && stdout.includes('P4'));

    // Artifact JSON contains the new lookup kinds.
    const latest = JSON.parse(readFileSync(path.join(out, 'pre-write-advisory.latest.json'), 'utf8'));
    const kinds = new Set(latest.lookups.map((l) => l.kind));
    assert('P12.T7. artifact lookups[] contains all 4 kinds (name/file/dependency/shape)',
      kinds.has('name') && kinds.has('file') && kinds.has('dependency') && kinds.has('shape'),
      `kinds=${[...kinds].join(',')}`);

    // Name-first ordering preserved.
    assert('P12.T8. lookups[] ordering: name first',
      latest.lookups[0].kind === 'name');
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ P2-0 — preWrite.anyInventoryPath on both advisory files ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'pw-p20-'));
  const out = mkdtempSync(path.join(tmpdir(), 'pw-p20-out-'));
  try {
    buildFixture(fx);
    const intent = { names: ['formatDate'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] };
    const intentPath = path.join(out, 'intent.json');
    writeFileSync(intentPath, JSON.stringify(intent));

    runPreWrite(fx, out, intentPath);
    const latest = JSON.parse(readFileSync(path.join(out, 'pre-write-advisory.latest.json'), 'utf8'));
    const invFiles = readdirSync(out).filter((n) =>
      n.startsWith('pre-write-advisory.') && !n.endsWith('.latest.json'));
    const inv = JSON.parse(readFileSync(path.join(out, invFiles[0]), 'utf8'));

    assert('P20.T1. latest.json carries preWrite.anyInventoryPath',
      !!latest.preWrite?.anyInventoryPath);
    assert('P20.T2. invocation.json carries preWrite.anyInventoryPath',
      !!inv.preWrite?.anyInventoryPath);
    assert('P20.T3. both pointers are identical',
      latest.preWrite?.anyInventoryPath === inv.preWrite?.anyInventoryPath);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ P1-2 — triage absent does NOT crash the CLI ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'pw-cli-p12-notriage-'));
  const out = mkdtempSync(path.join(tmpdir(), 'pw-cli-p12-notriage-out-'));
  try {
    buildFixture(fx);
    execFileSync(NODE, [path.join(DIR, 'build-symbol-graph.mjs'), '--root', fx, '--output', out], { stdio: ['ignore', 'pipe', 'pipe'] });
    // Intentionally skip triage; only topology + symbols present.
    execFileSync(NODE, [path.join(DIR, 'measure-topology.mjs'), '--root', fx, '--output', out], { stdio: ['ignore', 'pipe', 'pipe'] });

    const intent = { names: [], shapes: [], files: ['src/utils/time.ts'], dependencies: [], plannedTypeEscapes: [] };
    const intentPath = path.join(out, 'intent.json');
    writeFileSync(intentPath, JSON.stringify(intent));
    const stdout = runPreWrite(fx, out, intentPath);
    assert('P12.T9. triage.json absent → CLI exits 0 and renders NEW_FILE',
      stdout.includes('NEW_FILE') || stdout.includes('FILE_STATUS_UNKNOWN'));
    // Boundary sub-line shows not-evaluated, never ALLOWED/FORBIDDEN.
    assert('P12.T10. boundary "not evaluated" appears; never ALLOWED/FORBIDDEN',
      /not.evaluated/i.test(stdout) && !/boundary.*ALLOWED/i.test(stdout) && !/boundary.*FORBIDDEN/i.test(stdout));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ P1-3 — Cold-cache auto-run ═══

// Case 1: names-only cold-cache happy path.
{
  const fx = mkdtempSync(path.join(tmpdir(), 'pw-coldcache-full-'));
  const out = mkdtempSync(path.join(tmpdir(), 'pw-coldcache-full-out-'));
  try {
    buildFixture(fx);
    const intent = { names: ['formatDate'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] };
    const intentPath = path.join(out, 'intent.json');
    writeFileSync(intentPath, JSON.stringify(intent));

    // Out dir is empty (only intent.json). Default (no --no-fresh-audit)
    // should trigger only the artifact needed for a names lookup.
    const stdout = runPreWrite(fx, out, intentPath);
    assert('P13.CC1. cold-cache happy path exits 0 and renders grounded cue',
      stdout.includes('### Grounded facts') && stdout.includes('formatDate'));
    assert('P13.CC1b. symbols.json produced by cold-cache',
      existsSync(path.join(out, 'symbols.json')));
    assert('P13.CC1c. topology.json not produced for names-only cold-cache',
      !existsSync(path.join(out, 'topology.json')));
    assert('P13.CC1d. triage.json not produced for names-only cold-cache',
      !existsSync(path.join(out, 'triage.json')));
    assert('P13.CC1d2. shape-index.json not produced when intent has no shapes',
      !existsSync(path.join(out, 'shape-index.json')));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// Case 1b: shape-index cold-caches only when exact shape evidence is requested.
{
  const fx = mkdtempSync(path.join(tmpdir(), 'pw-coldcache-shape-'));
  const out = mkdtempSync(path.join(tmpdir(), 'pw-coldcache-shape-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'pw-shape-fx', type: 'module' }));
    write(fx, 'src/types.ts', `export type CalendarShape = { year: number };\n`);
    const intent = {
      names: [],
      shapes: [{ fields: [], typeLiteral: '{ year: number }' }],
      files: [],
      dependencies: [],
      plannedTypeEscapes: [],
    };
    const intentPath = path.join(out, 'intent.json');
    writeFileSync(intentPath, JSON.stringify(intent));

    const stdout = runPreWrite(fx, out, intentPath);
    assert('P13.CC1e. exact shape intent cold-caches shape-index.json',
      existsSync(path.join(out, 'shape-index.json')));
    assert('P13.CC1e2. exact shape intent does not cold-cache unrelated graph artifacts',
      !existsSync(path.join(out, 'symbols.json')) &&
      !existsSync(path.join(out, 'topology.json')) &&
      !existsSync(path.join(out, 'triage.json')));
    assert('P13.CC1f. cold-cached shape-index enables grounded shape cue',
      stdout.includes('same normalized type shape') && stdout.includes('src/types.ts::CalendarShape'),
      stdout);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// Case 1c: fields-only shape intents still cold-cache the producer, but
// the lookup stays honest because field names alone are not equality.
{
  const fx = mkdtempSync(path.join(tmpdir(), 'pw-coldcache-shape-fields-'));
  const out = mkdtempSync(path.join(tmpdir(), 'pw-coldcache-shape-fields-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'pw-shape-fields-fx', type: 'module' }));
    write(fx, 'src/types.ts', `export type LoggerShape = { info: string; warn: string; error: string; withContext: string };\n`);
    const intent = {
      names: [],
      shapes: [{ fields: ['info', 'warn', 'error', 'withContext'] }],
      files: [],
      dependencies: [],
      plannedTypeEscapes: [],
    };
    const intentPath = path.join(out, 'intent.json');
    writeFileSync(intentPath, JSON.stringify(intent));

    const stdout = runPreWrite(fx, out, intentPath);
    assert('P13.CC1g. fields-only shape intent also cold-caches shape-index.json',
      existsSync(path.join(out, 'shape-index.json')));
    assert('P13.CC1g2. fields-only shape intent does not cold-cache unrelated graph artifacts',
      !existsSync(path.join(out, 'symbols.json')) &&
      !existsSync(path.join(out, 'topology.json')) &&
      !existsSync(path.join(out, 'triage.json')));
    assert('P13.CC1h. fields-only shape stays UNAVAILABLE with exactness citation',
      stdout.includes('UNAVAILABLE') && stdout.includes('field names alone are not structural equality evidence'),
      stdout);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// Case 1d: function type-literal intents cold-cache function-clones and
// can surface same-signature helpers even when the planned name differs.
{
  const fx = mkdtempSync(path.join(tmpdir(), 'pw-coldcache-function-signature-'));
  const out = mkdtempSync(path.join(tmpdir(), 'pw-coldcache-function-signature-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'pw-function-signature-fx', type: 'module' }));
    write(fx, 'src/shallow.ts',
      `export function useShallow<S, U>(selector: (state: S) => U): (state: S) => U {\n` +
      `  return selector;\n` +
      `}\n`);
    const intent = {
      names: ['composeProjection'],
      shapes: [{
        fields: [],
        typeLiteral: '<S, U>(selector: (state: S) => U) => (state: S) => U',
      }],
      files: [],
      dependencies: [],
      plannedTypeEscapes: [],
    };
    const intentPath = path.join(out, 'intent.json');
    writeFileSync(intentPath, JSON.stringify(intent));

    const stdout = runPreWrite(fx, out, intentPath);
    assert('P13.CC1i. function signature intent cold-caches function-clones.json',
      existsSync(path.join(out, 'function-clones.json')));
    assert('P13.CC1j. function signature intent renders grounded signature cue',
      stdout.includes('same normalized function signature') &&
      stdout.includes('src/shallow.ts::useShallow') &&
      stdout.includes('function signature'),
      stdout);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// Case 2: partial cold-cache — only topology missing.
{
  const fx = mkdtempSync(path.join(tmpdir(), 'pw-coldcache-partial-'));
  const out = mkdtempSync(path.join(tmpdir(), 'pw-coldcache-partial-out-'));
  try {
    buildFixture(fx);
    // Pre-populate symbols.json and triage.json so ONLY topology cold-caches.
    execFileSync(NODE, [path.join(DIR, 'build-symbol-graph.mjs'), '--root', fx, '--output', out], { stdio: ['ignore', 'pipe', 'pipe'] });
    execFileSync(NODE, [path.join(DIR, 'triage-repo.mjs'), '--root', fx, '--output', out], { stdio: ['ignore', 'pipe', 'pipe'] });
    const symMtime = readFileSync(path.join(out, 'symbols.json')).length;
    const triageMtime = readFileSync(path.join(out, 'triage.json')).length;
    assert('P13.CC2-pre. symbols.json present, topology.json absent',
      existsSync(path.join(out, 'symbols.json')) &&
      !existsSync(path.join(out, 'topology.json')));

    const intent = { names: [], shapes: [], files: ['src/a.ts'], dependencies: [], plannedTypeEscapes: [] };
    const intentPath = path.join(out, 'intent.json');
    writeFileSync(intentPath, JSON.stringify(intent));

    // Capture stderr so we can verify ONLY measure-topology.mjs ran.
    const res = execFileSync(NODE, [PREWRITE, '--root', fx, '--output', out, '--intent', intentPath], {
      stdio: ['ignore', 'pipe', 'pipe'],
    });
    assert('P13.CC2. partial cold-cache succeeds',
      res.length > 0);
    assert('P13.CC2b. topology.json now present',
      existsSync(path.join(out, 'topology.json')));
    assert('P13.CC2c. symbols.json not re-built (size unchanged)',
      readFileSync(path.join(out, 'symbols.json')).length === symMtime);
    assert('P13.CC2d. triage.json not re-built (size unchanged)',
      readFileSync(path.join(out, 'triage.json')).length === triageMtime);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// Case 3: --no-fresh-audit — no cold-cache spawn.
{
  const fx = mkdtempSync(path.join(tmpdir(), 'pw-coldcache-nofresh-'));
  const out = mkdtempSync(path.join(tmpdir(), 'pw-coldcache-nofresh-out-'));
  try {
    buildFixture(fx);
    const intent = { names: ['anything'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] };
    const intentPath = path.join(out, 'intent.json');
    writeFileSync(intentPath, JSON.stringify(intent));

    const stdout = execFileSync(NODE, [PREWRITE,
      '--root', fx, '--output', out, '--intent', intentPath, '--no-fresh-audit',
    ], { stdio: ['ignore', 'pipe', 'pipe'] }).toString('utf8');

    assert('P13.CC3. --no-fresh-audit → NO producer spawned',
      !existsSync(path.join(out, 'symbols.json')) &&
      !existsSync(path.join(out, 'topology.json')) &&
      !existsSync(path.join(out, 'triage.json')));
    assert('P13.CC3b. Markdown renders [확인 불가] under --no-fresh-audit',
      stdout.includes('확인 불가'));
    const parsed = JSON.parse(readFileSync(path.join(out, 'pre-write-advisory.latest.json'), 'utf8'));
    assert('P13.CC3c. failures[] lists at least one missing-artifact kind',
      parsed.failures.some((f) => /-missing$/.test(f.kind)));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// Case 4: producer failure — broken package.json.
{
  const fx = mkdtempSync(path.join(tmpdir(), 'pw-coldcache-fail-'));
  const out = mkdtempSync(path.join(tmpdir(), 'pw-coldcache-fail-out-'));
  try {
    write(fx, 'package.json', '{ not valid json ');
    const intent = { names: ['x'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] };
    const intentPath = path.join(out, 'intent.json');
    writeFileSync(intentPath, JSON.stringify(intent));

    // May or may not crash; we accept either exit 0 with failures OR
    // exit non-zero as long as a failure entry is recorded when run
    // completed. Some CLIs treat unparseable package.json as fatal.
    let stdout = '';
    let exitedNormally = false;
    try {
      stdout = execFileSync(NODE, [PREWRITE, '--root', fx, '--output', out, '--intent', intentPath], {
        stdio: ['ignore', 'pipe', 'pipe'],
      }).toString('utf8');
      exitedNormally = true;
    } catch (e) {
      stdout = (e.stdout?.toString?.() ?? '') + (e.stderr?.toString?.() ?? '');
    }
    assert('P13.CC4. CLI does not hang on producer failure',
      exitedNormally || stdout.length > 0);
    if (exitedNormally) {
      const parsed = JSON.parse(readFileSync(path.join(out, 'pre-write-advisory.latest.json'), 'utf8'));
      const hasFail = parsed.failures.some((f) => /cold-cache/.test(f.kind) || /parse-error/.test(f.kind) || /missing/.test(f.kind));
      assert('P13.CC4b. producer failure recorded in failures[]', hasFail,
        `failures=${JSON.stringify(parsed.failures)}`);
    }
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// Case 5: shell-safety — path contains spaces AND $.
{
  const parent = mkdtempSync(path.join(tmpdir(), 'pw-coldcache-shell-'));
  const fx = path.join(parent, 'my $fixture');
  const out = path.join(parent, 'my $output');
  mkdirSync(fx, { recursive: true });
  mkdirSync(out, { recursive: true });
  try {
    buildFixture(fx);
    const intent = { names: ['formatDate'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] };
    const intentPath = path.join(out, 'intent.json');
    writeFileSync(intentPath, JSON.stringify(intent));

    const stdout = runPreWrite(fx, out, intentPath);
    assert('P13.CC5. space + $ path: cold-cache + render succeed',
      stdout.includes('### Grounded facts') && stdout.includes('formatDate'));
    assert('P13.CC5b. symbols.json produced under space+$ path',
      existsSync(path.join(out, 'symbols.json')));
  } finally {
    rmSync(parent, { recursive: true, force: true });
  }
}

// Case 6: stdout / stderr separation.
{
  const fx = mkdtempSync(path.join(tmpdir(), 'pw-coldcache-std-'));
  const out = mkdtempSync(path.join(tmpdir(), 'pw-coldcache-std-out-'));
  try {
    buildFixture(fx);
    const intent = { names: ['formatDate'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] };
    const intentPath = path.join(out, 'intent.json');
    writeFileSync(intentPath, JSON.stringify(intent));

    // Capture stdout and stderr independently.
    const child = spawnSync(NODE, [PREWRITE,
      '--root', fx, '--output', out, '--intent', intentPath,
    ], { encoding: 'utf8' });

    assert('P13.CC6. stdout contains advisory header',
      child.stdout.includes('## pre-write advisory'));
    assert('P13.CC6b. stderr does NOT contain advisory header',
      !child.stderr.includes('## pre-write advisory'));
    assert('P13.CC6c. stderr contains cold-cache diagnostic lines',
      /\[pre-write\] cold-cache/.test(child.stderr),
      `stderr=${child.stderr.slice(0, 300)}`);
    assert('P13.CC6d. stdout does NOT contain [pre-write] diagnostic prefix',
      !/\[pre-write\] cold-cache/.test(child.stdout));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// Case 7: timeout — very low threshold forces timeout.
{
  const fx = mkdtempSync(path.join(tmpdir(), 'pw-coldcache-timeout-'));
  const out = mkdtempSync(path.join(tmpdir(), 'pw-coldcache-timeout-out-'));
  try {
    buildFixture(fx);
    const intent = { names: ['formatDate'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] };
    const intentPath = path.join(out, 'intent.json');
    writeFileSync(intentPath, JSON.stringify(intent));

    // 1ms timeout — any producer will overrun.
    const res = execFileSync(NODE, [PREWRITE,
      '--root', fx, '--output', out, '--intent', intentPath,
    ], {
      stdio: ['ignore', 'pipe', 'pipe'],
      env: { ...process.env, PRE_WRITE_COLD_CACHE_TIMEOUT_MS: '1' },
    });
    assert('P13.CC7. CLI exits 0 even when cold-cache times out',
      res.length > 0);

    const parsed = JSON.parse(readFileSync(path.join(out, 'pre-write-advisory.latest.json'), 'utf8'));
    const hasTimeout = parsed.failures.some((f) => /timeout/.test(f.kind));
    assert('P13.CC7b. failures[] records cold-cache-*-timeout entry',
      hasTimeout,
      `failures=${JSON.stringify(parsed.failures)}`);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ Cue tiers: createLogger weak token suppression reaches JSON artifact ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'pw-cue-create-'));
  const out = mkdtempSync(path.join(tmpdir(), 'pw-cue-create-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'pw-cue-create', type: 'module' }));
    write(fx, 'src/store.ts', 'export const createStore = () => ({});\n');
    write(fx, 'src/storage.ts', 'export const createJSONStorage = () => ({});\n');
    const intent = {
      names: [{ name: 'createLogger', kind: 'function', why: 'create a logger helper' }],
      shapes: [],
      files: [],
      dependencies: [],
      plannedTypeEscapes: [],
    };
    const intentPath = path.join(out, 'intent.json');
    writeFileSync(intentPath, JSON.stringify(intent));

    const stdout = runPreWrite(fx, out, intentPath);
    const parsed = JSON.parse(readFileSync(path.join(out, 'pre-write-advisory.latest.json'), 'utf8'));
    assert('P14.CUE1. create-only hints do not render in default Markdown',
      !stdout.includes('createStore') && !stdout.includes('createJSONStorage'),
      stdout);
    const tokenSuppressed = parsed.suppressedCues?.filter((cue) =>
      cue.reason === 'domain-token-overlap' &&
      cue.tokenPolicyVersion === 'prewrite-token-policy-v1') ?? [];
    const servicePolicyMuted = parsed.suppressedCues?.filter((cue) =>
      cue.evidenceLane === 'service-operation-sibling' &&
      cue.reason === 'service-sibling-insufficient-suppressed-support' &&
      cue.policyVersion === 'prewrite-service-operation-sibling-cue-v1') ?? [];
    assert('P14.CUE2. create-only hints are recorded as suppressedCues',
      tokenSuppressed.length >= 2 && servicePolicyMuted.length >= 2,
      JSON.stringify(parsed.suppressedCues));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
