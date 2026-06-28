# Triggering Test Harness

> Adapted from `superpowers/tests/skill-triggering/` (Jesse Vincent, MIT).
> Extended with mode-accuracy checks, negative tests, and citation-discipline
> verification appropriate for an evidence-grounded audit skill.

## What this harness proves

`canonical/mode-contract.md` declares an explicit trigger vocabulary (§3.1–§3.5)
and an explicit non-trigger vocabulary (§3.4 guards). `SKILL.md` carries an
independent description string. **Without a triggering test, the gap between
canonical declaration and runtime behavior is invisible.**

This harness closes that gap with five classes of test:

1. **Positive triggering** — naive prompts that should activate the skill in
   the right mode (audit / pre-write / structural-review).
2. **Negative non-triggering** — prompts that should NOT activate the skill
   (guard-only inspections, doc rewrites, conceptual questions). Catches
   over-triggering, which erodes user trust.
3. **Iron-Law compliance under load** — the final assistant message on a
   triggered run must contain `[grounded, ...]` or `[확인 불가, ...]`
   citations. Bare counts without citations are spec violations.
4. **Rule 1 value checks** — saved model output can be checked against
   audit JSON artifacts so `[grounded, artifact.json.path = value]`
   claims become falsifiable, not just stylistic.
5. **Saved-answer behavior checks** — offline final-answer fixtures catch
   internal jargon leaks, review-only overclaims, accidental inheritance
   of `audit-summary.latest.md` ordering, and optional read-trace gaps for
   claims that should have required a specific JSON artifact.

## Why this matters more than superpowers' harness

`superpowers/tests/skill-triggering` only checks "was the Skill tool invoked?"
That works for them — every superpowers skill has one mode. This skill has
**five modes** and a **canon-declared trigger taxonomy**. A pass/fail on
"skill loaded" hides whether the right mode dispatched. So we extend the
verifier to inspect Bash invocations and assistant output structure.

## Layout

```
test-harness/
├── README.md
├── run-test.sh              # single-test runner (calls claude -p)
├── run-all.sh               # batch runner
├── lib/
│   ├── lint-prompts.mjs     # offline schema check (no claude CLI needed)
│   ├── verify.mjs           # JSON-stream parser + expectation checker
│   ├── verify-refactor-plan.mjs # refactor-plan output contract checker
│   ├── verify-citations.mjs # Rule 1 grounded citation value checker
│   └── verify-behavior-corpus.mjs # saved-answer behavior checker
├── behavior/
│   ├── cases.json           # saved-answer expectations
│   └── answers/             # good and known-bad answer fixtures
├── reports/                 # good and known-bad formal report fixtures
├── fixtures/
│   └── tiny-ts-repo/        # 3-file TS repo with planted dead/dup symbols
├── prompts/
│   ├── positive/            # 7 prompts that SHOULD trigger
│   └── negative/            # 4 prompts that SHOULD NOT trigger
└── expectations.json        # per-prompt expected mode/script/citations
```

## How to run

### Prerequisites

- `claude` CLI installed and authenticated (Claude Code).
- This harness must live under the maintainer repo root (so `--plugin-dir`
  points at the right place). Default assumption: harness sits at
  `<repo-root>/test-harness/`. Override with `PLUGIN_DIR=...`.
- `node` ≥ 20.

### Offline lint (no API cost)

Run before every PR. Validates that prompts and expectations stay in sync —
catches "added a prompt, forgot the expectation" type errors:

```bash
node lib/lint-prompts.mjs
```

The package-level maintainer check runs the same offline lint:

```bash
npm run check:skill-triggering
```

### Refactor-plan output verifier

Use this after saving a sample `/lumin-repo-lens-lab:refactor-plan`
answer to Markdown. It turns the template self-check into an executable
maintainer check without calling Claude:

```bash
node lib/verify-refactor-plan.mjs --mode short --expect-code-change sample-plan.md
```

The verifier checks the SHORT sections, evidence anchor, tone guard,
raw-JSON guard, and pre-write handoff for code-changing plans.

### Saved-answer behavior verifier

Use this for offline answer-level regression checks. It does not call a
model and it does not try to rank findings. It verifies saved Markdown
answers against small contracts such as "plain answers do not leak internal
jargon", "review-only dead exports stay caveated", and "cycle claims read
`topology.json`":

```bash
node lib/verify-behavior-corpus.mjs behavior/cases.json
npm run check:behavior
```

Negative fixtures set `"expectPass": false` so the checked-in corpus can
prove the verifier catches bad answers while still keeping CI green.

Cases may include a saved `trace` file plus `mustReadArtifacts` entries. This
is not live telemetry and does not spawn a model; it verifies that a saved
answer claiming, for example, a dependency cycle was paired with a trace that
read `topology.json`.

### Rule 1 citation verifier

Use this after saving a model answer to Markdown. It verifies grounded
labels against the JSON artifacts they cite:

```bash
node lib/verify-citations.mjs --artifacts .audit answer.md
```

It rejects unfalsifiable labels such as `[grounded, source:
topology.json]`, missing artifact paths, placeholder values, and value
mismatches.

### Single test

```bash
./run-test.sh prompts/positive/audit-ko-dead-export.txt
```

On Windows PowerShell with Git Bash and a Claude CLI installed outside the
Bash PATH, use forward slashes and set `CLAUDE_BIN` explicitly:

```powershell
$env:CLAUDE_BIN='<path-to-claude.exe>'
& 'C:\Program Files\Git\usr\bin\bash.exe' ./test-harness/run-test.sh prompts/positive/prewrite-ko-helper.txt
```

### Full sweep

```bash
./run-all.sh
```

Costs roughly 11 prompts × ~3 turns × ~5k tokens average = order-of-magnitude
$1–3 per run. Smoke-test with 2–3 prompts during development; full sweep on
release candidates only.

### Selective sweep

```bash
./run-all.sh prompts/positive       # positive only (cheaper)
./run-all.sh prompts/negative       # negative only (catches over-triggering)
```

## Interpreting failures

A failure tells you one of three things:

1. **The trigger vocabulary in mode-contract.md is wrong** (positive prompt
   didn't trigger, or negative prompt did).
2. **The SKILL.md description is misaligned with mode-contract** (the canon
   says "trigger here" but description gates only on different keywords).
3. **The Iron Law is being violated under live conditions** (skill triggered
   correctly but the final response has bare counts without `[grounded]`).

Each cause has a different fix. The verifier output names which check failed
so you don't have to guess.

## Known intentional surfaces

- `prompts/negative/guard-comment-typo.txt` uses 고쳐줘 (a §3.1 trigger verb).
  mode-contract.md §2.2 says comment-only edits should NOT trigger pre-write;
  the verb-only dispatcher cannot tell. If this currently triggers, that's
  the harness exposing a real spec-vs-implementation gap. Document the choice
  (improve dispatcher OR amend §2.2) — don't silence the test.

## Adapting to your repo

Edit `expectations.json` if your script paths differ. The verifier resolves
`expected_script` against the bash invocations Claude makes, so as long as
the script name matches it doesn't matter where it lives.
