# Rust Unused Deps Producer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move `unused-deps.json` artifact construction into `lumin-audit-core` while keeping the public JS producer script as a thin compatibility wrapper.

**Architecture:** Rust owns the typed request, artifact construction, package-scope matching, script-tool evidence, and review-only classification. JS keeps repo-mode/package discovery and file writing for this slice, then delegates classification to `lumin-audit-core unused-deps-artifact --result-output`. Manifest summary code continues to consume the same `unused-deps.v1` artifact shape.

**Tech Stack:** Rust `lumin-audit-core`, `serde`, `serde_json`, existing `_lib/audit-core.mjs` result-file bridge, existing Node compatibility test runner.

---

### Task 1: Register The Rust Owner Boundary

**Files:**
- Modify: `canonical/audit-core.md`
- Modify: `experiments/rust-main/lumin-audit-core/src/lib.rs`
- Modify: `experiments/rust-main/lumin-audit-core/src/cli/mod.rs`
- Modify: `experiments/rust-main/lumin-audit-core/src/cli/usage.rs`
- Create: `experiments/rust-main/lumin-audit-core/src/cli/unused_deps.rs`
- Test: `experiments/rust-main/lumin-audit-core/tests/orchestration_plan.rs`

- [ ] **Step 1: Add canonical ownership**

In `canonical/audit-core.md`, add a row to the canonical module table:

```markdown
| `experiments/rust-main/lumin-audit-core/src/unused_deps.rs` | `unused-deps.json` artifact construction from JS-supplied package records and already-produced `symbols.json`: package identity normalization, package-script tool evidence, package-scope consumer matching, review-only dependency classification, deterministic summary projection | JS/TS symbol graph production, repo-mode/package discovery, package manager execution, manifest summary rendering |
```

Also update the scope paragraph to include:

```markdown
review-only `unused-deps.json` dependency hygiene artifact construction
```

- [ ] **Step 2: Register Rust modules**

In `experiments/rust-main/lumin-audit-core/src/lib.rs`, add:

```rust
pub mod unused_deps;
```

In `experiments/rust-main/lumin-audit-core/src/cli/mod.rs`, add the module and import:

```rust
mod unused_deps;

use unused_deps::*;
```

Then add dispatch:

```rust
Some("unused-deps-artifact") => run_unused_deps_artifact(args.collect()),
```

- [ ] **Step 3: Add usage text**

In `experiments/rust-main/lumin-audit-core/src/cli/usage.rs`, add one command line:

```text
       lumin-audit-core unused-deps-artifact --input <path|-> [--result-output <path>]
```

- [ ] **Step 4: Create CLI module shell**

Create `experiments/rust-main/lumin-audit-core/src/cli/unused_deps.rs`:

```rust
use anyhow::{bail, Context, Result};
use std::path::PathBuf;

use super::io_support::{read_json_input, take_path, take_string, write_json_file, write_stdout_json};
use super::usage::USAGE;
use lumin_audit_core::unused_deps::{
    build_unused_deps_artifact, UnusedDepsProducerRequest,
};

pub(super) fn run_unused_deps_artifact(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut result_output: Option<PathBuf> = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            "--result-output" => result_output = Some(take_path(&mut args, "--result-output")?),
            _ => bail!("unused-deps-artifact: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("unused-deps-artifact: missing --input <path|->")?;
    let json = read_json_input(&input, "unused-deps-artifact")?;
    let request = serde_json::from_value::<UnusedDepsProducerRequest>(json)
        .context("unused-deps-artifact: invalid request shape")?;
    let artifact = build_unused_deps_artifact(request)?;
    if let Some(path) = result_output {
        write_json_file(&path, &artifact)
    } else {
        write_stdout_json(&artifact)
    }
}
```

Expected after this step: compilation fails only because `unused_deps` types do
not exist yet.

- [ ] **Step 5: Update orchestration owner expectation**

In `experiments/rust-main/lumin-audit-core/src/orchestration_plan.rs`, later
Task 5 will flip `build-unused-deps.mjs` to `ProducerOwner::Rust`. Prepare the
test expectation in `tests/orchestration_plan.rs` after the implementation
exists:

