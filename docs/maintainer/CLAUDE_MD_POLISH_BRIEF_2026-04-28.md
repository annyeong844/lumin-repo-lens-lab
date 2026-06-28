# Claude Markdown Polish Brief

Use this note when asking Claude Code to polish the Markdown surface for
`lumin-repo-lens-lab`. This is a guardrail document, not a public
skill reference. It is maintainer-only and should not be included in the
generated skill package.

## Goal

Polish wording and information architecture without changing the engine
contract.

The public voice should feel like:

- kind, plain, short, and action-first
- written for a vibe-coder who wants to know what to do next
- backed by machine evidence, but not drowning the user in artifact paths
- honest about uncertainty without sounding scolding

Do not make the tool sound more capable than the artifacts prove.

## Core Working Principles

1. Preserve the Iron Law.
   - Do not weaken "NO STRUCTURAL CLAIM WITHOUT AST EVIDENCE".
   - Do not weaken "NO ABSENCE CLAIM WITHOUT STATED SCAN RANGE".
2. Preserve one public voice.
   - Default chat-facing docs should be friendly and non-jargony.
   - Raw JSON paths, FP ids, tiers, HCA, P-phase terms, and canonical
     jargon stay in reserve unless the user asks for proof/debug/CI/formal
     detail.
3. Preserve progressive disclosure.
   - `SKILL.md` should route, not teach everything.
   - Detailed instructions belong in `references/`.
   - Output shapes belong in `templates/`.
   - Maintainer-only notes belong in `docs/maintainer/`.
4. Preserve the local-only execution contract.
   - The engine writes JSON/Markdown artifacts.
   - The engine does not call external models or APIs.
   - Claude Code may decide inside the same session whether to read a
     review lane locally or paste that whole lane into a built-in reviewer
     subagent.
5. Preserve artifact names and CLI contracts exactly unless explicitly
   asked to change code and tests.

## Allowed Editing Scope

Prefer source Markdown, not generated copies:

- `SKILL.md`
- `README.md`
- `references/*.md`
- `templates/*.md`
- `commands/*.md`
- `docs/maintainer/*.md`
- `docs/product-surface.md`
- `docs/README.md`

If you edit source files that are included in the generated skill
package, run `npm run build:skill` afterward so
`skills/lumin-repo-lens-lab/` is regenerated.

Generated copies under `skills/lumin-repo-lens-lab/` should normally
not be edited by hand.

## Do Not Touch Without Explicit Approval

- `_lib/*.mjs`, root `*.mjs`, and `scripts/*.mjs`
- `package.json`, `package-lock.json`
- `.claude-plugin/*.json`
- `canonical/*.md`
- `canonical-draft/`
- `tests/fixtures/`
- `test-harness/prompts/` and `test-harness/expectations.json`
- `docs/history/` and `docs/spec/`
- `false-positive-patterns.md` content, unless doing targeted wording
  around loading guidance

Exception: if a Markdown edit requires a test update, propose the test
change separately and explain why.

## Things That Must Not Break

Keep these exact ideas intact:

- Default command with no args runs a quick current-workspace audit; it
  must not ask the user to choose a mode.
- `pre-write` may infer compact intent from natural language; ordinary
  users should not be asked to hand-author intent JSON.
- `refactor-plan` is a coaching mode, not an engine mode; it has no
  `audit-repo.mjs --refactor-plan` flag.
- `audit-summary.latest.md` is an artifact map, not a ranked
  recommendation list; raw artifacts remain authoritative for proof and
  prioritization.
- Full/CI profiles may write `audit-review-pack.latest.md`; that pack is
  a local reminder surface and never calls external APIs/models.
- `SAFE_FIX`, `REVIEW_FIX`, `DEGRADED`, and `MUTED` are dead-export tiers,
  not generic architecture verdicts.
- `[확인 불가]` / unknown is a valid answer when evidence is missing.

Do not rename these artifacts or flags:

