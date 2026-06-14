// Product-surface contract: the repo may keep many engine scripts, but
// public docs and package metadata must converge on one shared engine,
// three model-facing skill surfaces, and the stable validation modes.

import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

function countOccurrences(text, needle) {
  return text.split(needle).length - 1;
}

function paragraphStartingWith(text, marker) {
  const normalized = text.replace(/\r\n/g, '\n');
  const start = normalized.indexOf(marker);
  if (start < 0) return '';
  const end = normalized.indexOf('\n\n', start);
  return normalized.slice(start, end < 0 ? normalized.length : end);
}

const PACKAGE = JSON.parse(readFileSync(path.join(DIR, 'package.json'), 'utf8'));
const README = readFileSync(path.join(DIR, 'README.md'), 'utf8');
const DOCS_README = readFileSync(path.join(DIR, 'docs/README.md'), 'utf8');
const INTERNAL_ENGINE = readFileSync(path.join(DIR, 'docs/internal-engine.md'), 'utf8');
const SKILL = readFileSync(path.join(DIR, 'SKILL.md'), 'utf8');
const WRITE_GATE_SKILL = readFileSync(path.join(DIR, 'SKILL.write-gate.md'), 'utf8');
const CANON_SKILL = readFileSync(path.join(DIR, 'SKILL.canon.md'), 'utf8');
const PRODUCT_SURFACE = readFileSync(path.join(DIR, 'docs/product-surface.md'), 'utf8');
const HISTORY_README = readFileSync(path.join(DIR, 'docs/history/README.md'), 'utf8');
const SPEC_README = readFileSync(path.join(DIR, 'docs/spec/README.md'), 'utf8');
const LAB_README = readFileSync(path.join(DIR, 'docs/lab/README.md'), 'utf8');
const GITIGNORE = readFileSync(path.join(DIR, '.gitignore'), 'utf8');
const PLUGIN = JSON.parse(readFileSync(path.join(DIR, '.claude-plugin/plugin.json'), 'utf8'));
const MARKETPLACE = JSON.parse(readFileSync(path.join(DIR, '.claude-plugin/marketplace.json'), 'utf8'));
const REFACTOR_PLAN_COMMAND = readFileSync(path.join(DIR, 'commands/refactor-plan.md'), 'utf8');
const AUDIT_COMMAND = readFileSync(path.join(DIR, 'commands/audit.md'), 'utf8');
const FULL_COMMAND = readFileSync(path.join(DIR, 'commands/full.md'), 'utf8');
const WELCOME_COMMAND = readFileSync(path.join(DIR, 'commands/welcome.md'), 'utf8');
const DEFAULT_COMMAND = readFileSync(path.join(DIR, 'commands/lumin-repo-lens-lab.md'), 'utf8');
const PRE_WRITE_COMMAND = readFileSync(path.join(DIR, 'commands/pre-write.md'), 'utf8');
const POST_WRITE_COMMAND = readFileSync(path.join(DIR, 'commands/post-write.md'), 'utf8');
const CANON_DRAFT_COMMAND = readFileSync(path.join(DIR, 'commands/canon-draft.md'), 'utf8');
const CHECK_CANON_COMMAND = readFileSync(path.join(DIR, 'commands/check-canon.md'), 'utf8');
const COMMAND_ROUTING = readFileSync(path.join(DIR, 'references/command-routing.md'), 'utf8');
const STRUCTURAL_REVIEW_WORKFLOW_PATH = path.join(DIR, 'references/structural-review-workflow.md');
const STRUCTURAL_REVIEW_WORKFLOW = existsSync(STRUCTURAL_REVIEW_WORKFLOW_PATH)
  ? readFileSync(STRUCTURAL_REVIEW_WORKFLOW_PATH, 'utf8')
  : '';
const REPORT_TEMPLATE = readFileSync(path.join(DIR, 'templates/report-template.md'), 'utf8');
const REFACTOR_PLAN_TEMPLATE = readFileSync(path.join(DIR, 'templates/refactor-plan-template.md'), 'utf8');
const REVIEW_CHECKLIST = readFileSync(path.join(DIR, 'templates/REVIEW_CHECKLIST.md'), 'utf8');
const REVIEW_CHECKLIST_SHORT = readFileSync(path.join(DIR, 'templates/REVIEW_CHECKLIST_SHORT.md'), 'utf8');
const REFACTOR_PLAN_POLICY = readFileSync(path.join(DIR, 'references/refactor-plan-policy.md'), 'utf8');
const TEMPLATES_README = readFileSync(path.join(DIR, 'templates/README.md'), 'utf8');
const HELP = execFileSync(process.execPath, [path.join(DIR, 'audit-repo.mjs'), '--help'], {
  cwd: DIR,
  encoding: 'utf8',
  stdio: ['ignore', 'pipe', 'pipe'],
});
const HELP_FLAT = HELP.replace(/\s+/g, ' ');
const README_FLAT = README.replace(/\s+/g, ' ');
const COMMAND_ROUTING_FLAT = COMMAND_ROUTING.replace(/\s+/g, ' ');
const STRUCTURAL_REVIEW_WORKFLOW_FLAT = STRUCTURAL_REVIEW_WORKFLOW.replace(/\s+/g, ' ');
const PRODUCT_SURFACE_FLAT = PRODUCT_SURFACE.replace(/\s+/g, ' ');
const ALL_SKILL_SURFACES = [SKILL, WRITE_GATE_SKILL, CANON_SKILL].join('\n---surface---\n');
const ENGLISH_PUBLIC_DOCS = [
  ['README.md', README],
  ['SKILL.md', SKILL],
  ['SKILL.write-gate.md', WRITE_GATE_SKILL],
  ['SKILL.canon.md', CANON_SKILL],
  ['references/command-routing.md', COMMAND_ROUTING],
  ['references/glossary.md', readFileSync(path.join(DIR, 'references/glossary.md'), 'utf8')],
  ['references/refactor-plan-policy.md', REFACTOR_PLAN_POLICY],
  ['references/structural-review-workflow.md', STRUCTURAL_REVIEW_WORKFLOW],
  ['templates/report-template.md', REPORT_TEMPLATE],
  ['templates/REVIEW_CHECKLIST.md', REVIEW_CHECKLIST],
  ['templates/REVIEW_CHECKLIST_SHORT.md', REVIEW_CHECKLIST_SHORT],
];
const KOREAN_EPISTEMIC_TOKENS = ['[확인 불가]', '확인 불가 / unknown', '확인 불가', '~인 것 같아요'];