```rust
assert!(plan
    .steps
    .iter()
    .any(|step| step.step == "build-unused-deps.mjs"
        && step.producer_owner.as_str() == "rust"));
```

Do not expose `ProducerOwner::as_str` publicly only for this assertion. Prefer
serialized JSON equality if visibility stays `pub(crate)`.

### Task 2: Implement Typed Rust Artifact Construction

**Files:**
- Create: `experiments/rust-main/lumin-audit-core/src/unused_deps.rs`
- Test: `experiments/rust-main/lumin-audit-core/tests/unused_deps.rs`

- [ ] **Step 1: Add Rust test file with product behavior cases**

Create `experiments/rust-main/lumin-audit-core/tests/unused_deps.rs`:

```rust
use anyhow::Result;
use serde_json::{json, Value};

use lumin_audit_core::unused_deps::{
    build_unused_deps_artifact, package_name_from_specifier, script_tool_evidence,
    UnusedDepsProducerRequest,
};

fn request(package_records: Value, symbols: Value) -> UnusedDepsProducerRequest {
    serde_json::from_value(json!({
        "schemaVersion": "lumin-unused-deps-producer-request.v1",
        "root": "C:/repo",
        "includeTests": true,
        "exclude": [],
        "packageRecords": package_records,
        "symbols": symbols
    }))
    .expect("fixture request should deserialize")
}

#[test]
fn normalizes_package_specifiers_like_js_producer() {
    assert_eq!(package_name_from_specifier("react"), Some("react".to_string()));
    assert_eq!(
        package_name_from_specifier("react/jsx-runtime"),
        Some("react".to_string())
    );
    assert_eq!(
        package_name_from_specifier("@scope/pkg/sub/path"),
        Some("@scope/pkg".to_string())
    );
    assert_eq!(package_name_from_specifier("node:fs"), None);
    assert_eq!(package_name_from_specifier("./local"), None);
    assert_eq!(package_name_from_specifier("../local"), None);
    assert_eq!(package_name_from_specifier("/abs/local"), None);
    assert_eq!(package_name_from_specifier("C:/abs/local"), None);
    assert_eq!(package_name_from_specifier("https://cdn.example/pkg.js"), None);
    assert_eq!(package_name_from_specifier("data:text/javascript,export{}"), None);
    assert_eq!(package_name_from_specifier("#internal"), None);
    assert_eq!(package_name_from_specifier("virtual:foo"), None);
    assert_eq!(package_name_from_specifier("@broken"), None);
    assert_eq!(package_name_from_specifier(""), None);
}

#[test]
fn extracts_direct_package_script_tool_evidence_without_following_wrappers() -> Result<()> {
    let record = serde_json::from_value(json!({
        "root": "C:/repo",
        "relRoot": ".",
        "packageJson": {
            "scripts": {
                "start": "tsx src/server.ts",
                "dev": "vite --host 0.0.0.0",
                "lint": "pnpm eslint .",
                "bunvite": "bunx vite build",
                "npxlint": "npx eslint .",
                "npmexec": "npm exec eslint .",
                "npmstart": "npm start",
                "npmtest": "npm test",
                "wrapped": "npm run start"
            }
        }
    }))?;
    let evidence = script_tool_evidence(&record);
    let keys: Vec<String> = evidence
        .iter()
        .map(|entry| format!("{}:{}", entry.tool, entry.script_name))
        .collect();

    assert_eq!(
        keys,
        vec![
            "eslint:lint",
            "eslint:npmexec",
            "eslint:npxlint",
            "tsx:start",
            "vite:bunvite",
            "vite:dev"
        ]
    );
    Ok(())
}

#[test]
fn classifies_used_muted_and_review_unused_dependencies() -> Result<()> {
    let artifact = build_unused_deps_artifact(request(
        json!([{
            "root": "C:/repo",
            "relRoot": ".",
            "packageJson": {
                "name": "app",
                "scripts": { "start": "tsx src/server.ts" },
                "dependencies": { "react": "^19.0.0", "left-pad": "^1.3.0" },
                "devDependencies": { "tsx": "^4.0.0", "@types/node": "^22.0.0" },
                "peerDependencies": { "@storybook/react": "^8.0.0" },
                "optionalDependencies": { "fsevents": "^2.3.0" }
            }
        }]),
        json!({
            "meta": { "supports": { "dependencyImportConsumers": true } },
            "dependencyImportConsumers": [
                {
                    "file": "src/app.tsx",
                    "fromSpec": "react/jsx-runtime",
                    "depRoot": "react",
                    "kind": "import",
                    "source": "source-import"
                }
            ]
        }),
    ))?;
    let value = serde_json::to_value(artifact)?;

    assert_eq!(value["schemaVersion"], "unused-deps.v1");
    assert_eq!(value["policyVersion"], "unused-deps-review-policy-v1");
    assert_eq!(value["status"], "complete");
    assert_eq!(value["summary"]["declaredDependencyCount"], 6);
    assert_eq!(value["summary"]["usedCount"], 1);
    assert_eq!(value["summary"]["mutedCount"], 4);
    assert_eq!(value["summary"]["reviewUnusedCount"], 1);
    assert_eq!(dep(&value, "react")["status"], "used");
    assert_eq!(dep(&value, "left-pad")["status"], "review-unused");
    assert_eq!(dep(&value, "tsx")["reason"], "package-script-tool");
    assert_eq!(dep(&value, "@types/node")["reason"], "ambient-types");
    assert_eq!(dep(&value, "@storybook/react")["reason"], "peer-contract");
    assert_eq!(dep(&value, "fsevents")["reason"], "optional-runtime");
    Ok(())
}

#[test]
fn keeps_workspace_package_scopes_separate() -> Result<()> {
    let artifact = build_unused_deps_artifact(request(
        json!([
            {
                "root": "C:/repo",
                "relRoot": ".",
                "packageJson": {
                    "name": "root-app",
                    "dependencies": {
                        "react": "^19.0.0",
                        "@repo/shared": "workspace:*"
                    }
                }
            },
            {
                "root": "C:/repo/packages/app",
                "relRoot": "packages/app",
                "packageJson": {
                    "name": "@repo/app",
                    "dependencies": {
                        "react": "^19.0.0",
                        "@repo/shared": "workspace:*"
                    }
                }
            },
            {
                "root": "C:/repo/packages/shared",
                "relRoot": "packages/shared",
                "packageJson": { "name": "@repo/shared" }
            }
        ]),
        json!({
            "meta": { "supports": { "dependencyImportConsumers": true } },
            "dependencyImportConsumers": [
                {
                    "file": "packages/app/src/App.tsx",
                    "fromSpec": "react",
                    "depRoot": "react",
                    "kind": "import",
                    "source": "source-import"
                }
            ]
        }),
    ))?;
    let value = serde_json::to_value(artifact)?;

    assert_eq!(pkg_dep(&value, ".", "react")["status"], "review-unused");
    assert_eq!(pkg_dep(&value, "packages/app", "react")["status"], "used");
    assert_eq!(
        pkg_dep(&value, "packages/app", "@repo/shared")["reason"],
        "workspace-internal"
    );
    Ok(())
}

#[test]
fn unavailable_when_dependency_import_consumer_support_is_missing() -> Result<()> {
    let artifact = build_unused_deps_artifact(request(
        json!([{
            "root": "C:/repo",
            "relRoot": ".",
            "packageJson": {
                "name": "app",
                "dependencies": { "react": "^19.0.0" }
            }
        }]),
        json!({
            "meta": { "supports": {} },
            "dependencyImportConsumers": []
        }),
    ))?;
    let value = serde_json::to_value(artifact)?;

    assert_eq!(value["status"], "unavailable");
    assert_eq!(value["reason"], "input-artifact-missing");
    assert_eq!(value["summary"]["declaredDependencyCount"], 0);
    assert_eq!(value["packages"], json!([]));
    Ok(())
}

fn dep<'a>(artifact: &'a Value, name: &str) -> &'a Value {
    pkg_dep(artifact, ".", name)
}

fn pkg_dep<'a>(artifact: &'a Value, package_dir: &str, name: &str) -> &'a Value {
    artifact["packages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|package| package["packageDir"] == package_dir)
        .unwrap()["dependencies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|dependency| dependency["name"] == name)
        .unwrap()
}
```

