# WT-SFC Beta.67 Template Component Target Verification

This note records the beta.67 public-install verification for the WT-SFC
`sfc-template-component-refs` target-evidence follow-up described by
[`sfc-support-policy.md`](../spec/sfc-support-policy.md).

The verification used the installed public package from public main
`e833b8a`, not only source tests. Public Package CI passed for that commit:
[`annyeong844/lumin-repo-lens-lab/actions/runs/26450888954`](https://github.com/annyeong844/lumin-repo-lens-lab/actions/runs/26450888954).
The source guards are
[`test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs).

## Result

PASS. The installed beta.67 package keeps SFC-to-SFC template component
references as muted review evidence while preserving `resolvedFile` for
reviewer navigation. Source-module component bindings still resolve normally,
and template component refs remain outside graph edges, named export fan-in,
deadness, and action lanes.

| #   | Checkpoint                                           | Result | Evidence                                                                                                                |
| --- | ---------------------------------------------------- | ------ | ----------------------------------------------------------------------------------------------------------------------- |
| 1   | Installed version is `0.9.0-beta.67`                 | PASS   | `plugin.json`, `marketplace.json`, skill `package.json`, and skill `package-lock.json` all reported beta.67.            |
| 2   | SFC-to-SFC component refs are retained               | PASS   | Vue, Svelte, and Astro component tags pointing at SFC files appeared in `symbols.sfcTemplateComponentRefs[]`.           |
| 3   | SFC targets stay muted                               | PASS   | SFC targets carried `status: "muted"` and reason `sfc-template-component-non-source-binding`.                           |
| 4   | SFC targets preserve reviewer navigation             | PASS   | `<Card/>`, `<user-list/>`, `<Header/>`, `<Sidebar/>`, `<Footer/>`, and `<Banner/>` carried concrete `resolvedFile`s.    |
| 5   | Source-module component target still resolves        | PASS   | The source-module control `<Thing/>` retained `status: "resolved"` and `resolvedFile: "src/thing.ts"`.                  |
| 6   | Dynamic, namespace, and missing refs remain weak     | PASS   | Those cases stayed muted or unresolved and did not gain `resolvedFile` without a single concrete target.                |
| 7   | Graph, fan-in, and action lanes remain uncontaminated | PASS   | No template component refs entered `resolvedInternalEdges[]`, fan-in, `fix-plan.json`, or `export-action-safety.json`.  |
| 8   | `sfc-scan-gap` remains visible                       | PASS   | The grouped scan-gap blind zone remained visible for the SFC fixture.                                                   |

## Safety Notes

SFC-to-SFC template tags are useful reviewer evidence, but they are not proof
that the target SFC's exports or runtime behavior are statically consumed.
Beta.67 therefore records the target file for navigation while keeping the
claim muted:

```json
{
  "status": "muted",
  "reason": "sfc-template-component-non-source-binding",
  "resolvedFile": "src/Card.vue",
  "eligibleForFanIn": false,
  "eligibleForSafeFix": false
}
```

That distinction is intentional. `resolvedFile` answers "which file does this
explicit binding point at?" It does not answer "which named exports are used?"
and does not authorize deadness, `SAFE_FIX`, `EXISTS`, package edits, or graph
reachability claims.

## Decision

Decision: `sfc-target-file-evidence-without-graph-claim` and
`template-ref-public-verified`.

WT-SFC remains `MVP`, not `DONE`: SFC script imports, script-source
reachability, style assets, and explicit template component bindings now have
grounded review evidence, but framework-specific SFC semantics still require
lane-specific contracts before they affect absence claims.