const CAPABILITIES = ['audit', 'pre-write', 'post-write', 'canon-draft', 'check-canon'];
const COMMANDS = ['lumin-repo-lens-lab', 'welcome', 'full', ...CAPABILITIES, 'refactor-plan'];

assert('S1. package.json exposes a recommended bin pointing at audit-repo.mjs',
  PACKAGE.name === 'lumin-repo-lens-lab-scripts' &&
  PACKAGE.bin &&
  PACKAGE.bin['lumin-repo-lens-lab'] === './audit-repo.mjs' &&
  !Object.hasOwn(PACKAGE.bin, 'lumin-audit') &&
  !Object.hasOwn(PACKAGE.bin, 'grounded-audit'),
  JSON.stringify(PACKAGE.bin));

assert('S2. package.json description names audit-repo.mjs as the public CLI',
  typeof PACKAGE.description === 'string' &&
  PACKAGE.description.includes('TS/JS monorepo evidence engine') &&
  PACKAGE.description.includes('public CLI: audit-repo.mjs'),
  PACKAGE.description);

assert('S2b. package.json exposes offline skill-triggering lint but keeps live sweeps opt-in',
  PACKAGE.scripts?.['build:plugin'] === 'node scripts/build-plugin-package.mjs' &&
  PACKAGE.scripts?.['check:skill-triggering'] === 'node test-harness/lib/lint-prompts.mjs' &&
  PACKAGE.scripts?.['check:behavior'] === 'node test-harness/lib/verify-behavior-corpus.mjs test-harness/behavior/cases.json' &&
  PACKAGE.scripts?.ci?.includes('check:skill-triggering') &&
  PACKAGE.scripts?.ci?.includes('check:behavior') &&
  !PACKAGE.scripts?.test?.includes('run-all.sh'),
  JSON.stringify(PACKAGE.scripts, null, 2));

assert('S3. README names audit-repo.mjs as the public CLI and lists the stable validation modes',
  README.includes('use the public wrapper:') &&
  README.includes('npm ci') &&
  README.includes('node skills/lumin-repo-lens-lab/scripts/audit-repo.mjs') &&
  README.includes('npm run build:plugin') &&
  README.includes('dist/lumin-repo-lens-lab-plugin') &&
  README.includes('Lumin Repo Lens') &&
  !README.includes('grounded-audit') &&
  !README.includes('lumin-audit') &&
  README.includes('lumin-repo-lens-lab-write-gate') &&
  README.includes('lumin-repo-lens-lab-canon') &&
  CAPABILITIES.every((name) => README.includes(`\`${name}\``)),
  README);

assert('S3b. README leads Claude Code users through marketplace install before Codex link install',
  README.includes('## First useful run') &&
  README.includes('/plugin marketplace add annyeong844/lumin-repo-lens-lab') &&
  README.includes('/plugin install lumin-repo-lens-lab@annyeong844-marketplace') &&
  README.includes('/reload-plugins') &&
  README.includes('/lumin-repo-lens-lab') &&
  README.includes('### Codex-native install') &&
  README.indexOf('## First useful run') >= 0 &&
  README.indexOf('## First useful run') < README.indexOf('### Codex-native install'),
  README.slice(0, 2400));

assert('S3c. README documents conservative evidence boundaries and the artifact reading path',
  README.includes('Function-clone cues are review cues') &&
  README.includes('not semantic-equivalence claims') &&
  README.includes('Shape index is exact') &&
  README.includes('nullable or widened types') &&
  README.includes('Start from `audit-summary.latest.md`, `manifest.json`, and `checklist-facts.json`') &&
  README.includes('open raw JSON artifacts only for the claim being cited'),
  README);

assert('S4. README explicitly demotes sibling root scripts to internal engine entrypoints',
  /internal engine entrypoints/i.test(README) &&
  README.includes('They are intentionally') &&
  /not\s+the\s+preferred\s+user-facing\s+interface/.test(README) &&
  README.includes('`docs/internal-engine.md`'),
  README);

assert('S5. SKILL.md public surface section names the audit surface and sibling surfaces',
  SKILL.includes('## Public Surface') &&
  SKILL.includes('LLM-facing repo evidence engine') &&
  SKILL.includes('Operator: the model reading this skill') &&
  !SKILL.includes('This is a vibe-coder-facing skill') &&
  SKILL.includes('Use the recommended orchestrator first.') &&
  SKILL.includes('Below, `<audit-repo>` means the command path for the current context.') &&
  SKILL.includes('references/glossary.md') &&
  SKILL.includes('node audit-repo.mjs') &&
  SKILL.includes('node scripts/audit-repo.mjs') &&
  SKILL.includes('This surface owns `audit`, `welcome`, and `refactor-plan`') &&
  SKILL.includes('lumin-repo-lens-lab-write-gate') &&
  SKILL.includes('lumin-repo-lens-lab-canon') &&
  !SKILL.includes('DOMAIN_CLUSTER_DETECTED') &&
  !SKILL.includes('references/pre-write-intent-shape.md') &&
  CAPABILITIES.every((name) => SKILL.includes(`\`${name}\``)),
  SKILL.slice(0, 1200));