- [ ] **Step 2: Implement public constants and request types**

In `unused_deps.rs`, define these constants and request models:

```rust
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};

pub const UNUSED_DEPS_SCHEMA_VERSION: &str = "unused-deps.v1";
pub const UNUSED_DEPS_POLICY_VERSION: &str = "unused-deps-review-policy-v1";
pub const UNUSED_DEPS_REQUEST_SCHEMA_VERSION: &str =
    "lumin-unused-deps-producer-request.v1";

const DEP_FIELDS: &[&str] = &[
    "dependencies",
    "devDependencies",
    "peerDependencies",
    "optionalDependencies",
];

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnusedDepsProducerRequest {
    pub schema_version: String,
    pub root: String,
    #[serde(default = "default_true")]
    pub include_tests: bool,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub package_records: Vec<PackageRecord>,
    pub symbols: Value,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageRecord {
    pub root: String,
    #[serde(default = "default_package_rel_root")]
    pub rel_root: String,
    #[serde(default)]
    pub package_json: Value,
}

fn default_true() -> bool {
    true
}

fn default_package_rel_root() -> String {
    ".".to_string()
}
```

The builder must reject unsupported request schema:

```rust
if request.schema_version != UNUSED_DEPS_REQUEST_SCHEMA_VERSION {
    bail!(
        "unused-deps-artifact: unsupported schemaVersion '{}'",
        request.schema_version
    );
}
```

