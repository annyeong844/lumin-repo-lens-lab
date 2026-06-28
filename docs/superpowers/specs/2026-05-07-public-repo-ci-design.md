# Public Repo CI For Packaged Lumin Repo Lens

## Goal

Move the expensive, repeatable package-surface verification away from the
private maintainer repository and onto the existing public package repository:
`annyeong844/lumin-repo-lens-lab`.

The private repository remains the development workspace. The public repository
verifies the artifact users actually install.

## Context

The private maintainer repository has limited GitHub Actions minutes. Large
resolver and evidence-engine changes still need local verification before they
are merged, but running full private CI for every draft or small iteration is no
longer sustainable.

The repository already has a public publishing script:
`scripts/publish-public-plugin.mjs`. It syncs only the generated plugin package
surface into `annyeong844/lumin-repo-lens-lab`.

## Design

Add a lightweight public CI workflow to the package surface that is published to
`annyeong844/lumin-repo-lens-lab`.

The public workflow should verify only public-package behavior:

- dependencies install with `npm ci` inside `skills/lumin-repo-lens-lab`;
- packaged smoke test runs;
- packaged CLI entrypoints can start and print help;
- package metadata remains installable and internally consistent.

The public workflow must not depend on maintainer-only files such as
`tests/`, `test-harness/`, `docs/spec/`, private fixtures, or local audit
outputs.

## Publication Flow

The source of truth for the public CI workflow should live in the private
maintainer repository and be synced by `scripts/publish-public-plugin.mjs`
alongside the generated plugin package.

Recommended source path:

```text
public-package/.github/workflows/ci.yml
```

Publishing copies that workflow to:

```text
<public checkout>/.github/workflows/ci.yml
```

This keeps public CI versioned with the packaging script and avoids manual
drift in the public repository.

## Non-Goals

- Do not run the full maintainer `npm run ci` in the public repository.
- Do not publish maintainer-only tests or fixtures to the public repository.
- Do not make public CI responsible for internal evidence-engine correctness.
- Do not add secrets, private paths, or local-user data to the public workflow.
- Do not require server CI for draft private PRs.

## Private Repository Policy

Private PRs may remain draft while under active development, so private GitHub
Actions can stay skipped by the existing workflow policy. Developers should run
focused local checks for the files they changed.

Large changes should still receive local verification before merge. Public CI is
not a substitute for source-level tests; it is a package-surface safety net.

## Public Repository Policy

Public CI should run on:

- `push` to `main`;
- `workflow_dispatch`.

Pull request CI in the public repository is optional. The first implementation
can skip it because the public repo is primarily updated by the publishing
script.

## Validation

Implementation is accepted when:

- `scripts/publish-public-plugin.mjs --dry-run` copies the public workflow into
  a temporary public checkout;
- public package validation still rejects maintainer-only root entries;
- the workflow references only files that exist in the public package;
- local syntax/lint checks pass in the maintainer repository;
- no private CI minutes are required for draft private PR validation.