assert('S5b. split skill source files separate audit, write-gate, and canon responsibility',
  /description:\s+"?Audit TypeScript\/JavaScript repos for structural debt/.test(SKILL) &&
  SKILL.includes('dead exports, cycles, oversized modules') &&
  /description:\s+"?Use before\/after TS\/JS code changes/.test(WRITE_GATE_SKILL) &&
  WRITE_GATE_SKILL.includes('add, edit, move, rename, refactor') &&
  WRITE_GATE_SKILL.includes('Infer intent from plain language') &&
  /description:\s+"?Maintainer-only canon surface/.test(CANON_SKILL) &&
  WRITE_GATE_SKILL.includes('name: lumin-repo-lens-lab-write-gate') &&
  WRITE_GATE_SKILL.includes('silent-new type escapes') &&
  !WRITE_GATE_SKILL.split('---')[1].includes('scan-range parity') &&
  WRITE_GATE_SKILL.includes('pre-write` and `post-write` together') &&
  WRITE_GATE_SKILL.includes('pre-write-advisory.<invocationId>.json') &&
  WRITE_GATE_SKILL.includes('Below, `<SKILL_ROOT>` means') &&
  WRITE_GATE_SKILL.includes('Below, `<audit-repo>` means whichever') &&
  WRITE_GATE_SKILL.includes('/lumin-repo-lens-lab:pre-write') &&
  WRITE_GATE_SKILL.includes('/lumin-repo-lens-lab:post-write') &&
  !WRITE_GATE_SKILL.includes('Reference paths below live under') &&
  WRITE_GATE_SKILL.includes('<SKILL_ROOT>/references/pre-write-intent-shape.md') &&
  WRITE_GATE_SKILL.includes('<SKILL_ROOT>/canonical/pre-write-gate.md') &&
  WRITE_GATE_SKILL.includes('<SKILL_ROOT>/canonical/any-contamination.md') &&
  WRITE_GATE_SKILL.includes('<SKILL_ROOT>/references/lifecycle-modes.md') &&
  WRITE_GATE_SKILL.includes('<SKILL_ROOT>/references/false-positive-index.md') &&
  !WRITE_GATE_SKILL.includes('<SKILL_ROOT>/references/false-positive-patterns.md') &&
  WRITE_GATE_SKILL.includes('long FP case ledger') &&
  WRITE_GATE_SKILL.includes('DOMAIN_CLUSTER_DETECTED') &&
  WRITE_GATE_SKILL.includes('same uninterrupted change transaction') &&
  WRITE_GATE_SKILL.includes('Do not use `pre-write-advisory.latest.json` after another pre-write run') &&
  WRITE_GATE_SKILL.includes('hand off to `lumin-repo-lens-lab`') &&
  WRITE_GATE_SKILL.includes('hand off to `lumin-repo-lens-lab-canon`') &&
  WRITE_GATE_SKILL.includes('Do not ask normal chat users to hand-write JSON') &&
  CANON_SKILL.includes('name: lumin-repo-lens-lab-canon') &&
  CANON_SKILL.includes('canon-draft` and `check-canon` together') &&
  CANON_SKILL.includes('Do not use casual hedging') &&
  CANON_SKILL.includes('Below, `<SKILL_ROOT>` means') &&
  CANON_SKILL.includes('Below, `<audit-repo>` means whichever') &&
  CANON_SKILL.includes('/lumin-repo-lens-lab:canon-draft') &&
  CANON_SKILL.includes('/lumin-repo-lens-lab:check-canon') &&
  !CANON_SKILL.includes('Reference paths below live under') &&
  CANON_SKILL.includes('<SKILL_ROOT>/canonical/canon-drift.md') &&
  CANON_SKILL.includes('<SKILL_ROOT>/canonical/fact-model.md') &&
  CANON_SKILL.includes('<SKILL_ROOT>/references/lifecycle-modes.md') &&
  CANON_SKILL.includes('hand off to `lumin-repo-lens-lab`') &&
  CANON_SKILL.includes('hand off to `lumin-repo-lens-lab-write-gate`') &&
  CANON_SKILL.includes('Drafts are proposals, not promoted truth'),
  `${SKILL.slice(0, 700)}\n${WRITE_GATE_SKILL}\n${CANON_SKILL}`);

assert('S5c. shared path tokens are duplicated only where independently loaded SKILL surfaces need them',
  countOccurrences(ALL_SKILL_SURFACES, 'Below, `<audit-repo>` means') === 3 &&
  countOccurrences(ALL_SKILL_SURFACES, 'Below, `<SKILL_ROOT>` means') === 2 &&
  paragraphStartingWith(WRITE_GATE_SKILL, 'Below, `<SKILL_ROOT>` means') ===
    paragraphStartingWith(CANON_SKILL, 'Below, `<SKILL_ROOT>` means') &&
  countOccurrences(WRITE_GATE_SKILL, '<SKILL_ROOT>/') >= 6 &&
  countOccurrences(CANON_SKILL, '<SKILL_ROOT>/') >= 5 &&
  !WRITE_GATE_SKILL.includes('Read `references/') &&
  !WRITE_GATE_SKILL.includes('Read `canonical/') &&
  !CANON_SKILL.includes('Read `references/') &&
  !CANON_SKILL.includes('Read `canonical/'),
  `${paragraphStartingWith(WRITE_GATE_SKILL, 'Below, `<SKILL_ROOT>` means')}\n---\n${paragraphStartingWith(CANON_SKILL, 'Below, `<SKILL_ROOT>` means')}`);