- [ ] **Step 3: Implement artifact structs**

Add serializable structs with `camelCase` fields:

```rust
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnusedDepsArtifact {
    pub schema_version: &'static str,
    pub policy_version: &'static str,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub root: String,
    pub scan_range: ScanRange,
    pub inputs: InputSummary,
    pub summary: UnusedDepsSummary,
    pub packages: Vec<PackageDependencyReport>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanRange {
    pub root: String,
    pub include_tests: bool,
    pub exclude: Vec<String>,
    pub source: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InputSummary {
    pub symbols: SymbolInputSummary,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SymbolInputSummary {
    pub artifact: &'static str,
    pub supports_dependency_import_consumers: bool,
    pub scan_range_source: String,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnusedDepsSummary {
    pub package_count: usize,
    pub declared_dependency_count: usize,
    pub used_count: usize,
    pub muted_count: usize,
    pub review_unused_count: usize,
    pub confidence_limited_count: usize,
    pub unavailable_count: usize,
    pub by_reason: BTreeMap<String, usize>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageDependencyReport {
    pub package_dir: String,
    pub package_name: Option<String>,
    pub manifest_path: String,
    pub status: &'static str,
    pub dependencies: Vec<DependencyReport>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyReport {
    pub name: String,
    pub field: String,
    pub range: String,
    pub status: String,
    pub reason: String,
    pub confidence: String,
    pub observed_import_count: usize,
    pub evidence: Vec<Value>,
}
```

- [ ] **Step 4: Implement normalization and script evidence helpers**

Implement `package_name_from_specifier`, command tokenization, and
`script_tool_evidence` to match the JS behavior. Keep helpers small and local
to `unused_deps.rs`; do not create a generic command parser module.

Required signatures:

```rust
pub fn package_name_from_specifier(specifier: &str) -> Option<String>;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScriptToolEvidence {
    pub kind: &'static str,
    pub package_dir: String,
    pub script_name: String,
    pub tool: String,
    pub command: String,
}

pub fn script_tool_evidence(package_record: &PackageRecord) -> Vec<ScriptToolEvidence>;
```

Sort script evidence by `packageDir|tool|scriptName`, matching JS.

