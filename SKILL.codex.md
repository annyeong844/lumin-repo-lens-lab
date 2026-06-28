---
name: lumin-repo-lens-lab-codex
description: "Codex-native lumin-repo-lens-lab for TS/JS repo structural review: find dead code, cycles, duplicate helpers/types, oversized modules, barrel fan-out, plan refactors, or answer 'does X exist anywhere?' with machine evidence."
---

# Lumin Repo Lens Codex

## Purpose

Use this Codex-native wrapper for TS/JS repository structure review,
audit artifact interpretation, and refactor planning. It is a thin
front door over the shared `lumin-repo-lens-lab` engine; it does not duplicate
runtime code.

Prefer this wrapper in Codex when the user asks things like:

- "audit this repo"
- "what should I clean up next?"
- "is this helper/type/file already present?"
- "run lumin-repo-lens-lab full"
- "do a structural review"

## Shared Engine

The shared engine must be installed as the sibling skill
`lumin-repo-lens-lab`. Set `<audit-repo>` to this path resolved relative to this
`SKILL.md`:

```text
../lumin-repo-lens-lab/scripts/audit-repo.mjs
```

If that sibling path is missing, tell the user to link the generated
`skills/lumin-repo-lens-lab` directory into Codex as well. Do not copy the
engine into this wrapper.

## Core Contract

```
NO STRUCTURAL CLAIM WITHOUT MACHINE EVIDENCE
NO ABSENCE CLAIM WITHOUT STATED SCAN RANGE
NO STRUCTURAL REVIEW WITHOUT A CHECKLIST GATE
```

Run the engine, read the artifacts, then make the claim. The short chat
answer is only output density; it never permits skipping checklist
triage.

## Codex Workflow

For the first pass on a repo, stale artifacts, explicit audit/review,
large refactor planning, due diligence, or post-refactor review, run:

```bash
node <audit-repo> --root . --output .audit --profile full
```

For a small follow-up with a fresh full baseline, run:

```bash
node <audit-repo> --root . --output .audit --profile quick
```

Then read `manifest.json`, `checklist-facts.json`, `fix-plan.json`, and
the relevant raw artifacts under `.audit/`. Treat
`audit-summary.latest.md` as an artifact map, not the final answer.

Before any structural review answer, pass the checklist gate: triage
C/D/E/A/B/F using `checklist-facts.json` plus relevant raw artifacts.
For full, deep, exhaustive, due-diligence, CI, or formal review, also
open the sibling engine template at:

```text
../lumin-repo-lens-lab/templates/REVIEW_CHECKLIST.md
```

For normal chat, keep the answer short and humane: what is stable, at
most three things worth smoothing next, what to keep as-is, and the scan
range/confidence. If a full profile ran but the user did not ask for a
full report, end with one required feature-discovery tail. This is not
decoration: it tells the user that the same evidence can be expanded
into a full checklist walk, formal report, or due-diligence handoff.
Give the user copyable phrases, for example: "full checklist로
펼쳐줘", "formal report로 써줘", or "due-diligence handoff로 정리해줘".
Do not omit it after full-profile short answers.

For a saved formal report, the final author re-reads the report before
final answer or handoff and manually checks headline counts, same-site
classifications, broad conclusions, and chat-persona leakage against the
cited artifacts or source. Do not replace this with a string heuristic.

## Code-Change Requests

For add/edit/move/rename work, use the sibling `lumin-repo-lens-lab-write-gate`
skill when available. It owns pre-write reuse screening and post-write
delta checks. If the sibling skill is not installed, run the same engine
with `--pre-write` or `--post-write` and explain the missing sibling
surface.

## Canon Work

For canonical draft or drift checks, use the sibling `lumin-repo-lens-lab-canon`
skill when available. This wrapper stays focused on Codex audit and
review flow.