assert('S6. SKILL.md keeps runtime canon slim and lab dirs outside the public contract',
  SKILL.includes('The runtime canon spine lives') &&
  SKILL.includes('in `canonical/`; templates live in `templates/`; self-contained') &&
  SKILL.includes('operating guides live in `references/`') &&
  SKILL.includes('Maintainer-only history,') &&
  SKILL.includes('self-audit fact snapshots'),
  SKILL);

{
  const offenders = [];
  for (const [name, text] of ENGLISH_PUBLIC_DOCS) {
    for (const token of KOREAN_EPISTEMIC_TOKENS) {
      if (text.includes(token)) offenders.push(`${name}: ${token}`);
    }
  }
  assert('S6a. English public docs use the English `unknown` evidence label',
    offenders.length === 0 &&
      ENGLISH_PUBLIC_DOCS.every(([, text]) => !text.includes('확인 불가')) &&
      REVIEW_CHECKLIST.includes('[unknown, scan range:') &&
      REVIEW_CHECKLIST_SHORT.includes('[unknown, scan range:'),
    offenders.join('\n'));
}

assert('S6b. plugin manifest uses default component discovery and ships marketplace metadata',
  !Object.hasOwn(PLUGIN, 'skills') &&
  !Object.hasOwn(PLUGIN, 'commands') &&
  PLUGIN.author?.name === 'annyeong844' &&
  PLUGIN.repository === 'https://github.com/annyeong844/lumin-repo-lens-lab' &&
  MARKETPLACE.name === 'annyeong844-marketplace' &&
  MARKETPLACE.owner?.name === 'annyeong844' &&
  PLUGIN.name === 'lumin-repo-lens-lab' &&
  PLUGIN.description.includes('repo structure lens') &&
  MARKETPLACE.plugins?.[0]?.name === 'lumin-repo-lens-lab' &&
  MARKETPLACE.plugins?.[0]?.description.includes('repo structure lens') &&
  MARKETPLACE.plugins?.[0]?.source === './' &&
  COMMANDS.every((name) => existsSync(path.join(DIR, 'commands', `${name}.md`))),
  `${JSON.stringify(PLUGIN, null, 2)}\n${JSON.stringify(MARKETPLACE, null, 2)}`);

assert('S6b2. welcome command gives a gentle first-touch route without running a scan',
  WELCOME_COMMAND.includes('Mode: `welcome`') &&
  WELCOME_COMMAND.includes('references/command-routing.md') &&
  COMMAND_ROUTING.includes('### welcome') &&
  COMMAND_ROUTING.includes('Do not run a scan immediately') &&
  COMMAND_ROUTING.includes('at most three choices') &&
  COMMAND_ROUTING.includes('Do not tell first-time users that they must') &&
  COMMAND_ROUTING.includes('/lumin-repo-lens-lab:refactor-plan') &&
  COMMAND_ROUTING.includes('Maintainers can call') &&
  COMMAND_ROUTING.includes('/lumin-repo-lens-lab:canon-draft') &&
  COMMAND_ROUTING.includes('/lumin-repo-lens-lab:check-canon') &&
  README.includes('/lumin-repo-lens-lab:welcome') &&
  SKILL.includes('/lumin-repo-lens-lab:welcome'),
  `${WELCOME_COMMAND}\n${COMMAND_ROUTING}`);

assert('S6b3. pre-write command accepts natural language and hides intent JSON from normal chat users',
  PRE_WRITE_COMMAND.includes('natural-language change request') &&
  PRE_WRITE_COMMAND.includes('skills/lumin-repo-lens-lab-write-gate/SKILL.md') &&
  POST_WRITE_COMMAND.includes('skills/lumin-repo-lens-lab-write-gate/SKILL.md') &&
  CANON_DRAFT_COMMAND.includes('skills/lumin-repo-lens-lab-canon/SKILL.md') &&
  CHECK_CANON_COMMAND.includes('skills/lumin-repo-lens-lab-canon/SKILL.md') &&
  COMMAND_ROUTING.includes('If `--intent` is not provided') &&
  COMMAND_ROUTING.includes('do not ask them to write JSON') &&
  COMMAND_ROUTING.includes('Infer the smallest compact') &&
  COMMAND_ROUTING.includes('intent you can from the request') &&
  README.includes('/lumin-repo-lens-lab:pre-write') &&
  README.includes('Ask naturally') &&
  README.includes('assistant infers the compact') &&
  README.includes('intent internally'),
  `${PRE_WRITE_COMMAND}\n${POST_WRITE_COMMAND}\n${CANON_DRAFT_COMMAND}\n${CHECK_CANON_COMMAND}\n${COMMAND_ROUTING}`);