- [ ] **Step 5: Implement package scope and classification**

Implement:

```rust
pub fn build_unused_deps_artifact(
    request: UnusedDepsProducerRequest,
) -> Result<UnusedDepsArtifact>;
```

Use these internal functions:

```rust
fn supports_dependency_import_consumers(symbols: &Value) -> bool;
fn collect_declarations(package_record: &PackageRecord) -> Vec<Declaration>;
fn file_belongs_to_package(
    package_rel_root: &str,
    consumer_file: &str,
    all_package_rel_roots: &[String],
) -> bool;
fn classify_dependency(...) -> DependencyReport;
```

Ordering contract:

- packages sort by `packageDir`;
- declarations sort by dependency name, then `DEP_FIELDS` rank;
- consumer evidence sorts by `file|fromSpec|kind`;
- `summary.byReason` uses `BTreeMap`.

- [ ] **Step 6: Run the Rust producer tests**

Run:

```powershell
cargo test --manifest-path experiments/Cargo.toml -p lumin-audit-core --locked --profile ci-test unused_deps
```

Expected: all `unused_deps` tests pass.

### Task 3: Add CLI Result-File Coverage

**Files:**
- Modify: `experiments/rust-main/lumin-audit-core/tests/unused_deps.rs`
- Modify: `_lib/audit-core.mjs`

- [ ] **Step 1: Add CLI test for stdout and result-output**

Append this test to `tests/unused_deps.rs`:

```rust
use std::fs;
use std::process::Command;

#[test]
fn cli_unused_deps_artifact_writes_result_file() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input = temp.path().join("request.json");
    let result = temp.path().join("result.json");
    fs::write(
        &input,
        serde_json::to_vec(&json!({
            "schemaVersion": "lumin-unused-deps-producer-request.v1",
            "root": "C:/repo",
            "includeTests": true,
            "exclude": [],
            "packageRecords": [{
                "root": "C:/repo",
                "relRoot": ".",
                "packageJson": {
                    "name": "app",
                    "dependencies": { "left-pad": "^1.3.0" }
                }
            }],
            "symbols": {
                "meta": { "supports": { "dependencyImportConsumers": true } },
                "dependencyImportConsumers": []
            }
        }))?,
    )?;

    let output = Command::new(audit_core_bin())
        .arg("unused-deps-artifact")
        .arg("--input")
        .arg(&input)
        .arg("--result-output")
        .arg(&result)
        .output()?;

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    let artifact: Value = serde_json::from_slice(&fs::read(&result)?)?;
    assert_eq!(artifact["schemaVersion"], "unused-deps.v1");
    assert_eq!(artifact["summary"]["reviewUnusedCount"], 1);
    Ok(())
}

fn audit_core_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lumin-audit-core")
}
```

If `audit_core_bin` already exists in the file from another test helper, reuse
the existing function rather than defining a second one.

- [ ] **Step 2: Add contract probe for the new command**

In `_lib/audit-core.mjs`, add a missing-input probe:

```js
[
  ['unused-deps-artifact'],
  'unused-deps-artifact: missing --input <path|->',
],
```

Do not add `unused-deps-artifact` to `RESULT_FILE_REQUIRED_SUBCOMMANDS`; the
command can still print small fixtures to stdout for direct CLI debugging.

- [ ] **Step 3: Run contract checks**

Run:

```powershell
node --check .\_lib\audit-core.mjs
cargo test --manifest-path experiments/Cargo.toml -p lumin-audit-core --locked --profile ci-test unused_deps
```

Expected: syntax check succeeds and Rust tests pass.

### Task 4: Convert The JS Producer To A Thin Wrapper

**Files:**
- Modify: `build-unused-deps.mjs`
- Modify: `_lib/unused-deps-artifact.mjs`
- Modify: `tests/test-unused-deps-producer.mjs`
- Modify: `tests/unused-deps-producer.test.mjs`

- [ ] **Step 1: Update JS producer to call Rust**

Replace the direct import in `build-unused-deps.mjs`:

```js
import { buildUnusedDepsArtifact } from './_lib/unused-deps-artifact.mjs';
```