- `manifest.json`
- `audit-summary.latest.md`
- `audit-review-pack.latest.md`
- `pre-write-advisory.latest.json`
- `post-write-delta.latest.json`
- `--pre-write`
- `--post-write`
- `--canon-draft`
- `--check-canon`
- `--profile quick|full|ci`

## Preferred Polish Targets

Good polish work:

- Shorten `SKILL.md` by moving detail to already-linked references.
- Make first-use paths more obvious in `README.md`.
- Replace insider wording such as "blessed" with "recommended" or
  "canonical" where appropriate.
- Clarify template selection:
  - normal chat -> `templates/REVIEW_CHECKLIST_SHORT.md`
  - formal report -> `templates/report-template.md`
  - coaching plan -> `templates/refactor-plan-template.md`
- Add one-line glossary pointers before dense abbreviations.
- Make Korean/English user-facing wording consistent and friendly.
- Keep "Ask the coding agent:" prompts copy/pasteable.

Bad polish work:

- Turning careful uncertainty into confident claims.
- Removing scan-range or artifact-proof requirements.
- Moving maintainer-only detail into public skill docs.
- Adding long marketing copy to `SKILL.md`.
- Changing command behavior through prose without changing code/tests.
- Asking the user to inspect JSON as the default path.

## Review Checklist For The Polish Pass

Before editing:

1. Read `SKILL.md`.
2. Read `references/command-routing.md`.
3. Read the specific file you intend to polish.
4. Name the target user for that file:
   - first-time vibe-coder
   - returning user
   - maintainer/debug user
   - CI/formal report user

After editing, check:

1. Does the first screen tell the user what to do next?
2. Did any raw tier/FP/canonical jargon leak into a vibe-coder default
   surface?
3. Did any exact command, artifact path, or mode contract change?
4. Did any claim become broader than the engine can prove?
5. Did the generated skill package need `npm run build:skill`?

Run at minimum:

```bash
node tests/test-skill-surface.mjs
npm run check:doc-script-refs
npm run check:skill-triggering
```

If generated package files changed, also run:

```bash
npm run build:skill
npm run check:drift
claude plugin validate .
```

For a release-bound polish pass, run:

```bash
npm run ci
```

## Suggested Prompt To Give Claude Code

```text
Please polish the Markdown surface of this repository without changing the
engine contract.

Read these first:
- docs/maintainer/CLAUDE_MD_POLISH_BRIEF_2026-04-28.md
- SKILL.md
- references/command-routing.md

Scope:
- You may edit SKILL.md, README.md, references/*.md, templates/*.md,
  commands/*.md, and docs/maintainer/*.md.
- Do not edit engine code, package metadata, canonical files, generated
  canonical drafts, fixtures, or tests unless you first explain why.
- Do not edit generated files under skills/lumin-repo-lens-lab/ by
  hand; if source docs change, run npm run build:skill.

Goal:
- Make the public Markdown easier for a vibe-coder to understand.
- Keep the public voice kind, plain, short, and action-first.
- Preserve exact artifact names, CLI flags, evidence rules, and scan-range
  honesty.
- Keep raw JSON paths, FP ids, tiers, HCA/P-phase terms, and canonical
  jargon out of default chat-facing prose unless proof/debug/formal detail
  is explicitly requested.

Please propose a small patch first. Prioritize:
1. SKILL.md concision and first-time routing clarity.
2. README first-run clarity.
3. command-routing readability without changing behavior.
4. template wording consistency.

After the patch, run:
- node tests/test-skill-surface.mjs
- npm run check:doc-script-refs
- npm run check:skill-triggering

If you changed source docs included in the generated skill package, also
run npm run build:skill and npm run check:drift.
```

## Handoff Note

This repo currently treats `lumin-repo-lens-lab` as a vibe-coder
facing skill with maintainer-grade evidence behind the scenes. The polish
pass should not split those into two voices again. The default voice is
gentle and useful; the cold evidence remains available when proof is
needed.