assert('S6c. refactor-plan command is template-led and defaults to readable coaching output',
  REFACTOR_PLAN_COMMAND.includes('Mode: `refactor-plan`') &&
  REFACTOR_PLAN_COMMAND.includes('references/command-routing.md') &&
  COMMAND_ROUTING.includes('references/refactor-plan-policy.md') &&
  COMMAND_ROUTING.includes('templates/refactor-plan-template.md') &&
  COMMAND_ROUTING.includes('coaching command, not an engine mode') &&
  COMMAND_ROUTING.includes('no `audit-repo.mjs --refactor-plan` flag') &&
  COMMAND_ROUTING.includes('Run a full audit for first baseline') &&
  COMMAND_ROUTING.includes('SHORT') &&
  COMMAND_ROUTING.includes('pre-write handoff') &&
  REFACTOR_PLAN_POLICY.includes('audit evidence -> LLM-authored plan -> implementation by user/agent -> scoped quick audit -> closeout') &&
  REFACTOR_PLAN_POLICY.includes('has no CLI flag, no') &&
  REFACTOR_PLAN_POLICY.includes('## Tone Contract') &&
  REFACTOR_PLAN_POLICY.includes('## Phase Scope Rules') &&
  REFACTOR_PLAN_POLICY.includes('## Lifecycle Integration') &&
  REFACTOR_PLAN_POLICY.includes('## Ripple-Aware Changes') &&
  REFACTOR_PLAN_POLICY.includes('## Selection Rules') &&
  REFACTOR_PLAN_POLICY.includes('## Proof Requests') &&
  REFACTOR_PLAN_POLICY.includes('one public voice') &&
  REFACTOR_PLAN_POLICY.includes('phrase criticism as actionable improvement') &&
  REFACTOR_PLAN_POLICY.includes('Semantic phase grouping is allowed because it is LLM judgment over') &&
  REFACTOR_PLAN_POLICY.includes('pre-write intent for Phase 1') &&
  REFACTOR_PLAN_TEMPLATE.includes('## SHORT Mode') &&
  REFACTOR_PLAN_TEMPLATE.includes('## FULL Mode') &&
  REFACTOR_PLAN_TEMPLATE.includes('Ask the coding agent:') &&
  REFACTOR_PLAN_TEMPLATE.includes('Default length target: 20 to 35 lines') &&
  REFACTOR_PLAN_TEMPLATE.includes('Only include machine-readable scope JSON when the user asks for it') &&
  !REFACTOR_PLAN_TEMPLATE.includes('imply the user should have known better') &&
  STRUCTURAL_REVIEW_WORKFLOW.includes('short four-section chat plan'),
  `${REFACTOR_PLAN_COMMAND}\n${REFACTOR_PLAN_POLICY.slice(0, 1600)}\n${REFACTOR_PLAN_TEMPLATE.slice(0, 1600)}`);

assert('S6d. validation modes stay distinct from refactor-plan coaching mode',
  AUDIT_COMMAND.includes('Mode: `audit`') &&
  FULL_COMMAND.includes('Mode: `full`') &&
  FULL_COMMAND.includes('Required profile: `full`') &&
  FULL_COMMAND.includes('/lumin-repo-lens-lab:full') &&
  AUDIT_COMMAND.includes('references/command-routing.md') &&
  COMMAND_ROUTING.includes('### full') &&
  COMMAND_ROUTING.includes('/lumin-repo-lens-lab:full') &&
  COMMAND_ROUTING.includes('forces') &&
  COMMAND_ROUTING.includes('`--profile full`') &&
  COMMAND_ROUTING.includes('One public voice') &&
  COMMAND_ROUTING.includes('Cold artifacts are the source of truth') &&
  COMMAND_ROUTING.includes('audit-summary.latest.md') &&
  COMMAND_ROUTING.includes('artifact map') &&
  COMMAND_ROUTING.includes('route to `refactor-plan`') &&
  COMMAND_ROUTING.includes('Default to the short') &&
  SKILL.includes('The engine preserves cold artifacts on disk') &&
  SKILL.includes('`refactor-plan` is a coaching mode') &&
  SKILL.includes('it has no CLI') &&
  PRODUCT_SURFACE.includes('human-in-the-loop boundary'),
  `${AUDIT_COMMAND}\n${FULL_COMMAND}\n${REFACTOR_PLAN_COMMAND}\n${COMMAND_ROUTING}`);

assert('S6d2. full audit review pack is an in-session reminder surface, not an external API runner',
  COMMAND_ROUTING_FLAT.includes('audit-review-pack.latest.md') &&
  COMMAND_ROUTING_FLAT.includes('reviewer-lane surface for deep review') &&
  COMMAND_ROUTING_FLAT.includes('engine never calls external') &&
  COMMAND_ROUTING_FLAT.includes('main-controller artifact brief') &&
  COMMAND_ROUTING_FLAT.includes('read the lanes locally') &&
  COMMAND_ROUTING_FLAT.includes('built-in reviewer subagents') &&
  COMMAND_ROUTING_FLAT.includes('translate a chosen lane into a codebase-reading task') &&
  COMMAND_ROUTING_FLAT.includes('Do not paste checklist or artifact lanes wholesale') &&
  README_FLAT.includes('audit-review-pack.latest.md') &&
  README_FLAT.includes('does not call any') &&
  README_FLAT.includes('Claude Code') &&
  README_FLAT.includes('focused codebase-reading assignment') &&
  README_FLAT.includes('Subagents should inspect repository files directly'),
  `${README}\n${COMMAND_ROUTING}`);