with:

```js
import { runAuditCoreJsonResultFile } from './_lib/audit-core.mjs';
```

Then replace the artifact construction block with:

```js
const request = {
  schemaVersion: 'lumin-unused-deps-producer-request.v1',
  root: ROOT,
  includeTests: cli.includeTests,
  exclude: cli.exclude,
  packageRecords: packageRecordsFromRepoMode(ROOT, repoMode),
  symbols,
};

const artifact = runAuditCoreJsonResultFile(
  ['unused-deps-artifact', '--input', '-'],
  'build-unused-deps',
  { input: JSON.stringify(request) }
);
```

Keep the existing `atomicWrite(...)` and console summary lines unchanged.

- [ ] **Step 2: Retire JS classifier exports without breaking imports silently**

Replace `_lib/unused-deps-artifact.mjs` with a compatibility bridge that does
not reimplement classification:

```js
import { runAuditCoreJsonResultFile } from './audit-core.mjs';

export const UNUSED_DEPS_SCHEMA_VERSION = 'unused-deps.v1';
export const UNUSED_DEPS_POLICY_VERSION = 'unused-deps-review-policy-v1';
export const UNUSED_DEPS_REQUEST_SCHEMA_VERSION =
  'lumin-unused-deps-producer-request.v1';

export function buildUnusedDepsArtifact(request = {}) {
  return runAuditCoreJsonResultFile(
    ['unused-deps-artifact', '--input', '-'],
    'build-unused-deps-artifact',
    {
      input: JSON.stringify({
        schemaVersion: UNUSED_DEPS_REQUEST_SCHEMA_VERSION,
        root: request.root,
        includeTests: request.includeTests ?? true,
        exclude: request.exclude ?? [],
        packageRecords: request.packageRecords ?? [],
        symbols: request.symbols ?? {},
      }),
    }
  );
}
```

Do not keep `packageNameFromSpecifier` or `collectPackageScriptToolEvidence` in
JS. Their behavior moves to Rust tests.

- [ ] **Step 3: Narrow the Node and Vitest compatibility tests**

In `tests/test-unused-deps-producer.mjs`, remove imports of
`packageNameFromSpecifier` and `collectPackageScriptToolEvidence`, and delete
the direct helper checks `UD1` and `UD2`. Keep:

- the direct `buildUnusedDepsArtifact(...)` artifact classification checks;
- workspace scope checks;
- unavailable artifact checks;
- `audit-repo` end-to-end emission check.

Update `UD3` label to `UD1`, `UD4` to `UD2`, `UD5` to `UD3`, and `UD6` to
`UD4` only if the file already uses sequential labels. The labels are not
product semantics, so do not churn assertions beyond readability.

Apply the same import and direct-helper-test narrowing to
`tests/unused-deps-producer.test.mjs` so the named Vitest reference script does
not keep importing JS helper functions that Rust now owns. Do not run the
Vitest mirror as an authoritative gate unless explicitly requested.

- [ ] **Step 4: Run the Node wrapper test**

Run:

```powershell
node tests/test-unused-deps-producer.mjs
```

Expected: the compatibility test passes and the `audit-repo` fixture still
emits `unused-deps.json`.

### Task 5: Mark The Producer As Rust-Owned In The Plan

**Files:**
- Modify: `experiments/rust-main/lumin-audit-core/src/orchestration_plan.rs`
- Modify: `experiments/rust-main/lumin-audit-core/tests/orchestration_plan.rs`

- [ ] **Step 1: Flip producer owner**

In `push_base_pipeline_steps`, change the `build-unused-deps.mjs` step from:

```rust
ProducerOwner::JsMjs,
```

to:

```rust
ProducerOwner::Rust,
```

Only change that step. Do not move adjacent resolver/entry/dead-export steps.

- [ ] **Step 2: Update tests to prove owner, not step name churn**

In `tests/orchestration_plan.rs`, keep the expected quick step names unchanged.
Add a JSON-based assertion:

