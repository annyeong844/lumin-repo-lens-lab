# Lab Staging Area

This directory describes reproducible working surfaces that matter to
maintainers and benchmarking, but are not part of the small public
entrypoint.

For the broader docs map, start at [`docs/README.md`](../README.md).

Typical lab surfaces:

- `output/` — generated audit artifacts from local or CI runs
- `review-output*/` — local self-review, dogfood, and external-corpus
  evidence directories
- `p6-corpus/` — benchmark and dogfood corpora
- `canonical-draft/` — generated draft observations and versioned draft
  outputs
- `audit-artifacts/` and similar local evidence stores
- `audit-artifacts-smoke/` — local smoke-run evidence
- optional local evidence stores such as `.audit/`
- local tool state such as `.claude/`

Current lab notes:

- [M5 Rust topology prefer closeout - 2026-06-16](../../baselines/m5-rust-topology-prefer-closeout-2026-06-16.md)
- [M5 Rust topology prefer design - 2026-06-15](m5-rust-topology-prefer-design-2026-06-15.md)
- [M4 Rust topology quorum evidence design - 2026-06-15](m4-rust-topology-quorum-evidence-design-2026-06-15.md)
- [M3 Rust topology prefer gate design - 2026-06-15](m3-rust-topology-prefer-gate-design-2026-06-15.md)
- [Self-audit health note - 2026-04-26](self-audit-health-2026-04-26.md)
- [Suyeon dead export dogfood note - 2026-04-26](suyeon-dead-export-dogfood-2026-04-26.md)
- [WT-18 artifact read measurement - 2026-05-10](wt18-artifact-read-measurement-2026-05-10.md)
- [WT-18 symbol graph phase measurement - 2026-05-10](wt18-symbol-graph-phase-measurement-2026-05-10.md)
- [WT-18 symbol graph subphase measurement - 2026-05-10](wt18-symbol-graph-subphase-measurement-2026-05-10.md)
- [WT-18 source-use assembly measurement - 2026-05-10](wt18-source-use-assembly-measurement-2026-05-10.md)
- [WT-18 scoped baseUrl probe cache measurement - 2026-05-10](wt18-baseurl-probe-cache-measurement-2026-05-10.md)
- [WT-18 beta.38 public cal.diy verification - 2026-05-10](wt18-beta38-public-cal-diy-verification-2026-05-10.md)
- [WT-18 beta.39 public cal.diy verification - 2026-05-10](wt18-beta39-public-cal-diy-verification-2026-05-10.md)
- [WT-23 P2 service-operation cue readiness - 2026-05-13](wt23-p2-service-operation-cue-readiness-2026-05-13.md)
- [WT-23 beta.50 service-operation Markdown verification - 2026-05-14](wt23-beta50-service-operation-markdown-verification-2026-05-14.md)
- [WT-23 service-operation corpus calibration plan - 2026-05-16](wt23-service-operation-corpus-calibration-plan-2026-05-16.md)
- [WT-23 service-operation corpus calibration - 2026-05-16](wt23-service-operation-corpus-calibration-2026-05-16.md)
- [WT-23 beta.54 local-operation support reason verification - 2026-05-17](wt23-beta54-local-operation-support-reason-verification-2026-05-17.md)
- [WT-23 beta.55 service-operation type-name filter verification - 2026-05-17](wt23-beta55-service-operation-type-name-filter-verification-2026-05-17.md)
- [WT-09 block clone fixture inventory - 2026-05-24](wt09-block-clone-fixture-inventory-2026-05-24.md)
- [WT-09 beta.60 block clone noise policy verification - 2026-05-25](wt09-beta60-block-clone-noise-policy-verification-2026-05-25.md)
- [WT-09 beta.61 block clone cap/noise v2 verification - 2026-05-25](wt09-beta61-block-clone-cap-noise-v2-verification-2026-05-25.md)
- [WT-SFC beta.63 script import consumers verification - 2026-05-25](wt-sfc-beta63-script-import-consumers-verification-2026-05-25.md)
- [WT-SFC script src fixture inventory - 2026-05-25](wt-sfc-script-src-fixture-inventory-2026-05-25.md)
- [WT-SFC beta.64 script src reachability verification - 2026-05-25](wt-sfc-beta64-script-src-reachability-verification-2026-05-25.md)
- [WT-SFC style asset fixture inventory - 2026-05-25](wt-sfc-style-asset-fixture-inventory-2026-05-25.md)
- [WT-SFC beta.65 style asset verification - 2026-05-26](wt-sfc-beta65-style-assets-verification-2026-05-26.md)
- [WT-SFC template component ref fixture inventory - 2026-05-26](wt-sfc-template-component-ref-fixture-inventory-2026-05-26.md)
- [WT-SFC beta.67 template component target verification - 2026-05-26](wt-sfc-beta67-template-component-target-verification-2026-05-26.md)
- [WT-SFC remaining gaps inventory - 2026-05-26](wt-sfc-remaining-gaps-inventory-2026-05-26.md)
- [WT-SFC global component registration fixture inventory - 2026-05-26](wt-sfc-global-component-registration-fixture-inventory-2026-05-26.md)
- [WT-SFC beta.68 global component registration verification - 2026-05-27](wt-sfc-beta68-global-component-registration-verification-2026-05-27.md)
- [WT-SFC global registration P2 fixture inventory - 2026-05-30](wt-sfc-global-registration-p2-fixture-inventory-2026-05-30.md)
- [WT-SFC framework magic fixture inventory - 2026-05-27](wt-sfc-framework-magic-fixture-inventory-2026-05-27.md)
- [WT-SFC beta.71 Nuxt convention verification - 2026-05-28](wt-sfc-beta71-nuxt-convention-verification-2026-05-28.md)
- [WT-SFC beta.72 unplugin config verification - 2026-05-28](wt-sfc-beta72-unplugin-config-verification-2026-05-28.md)
- [WT-SFC Astro client directive fixture inventory - 2026-05-28](wt-sfc-astro-client-directive-fixture-inventory-2026-05-28.md)
- [WT-SFC beta.73 Astro client directive verification - 2026-05-28](wt-sfc-beta73-astro-client-directive-verification-2026-05-28.md)
- [WT-SFC Svelte action directive fixture inventory - 2026-05-29](wt-sfc-svelte-action-directive-fixture-inventory-2026-05-29.md)
- [WT-SFC beta.74 Svelte action directive verification - 2026-05-28](wt-sfc-beta74-svelte-action-directive-verification-2026-05-28.md)
- [WT-SFC beta.75 Vue macro registration verification - 2026-05-28](wt-sfc-beta75-vue-macro-registration-verification-2026-05-28.md)
- [WT-SFC beta.76 Vue Options API registration verification - 2026-05-29](wt-sfc-beta76-vue-options-registration-verification-2026-05-29.md)
- [WT-SFC beta.78 SFC evidence audit brief verification - 2026-05-31](wt-sfc-beta78-sfc-evidence-brief-verification-2026-05-31.md)
- [WT-SFC MVP status and remaining gaps - 2026-05-31](wt-sfc-mvp-status-and-remaining-gaps-2026-05-31.md)
- [WT-SFC corpus calibration plan - 2026-05-31](wt-sfc-corpus-calibration-plan-2026-05-31.md)
- [WT-SFC Vite corpus calibration - 2026-05-31](wt-sfc-vite-corpus-calibration-2026-05-31.md)
- [WT-SFC IMA2 Astro corpus calibration - 2026-05-31](wt-sfc-ima2-astro-corpus-calibration-2026-05-31.md)
- [WT-SFC Astro client corpus calibration - 2026-05-31](wt-sfc-astro-client-corpus-calibration-2026-05-31.md)
- [WT-SFC SvelteKit corpus calibration - 2026-05-31](wt-sfc-sveltekit-corpus-calibration-2026-05-31.md)
- [WT-SFC beta.79 SvelteKit local action regression - 2026-05-31](wt-sfc-beta79-sveltekit-local-action-regression-2026-05-31.md)
- [WT-SFC Nuxt app-dir and custom resolver inventory - 2026-05-31](wt-sfc-nuxt-app-dir-custom-resolver-inventory-2026-05-31.md)
- [WT-SFC beta.80 Nuxt app-dir verification - 2026-05-31](wt-sfc-beta80-nuxt-app-dir-verification-2026-05-31.md)
- [WT-SFC beta.81 Nuxt #components alias verification - 2026-05-31](wt-sfc-beta81-nuxt-components-alias-verification-2026-05-31.md)
- [WT-SFC Vue Options corpus calibration - 2026-05-31](wt-sfc-vue-options-corpus-calibration-2026-05-31.md)
- [WT-SFC Storybook Vue corpus calibration - 2026-05-31](wt-sfc-storybook-vue-corpus-calibration-2026-05-31.md)
- [WT-SFC Nuxt main corpus calibration - 2026-06-01](wt-sfc-nuxt-main-corpus-calibration-2026-06-01.md)
- [WT-SFC beta.86 Nuxt alias helper filter verification - 2026-06-02](wt-sfc-beta86-nuxt-alias-helper-filter-verification-2026-06-02.md)

Production-ready default tracking should treat generated draft markdown
under `canonical-draft/*.md`, review output directories, benchmark
corpora, and local tool state as lab surfaces rather than as shipping
assets.

These surfaces are intentionally useful, but they are not the public
capability contract. They should be discovered through maintainer docs,
not through onboarding or the first screen of the repo.

The public contract remains anchored at:

- `SKILL.md`
- `README.md`
- `audit-repo.mjs`
- `canonical/`