assert('S6f. default slash command runs a baseline-aware current-workspace audit instead of asking for mode selection',
  DEFAULT_COMMAND.includes('Mode: `default`') &&
  DEFAULT_COMMAND.includes('If `Arguments` is empty') &&
  DEFAULT_COMMAND.includes('Do not ask a follow-up') &&
  DEFAULT_COMMAND.includes('baseline-aware repo lens pass') &&
  DEFAULT_COMMAND.includes('Use full for a first or') &&
  DEFAULT_COMMAND.includes('do not print a mode menu') &&
  DEFAULT_COMMAND.includes('references/command-routing.md') &&
  COMMAND_ROUTING.includes('do not ask which mode') &&
  COMMAND_ROUTING.includes('one-click path') &&
  COMMAND_ROUTING.includes('do not wait for confirmation') &&
  COMMAND_ROUTING.includes('choose the profile by cadence') &&
  COMMAND_ROUTING.includes('--root . --output .audit --profile full') &&
  COMMAND_ROUTING.includes('--root . --output .audit --profile quick') &&
  COMMAND_ROUTING.includes('REVIEW_CHECKLIST_SHORT') &&
  COMMAND_ROUTING.includes('Checklist gate: the checklist is a required review step') &&
  COMMAND_ROUTING.includes('Short output is not permission to skip it') &&
  COMMAND_ROUTING.includes('triage C/D/E/A/B/F') &&
  COMMAND_ROUTING.includes('open `templates/REVIEW_CHECKLIST.md` and walk it before drafting') &&
  COMMAND_ROUTING.includes('under about 12 bullets') &&
  COMMAND_ROUTING.includes('required feature-discovery tail') &&
  COMMAND_ROUTING.includes('This tail is not decoration') &&
  COMMAND_ROUTING.includes('full checklist로 펼쳐줘') &&
  COMMAND_ROUTING.includes('formal report로 써줘') &&
  COMMAND_ROUTING.includes('due-diligence handoff로 정리해줘') &&
  /Do not\s+omit it after full-profile short answers/.test(COMMAND_ROUTING) &&
  COMMAND_ROUTING.includes('Worth Smoothing Next') &&
  COMMAND_ROUTING.includes('Keep As-Is For Now') &&
  COMMAND_ROUTING.includes('Current State') &&
  readFileSync(path.join(DIR, 'templates/REVIEW_CHECKLIST_SHORT.md'), 'utf8').includes('Ask the coding agent:') &&
  readFileSync(path.join(DIR, 'templates/REVIEW_CHECKLIST_SHORT.md'), 'utf8').includes('Truth Before Warmth') &&
  /over "findings"\s+language/.test(COMMAND_ROUTING) &&
  COMMAND_ROUTING.includes('instead of a mode menu') &&
  README.includes('Quick baseline-aware repo lens pass') &&
  README.includes('First pass, stale or missing artifacts') &&
  README.includes('post-refactor review should run `--profile full`') &&
  /likely\s+false-positive families\s+to keep as-is/.test(COMMAND_ROUTING) &&
  COMMAND_ROUTING.includes('skills/lumin-repo-lens-lab/_engine'),
  `${DEFAULT_COMMAND}\n${COMMAND_ROUTING}`);

assert('S6e. SKILL.md separates short chat-facing reviews from formal report-template reports',
  SKILL.split('\n').length <= 165 &&
  SKILL.includes('references/structural-review-workflow.md') &&
  !SKILL.includes('## Standard Audit Workflow') &&
  !SKILL.includes('## Structural Review Mode') &&
  STRUCTURAL_REVIEW_WORKFLOW.includes('Audit cadence:') &&
  STRUCTURAL_REVIEW_WORKFLOW.includes('For normal chat-facing structural reviews, follow') &&
  SKILL.includes('`templates/REVIEW_CHECKLIST_SHORT.md`.') &&
  STRUCTURAL_REVIEW_WORKFLOW.includes('First pass on a repo') &&
  STRUCTURAL_REVIEW_WORKFLOW.includes('post-refactor review') &&
  STRUCTURAL_REVIEW_WORKFLOW.includes('Short chat output is still allowed after a full run') &&
  SKILL.includes('NO STRUCTURAL REVIEW WITHOUT A CHECKLIST GATE') &&
  STRUCTURAL_REVIEW_WORKFLOW.includes('Checklist gate and output density are separate') &&
  STRUCTURAL_REVIEW_WORKFLOW.includes('The Core Contract makes') &&
  STRUCTURAL_REVIEW_WORKFLOW.includes('required review step, not merely a template') &&
  STRUCTURAL_REVIEW_WORKFLOW.includes('triage C/D/E/A/B/F') &&
  STRUCTURAL_REVIEW_WORKFLOW.includes('does not permit skipping the checklist pass') &&
  /open `templates\/REVIEW_CHECKLIST\.md`\s+and\s+walk it/.test(STRUCTURAL_REVIEW_WORKFLOW) &&
  STRUCTURAL_REVIEW_WORKFLOW_FLAT.includes('For explicit full audit reports, due diligence, CI-style review') &&
  SKILL.includes('follow `templates/report-template.md`') &&
  STRUCTURAL_REVIEW_WORKFLOW.includes('Before finalizing any saved formal report') &&
  STRUCTURAL_REVIEW_WORKFLOW.includes('Do not delegate this closeout to a string heuristic') &&
  REPORT_TEMPLATE.includes('Final author closeout') &&
  REPORT_TEMPLATE.includes('Do not') &&
  REPORT_TEMPLATE.includes('replace this with string-lint heuristics') &&
  TEMPLATES_README.includes('Saved formal reports require') &&
  TEMPLATES_README.includes('final-author closeout pass') &&
  TEMPLATES_README.includes('Use this folder for output shapes') &&
  TEMPLATES_README.includes('Short output does not mean shallow analysis') &&
  TEMPLATES_README.includes('references/refactor-plan-policy.md') &&
  REFACTOR_PLAN_POLICY.includes('if grounded strengths are thin') &&
  TEMPLATES_README.includes('Maintainer-only dogfood notes live in `docs/maintainer/`'),
  SKILL);

assert('S7. audit-repo --help presents the stable validation modes on the recommended entrypoint',
  HELP.includes('lumin-repo-lens-lab public CLI') &&
  HELP.includes('Recommended entrypoint:') &&
  HELP.includes('default: quick') &&
  CAPABILITIES.every((name) => HELP.includes(name)),
  HELP);

assert('S7b. full profile is documented as the shape-index profile, not quick',
  STRUCTURAL_REVIEW_WORKFLOW.includes('`full`: quick plus call graph, barrel discipline, shape index') &&
  README.includes('shape index') &&
  HELP.includes('shape index'),
  `${STRUCTURAL_REVIEW_WORKFLOW}\n${README}\n${HELP}`);