```rust
let value = serde_json::to_value(&plan)?;
let unused_deps = value["steps"]
    .as_array()
    .unwrap()
    .iter()
    .find(|step| step["step"] == "build-unused-deps.mjs")
    .unwrap();
assert_eq!(unused_deps["producerOwner"], "rust");
assert_eq!(unused_deps["executionOwner"], "lumin-audit-core");
```

Update the full plan Rust-owned count from `1` to `2` when
`rust_analyzer: true`, because the Rust analyzer step and `unused-deps` step
are both Rust-owned.

- [ ] **Step 3: Run orchestration tests**

Run:

```powershell
cargo test --manifest-path experiments/Cargo.toml -p lumin-audit-core --locked --profile ci-test orchestration_plan
```

Expected: orchestration plan tests pass.

### Task 6: Verify And Commit The Implementation

**Files:**
- Verify all files changed by Tasks 1-5

- [ ] **Step 1: Run focused verification**

Run:

```powershell
cargo test --manifest-path experiments/Cargo.toml -p lumin-audit-core --locked --profile ci-test unused_deps
cargo test --manifest-path experiments/Cargo.toml -p lumin-audit-core --locked --profile ci-test orchestration_plan
node tests/test-unused-deps-producer.mjs
node --check build-unused-deps.mjs
node --check _lib/unused-deps-artifact.mjs
node --check _lib/audit-core.mjs
git diff --check
```

Expected: all commands pass.

- [ ] **Step 2: Run the Cargo audit runtime gate**

Run:

```powershell
npm run test:audit-runtime-gate
```

Expected: `lumin-audit-core` tests pass under the repository's locked CI test
profile.

- [ ] **Step 3: Review changed file list**

Run:

```powershell
git status -sb
git diff --stat
```

Expected changes are limited to:

- `canonical/audit-core.md`
- `build-unused-deps.mjs`
- `_lib/audit-core.mjs`
- `_lib/unused-deps-artifact.mjs`
- `experiments/rust-main/lumin-audit-core/src/lib.rs`
- `experiments/rust-main/lumin-audit-core/src/unused_deps.rs`
- `experiments/rust-main/lumin-audit-core/src/cli/mod.rs`
- `experiments/rust-main/lumin-audit-core/src/cli/usage.rs`
- `experiments/rust-main/lumin-audit-core/src/cli/unused_deps.rs`
- `experiments/rust-main/lumin-audit-core/src/orchestration_plan.rs`
- `experiments/rust-main/lumin-audit-core/tests/unused_deps.rs`
- `experiments/rust-main/lumin-audit-core/tests/orchestration_plan.rs`
- `tests/test-unused-deps-producer.mjs`
- `tests/unused-deps-producer.test.mjs`

- [ ] **Step 4: Commit**

Run:

```powershell
git add canonical/audit-core.md build-unused-deps.mjs _lib/audit-core.mjs _lib/unused-deps-artifact.mjs experiments/rust-main/lumin-audit-core/src/lib.rs experiments/rust-main/lumin-audit-core/src/unused_deps.rs experiments/rust-main/lumin-audit-core/src/cli/mod.rs experiments/rust-main/lumin-audit-core/src/cli/usage.rs experiments/rust-main/lumin-audit-core/src/cli/unused_deps.rs experiments/rust-main/lumin-audit-core/src/orchestration_plan.rs experiments/rust-main/lumin-audit-core/tests/unused_deps.rs experiments/rust-main/lumin-audit-core/tests/orchestration_plan.rs tests/test-unused-deps-producer.mjs tests/unused-deps-producer.test.mjs
git commit -m "Move unused dependency producer to audit core"
```

## Self-Review

- Spec coverage: covered owner boundary, CLI shape, product semantics,
  wrapper strategy, Cargo tests, Node wrapper test, and review-only acceptance.
- Placeholder scan: no `TBD`, `TODO`, or "implement later" steps are present.
- Type consistency: request fields use `schemaVersion`, `includeTests`,
  `packageRecords`, `packageJson`, and `dependencyImportConsumers`, matching
  the approved design and existing JS artifact vocabulary.