assert('S8. audit-repo --help tells users that sibling root scripts are internal engine entrypoints',
  /internal engine entrypoints/i.test(HELP_FLAT) &&
  HELP_FLAT.includes('public surface is audit-repo.mjs plus the validation modes above'),
  HELP);

assert('S8b. audit routing requires a dead-export FP screen before chat smoothing candidates',
  COMMAND_ROUTING.includes('Before reporting dead-export') &&
  COMMAND_ROUTING.includes('fix-plan.summary') &&
  COMMAND_ROUTING.includes('references/false-positive-index.md') &&
  COMMAND_ROUTING.includes('long FP case ledger is') &&
  COMMAND_ROUTING.includes('do not load historical FP case') &&
  COMMAND_ROUTING.includes('Keep As-Is For Now') &&
  COMMAND_ROUTING.includes('Worth Smoothing Next'),
  COMMAND_ROUTING);

assert('S9. README links the productization map plus history/spec/lab staging docs',
  README.includes('`docs/README.md`') &&
  README.includes('`docs/product-surface.md`') &&
  README.includes('`docs/history/README.md`') &&
  README.includes('`docs/spec/README.md`') &&
  README.includes('`docs/lab/README.md`'),
  README);

assert('S10. docs/product-surface.md defines keep/move/hide boundaries around the public contract',
  PRODUCT_SURFACE.includes('## Public contract') &&
  PRODUCT_SURFACE.includes('## Keep / move / hide') &&
  PRODUCT_SURFACE.includes('`commands/`') &&
  PRODUCT_SURFACE.includes('`refactor-plan`') &&
  PRODUCT_SURFACE.includes('runtime canon spine') &&
  PRODUCT_SURFACE_FLAT.includes('one thin Codex wrapper') &&
  PRODUCT_SURFACE_FLAT.includes('skills/lumin-repo-lens-lab-codex/') &&
  PRODUCT_SURFACE_FLAT.includes('Codex wrapper contains no runtime copy') &&
  PRODUCT_SURFACE_FLAT.includes('maintainer self-audit fact snapshots') &&
  PRODUCT_SURFACE.includes('### Keep at root') &&
  PRODUCT_SURFACE.includes('### Currently staged in history or lab surface') &&
  PRODUCT_SURFACE.includes('### Hide from shipping surface'),
  PRODUCT_SURFACE);

assert('S11. docs/history/README.md anchors history as non-public and preserves the recommended contract',
  HISTORY_README.includes('This area is documentation history, not the public capability contract.') &&
  HISTORY_README.includes('`audit-repo.mjs`') &&
  HISTORY_README.includes('`canonical/`'),
  HISTORY_README);

assert('S12. docs/spec/README.md anchors maintainer specs outside the public contract',
  SPEC_README.includes('This directory holds maintainer-facing design references') &&
  SPEC_README.includes('small public capability surface') &&
  SPEC_README.includes('`templates/REVIEW_CHECKLIST.md`') &&
  SPEC_README.includes('`references/lifecycle-modes.md`') &&
  SPEC_README.includes('`audit-repo.mjs`') &&
  SPEC_README.includes('`canonical/`'),
  SPEC_README);

assert('S13. docs/lab/README.md anchors reproducible lab artifacts outside the public contract',
  LAB_README.includes('reproducible working surfaces') &&
  LAB_README.includes('[`docs/README.md`](../README.md)') &&
  LAB_README.includes('`output/`') &&
  LAB_README.includes('`review-output*/`') &&
  LAB_README.includes('`p6-corpus/`') &&
  LAB_README.includes('`canonical-draft/`') &&
  LAB_README.includes('`audit-artifacts/`') &&
  LAB_README.includes('`audit-artifacts-smoke/`') &&
  LAB_README.includes('`.audit/`') &&
  LAB_README.includes('`.claude/`') &&
  LAB_README.includes('`audit-repo.mjs`') &&
  LAB_README.includes('`canonical/`'),
  LAB_README);

assert('S14. docs/README.md links the public contract plus history/spec/lab staging areas',
  DOCS_README.includes('maintainer-facing map') &&
  DOCS_README.includes('`README.md`') &&
  DOCS_README.includes('`SKILL.md`') &&
  DOCS_README.includes('`audit-repo.mjs`') &&
  DOCS_README.includes('`canonical/` runtime spine') &&
  DOCS_README.includes('`templates/`') &&
  DOCS_README.includes('`references/`') &&
  DOCS_README.includes('[product surface map](product-surface.md)') &&
  DOCS_README.includes('[internal engine map](internal-engine.md)') &&
  DOCS_README.includes('[history staging area](history/README.md)') &&
  DOCS_README.includes('[spec staging area](spec/README.md)') &&
  DOCS_README.includes('[lab staging area](lab/README.md)'),
  DOCS_README);

assert('S15. docs/internal-engine.md groups root scripts while keeping audit-repo.mjs as the recommended public CLI',
  INTERNAL_ENGINE.includes('not the primary public entrypoint') &&
  INTERNAL_ENGINE.includes('`audit-repo.mjs`') &&
  INTERNAL_ENGINE.includes('## Collection and measurement') &&
  INTERNAL_ENGINE.includes('## Classification and reporting') &&
  INTERNAL_ENGINE.includes('## Lifecycle and canon workflows') &&
  INTERNAL_ENGINE.includes('`build-symbol-graph.mjs`') &&
  INTERNAL_ENGINE.includes('`rank-fixes.mjs`') &&
  INTERNAL_ENGINE.includes('`pre-write.mjs`') &&
  INTERNAL_ENGINE.includes('`check-canon.mjs`'),
  INTERNAL_ENGINE);

assert('S16. .gitignore keeps generated lab artifacts out of default tracking',
  GITIGNORE.includes('output/') &&
  GITIGNORE.includes('review-output*/') &&
  GITIGNORE.includes('p6-corpus/') &&
  GITIGNORE.includes('canonical-draft/*.md') &&
  GITIGNORE.includes('.audit/') &&
  GITIGNORE.includes('.claude/') &&
  GITIGNORE.includes('audit-artifacts/') &&
  GITIGNORE.includes('audit-artifacts-smoke/'),
  GITIGNORE);

assert('S17. phase/research docs are staged while user-facing templates/references occupy shipping surface',
  existsSync(path.join(DIR, 'docs/history/phases/p1/session.md')) &&
  existsSync(path.join(DIR, 'docs/history/phases/p5/session.md')) &&
  existsSync(path.join(DIR, 'docs/history/phases/p6/ts-precision-debt.md')) &&
  existsSync(path.join(DIR, 'docs/history/FP-41-regression.md')) &&
  existsSync(path.join(DIR, 'docs/spec/FP-41-sentinel-spec.md')) &&
  existsSync(path.join(DIR, 'docs/spec/SPEC-canon-generator.md')) &&
  existsSync(path.join(DIR, 'templates/report-template.md')) &&
  existsSync(path.join(DIR, 'templates/README.md')) &&
  existsSync(path.join(DIR, 'templates/refactor-plan-template.md')) &&
  existsSync(path.join(DIR, 'templates/REVIEW_CHECKLIST_SHORT.md')) &&
  existsSync(path.join(DIR, 'templates/REVIEW_CHECKLIST.md')) &&
  existsSync(path.join(DIR, 'docs/maintainer/SELF_AUDIT_HANDBOOK.md')) &&
  existsSync(path.join(DIR, 'docs/maintainer/false-positive-patterns-ledger.md')) &&
  existsSync(path.join(DIR, 'references/false-positive-index.md')) &&
  existsSync(path.join(DIR, 'references/false-positive-patterns.md')) &&
  existsSync(path.join(DIR, 'references/pre-write-intent-shape.md')) &&
  existsSync(path.join(DIR, 'references/glossary.md')) &&
  existsSync(path.join(DIR, 'references/refactor-plan-policy.md')) &&
  existsSync(path.join(DIR, 'references/lifecycle-modes.md')) &&
  existsSync(path.join(DIR, 'references/cli-options.md')) &&
  existsSync(path.join(DIR, 'references/operational-gates.md')) &&
  existsSync(path.join(DIR, 'references/language-support.md')) &&
  existsSync(path.join(DIR, 'docs/lab/README.md')) &&
  SKILL.split('\n').length <= 250 &&
  !existsSync(path.join(DIR, 'p1')) &&
  !existsSync(path.join(DIR, 'p2')) &&
  !existsSync(path.join(DIR, 'p3')) &&
  !existsSync(path.join(DIR, 'p4')) &&
  !existsSync(path.join(DIR, 'p5')) &&
  !existsSync(path.join(DIR, 'p6')) &&
  !existsSync(path.join(DIR, 'false-positive-patterns.md')) &&
  !existsSync(path.join(DIR, 'FP-41-regression.md')) &&
  !existsSync(path.join(DIR, 'FP-41-sentinel-spec.md')) &&
  !existsSync(path.join(DIR, 'SPEC-canon-generator.md')) &&
  !existsSync(path.join(DIR, 'report-template.md')) &&
  !existsSync(path.join(DIR, 'REVIEW_CHECKLIST.md')) &&
  !existsSync(path.join(DIR, 'templates/SELF_AUDIT_HANDBOOK.md')),
  'expected research docs under docs/history/docs/spec and user-facing material under templates/references');

{
  const longChecklist = readFileSync(path.join(DIR, 'templates/REVIEW_CHECKLIST.md'), 'utf8');
  const selfAuditHandbook = readFileSync(path.join(DIR, 'docs/maintainer/SELF_AUDIT_HANDBOOK.md'), 'utf8');
  assert('S17b. long checklist stays repo-neutral; self-audit handbook stays maintainer-only',
    !longChecklist.includes('For THIS repo') &&
    !longChecklist.includes('_lib/vocab.mjs is the single source') &&
    longChecklist.includes('This checklist is repo-neutral') &&
    longChecklist.includes('intentionally does not include') &&
    selfAuditHandbook.includes('Use this only when reviewing `lumin-repo-lens-lab` itself') &&
    !longChecklist.includes('docs/maintainer/SELF_AUDIT_HANDBOOK.md') &&
    selfAuditHandbook.includes('_lib/ranking.mjs::TIERS') &&
    selfAuditHandbook.includes('execFileSync'),
    'expected self-reference notes outside templates/REVIEW_CHECKLIST.md and outside the shipping template folder');
}

assert('S18. maintainer skill-triggering harness is present but clearly separate from shipping surface',
  existsSync(path.join(DIR, 'test-harness/README.md')) &&
  existsSync(path.join(DIR, 'test-harness/expectations.json')) &&
  existsSync(path.join(DIR, 'test-harness/behavior/cases.json')) &&
  existsSync(path.join(DIR, 'test-harness/lib/lint-prompts.mjs')) &&
  existsSync(path.join(DIR, 'test-harness/lib/verify.mjs')) &&
  existsSync(path.join(DIR, 'test-harness/lib/verify-citations.mjs')) &&
  existsSync(path.join(DIR, 'test-harness/lib/verify-behavior-corpus.mjs')) &&
  README.includes('The skill-triggering harness is maintainer-only') &&
  README.includes('npm run check:skill-triggering') &&
  README.includes('npm run check:behavior'),
  'expected test-harness/ maintainer checks and README pointer');

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
