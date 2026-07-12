use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

pub const UNUSED_DEPS_SCHEMA_VERSION: &str = "unused-deps.v1";
pub const UNUSED_DEPS_POLICY_VERSION: &str = "unused-deps-review-policy-v1";
pub const UNUSED_DEPS_REQUEST_SCHEMA_VERSION: &str = "lumin-unused-deps-producer-request.v1";

const DEP_FIELDS: &[&str] = &[
    "dependencies",
    "devDependencies",
    "peerDependencies",
    "optionalDependencies",
];

const PACKAGE_RUNNERS: &[&str] = &["npm", "pnpm", "yarn", "bun"];
const DIRECT_EXEC_RUNNERS: &[&str] = &["bunx", "npx"];
const RUNNER_EXEC_SUBCOMMANDS: &[&str] = &["exec", "x", "dlx"];
const WRAPPER_SUBCOMMANDS: &[&str] = &["run", "run-script"];

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

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScriptToolEvidence {
    pub kind: &'static str,
    pub package_dir: String,
    pub script_name: String,
    pub tool: String,
    pub command: String,
}

#[derive(Debug, Clone)]
struct Declaration {
    name: String,
    field: String,
    range: String,
}

#[derive(Debug, Clone)]
struct ObservedConsumer {
    file: String,
    from_spec: Value,
    kind: String,
    source: String,
    type_only: Option<bool>,
}

fn default_true() -> bool {
    true
}

fn default_package_rel_root() -> String {
    ".".to_string()
}

pub fn build_unused_deps_artifact(
    request: UnusedDepsProducerRequest,
) -> Result<UnusedDepsArtifact> {
    if request.schema_version != UNUSED_DEPS_REQUEST_SCHEMA_VERSION {
        bail!(
            "unused-deps-artifact: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }

    let scan_range = scan_range_from_inputs(&request);
    if !supports_dependency_import_consumers(&request.symbols) {
        return Ok(unavailable_artifact(
            request.root,
            scan_range,
            input_summary(&request.symbols, false),
            "input-artifact-missing",
        ));
    }

    let all_package_rel_roots = request
        .package_records
        .iter()
        .map(|record| record.rel_root.clone())
        .collect::<Vec<_>>();
    let workspace_package_names = request
        .package_records
        .iter()
        .filter_map(package_name)
        .collect::<BTreeSet<_>>();

    let mut packages = request
        .package_records
        .iter()
        .map(|package_record| {
            let script_evidence = script_tool_evidence(package_record);
            let observed_consumers = build_observed_consumer_index(
                &request.root,
                package_record,
                &request.symbols,
                &all_package_rel_roots,
            );
            let current_package_name = package_name(package_record);
            let dependencies = collect_declarations(package_record)
                .into_iter()
                .map(|declaration| {
                    classify_dependency(
                        declaration,
                        current_package_name.as_deref(),
                        &workspace_package_names,
                        &observed_consumers,
                        &script_evidence,
                    )
                })
                .collect::<Vec<_>>();
            PackageDependencyReport {
                package_dir: package_record.rel_root.clone(),
                package_name: current_package_name,
                manifest_path: manifest_path(&package_record.rel_root),
                status: "complete",
                dependencies,
            }
        })
        .collect::<Vec<_>>();
    packages.sort_by(|left, right| left.package_dir.cmp(&right.package_dir));

    let summary = summarize(&packages);
    Ok(UnusedDepsArtifact {
        schema_version: UNUSED_DEPS_SCHEMA_VERSION,
        policy_version: UNUSED_DEPS_POLICY_VERSION,
        status: "complete".to_string(),
        reason: None,
        root: request.root,
        scan_range,
        inputs: input_summary(&request.symbols, true),
        summary,
        packages,
    })
}

pub fn package_name_from_specifier(specifier: &str) -> Option<String> {
    let spec = specifier.trim();
    if spec.is_empty()
        || spec.starts_with('.')
        || spec.starts_with('/')
        || spec.starts_with('\\')
        || spec.starts_with("node:")
        || spec.starts_with('#')
        || looks_like_scheme(spec)
        || looks_like_windows_absolute_path(spec)
    {
        return None;
    }

    if spec.starts_with('@') {
        let mut parts = spec.split('/');
        let scope = parts.next()?;
        let name = parts.next()?;
        if scope.is_empty() || name.is_empty() {
            return None;
        }
        return Some(format!("{scope}/{name}"));
    }
    spec.split('/')
        .next()
        .filter(|part| !part.is_empty())
        .map(str::to_string)
}

pub fn script_tool_evidence(package_record: &PackageRecord) -> Vec<ScriptToolEvidence> {
    let Some(scripts) = package_record
        .package_json
        .get("scripts")
        .and_then(Value::as_object)
    else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (script_name, command) in scripts {
        let Some(command) = command.as_str() else {
            continue;
        };
        let tokens = tokenize_command(command);
        let Some(tool) = script_tool_from_tokens(&tokens) else {
            continue;
        };
        out.push(ScriptToolEvidence {
            kind: "package-script",
            package_dir: package_record.rel_root.clone(),
            script_name: script_name.clone(),
            tool,
            command: command.to_string(),
        });
    }
    out.sort_by(|left, right| {
        format!("{}|{}|{}", left.package_dir, left.tool, left.script_name).cmp(&format!(
            "{}|{}|{}",
            right.package_dir, right.tool, right.script_name
        ))
    });
    out
}

fn looks_like_scheme(value: &str) -> bool {
    let Some((head, _)) = value.split_once(':') else {
        return false;
    };
    let mut chars = head.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_alphabetic()
        && chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '.' | '-'))
}

fn looks_like_windows_absolute_path(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && matches!(bytes[2], b'/' | b'\\')
}

fn slash_path(value: &str) -> String {
    value.replace('\\', "/")
}

fn command_name(token: &str) -> String {
    let name = slash_path(token)
        .rsplit('/')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    for suffix in [".cmd", ".ps1", ".exe"] {
        if let Some(stripped) = name.strip_suffix(suffix) {
            return stripped.to_string();
        }
    }
    name
}

fn tokenize_command(command: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut token = String::new();
    let mut quote = None;
    let mut escaping = false;

    for ch in command.chars() {
        if escaping {
            token.push(ch);
            escaping = false;
            continue;
        }
        if ch == '\\' {
            escaping = true;
            continue;
        }
        if let Some(active_quote) = quote {
            if ch == active_quote {
                quote = None;
            } else {
                token.push(ch);
            }
            continue;
        }
        if ch == '"' || ch == '\'' {
            quote = Some(ch);
            continue;
        }
        if ch.is_whitespace() {
            if !token.is_empty() {
                out.push(std::mem::take(&mut token));
            }
            continue;
        }
        token.push(ch);
    }
    if !token.is_empty() {
        out.push(token);
    }
    out
}

fn script_tool_from_tokens(tokens: &[String]) -> Option<String> {
    let first = command_name(tokens.first()?);
    if first.is_empty() {
        return None;
    }
    if DIRECT_EXEC_RUNNERS.contains(&first.as_str()) {
        return tokens.get(1).map(|token| command_name(token));
    }
    if !PACKAGE_RUNNERS.contains(&first.as_str()) {
        return Some(first);
    }
    let subcommand = command_name(tokens.get(1)?);
    if subcommand.is_empty() || WRAPPER_SUBCOMMANDS.contains(&subcommand.as_str()) {
        return None;
    }
    if RUNNER_EXEC_SUBCOMMANDS.contains(&subcommand.as_str()) {
        return tokens.get(2).map(|token| command_name(token));
    }
    if first == "npm" {
        return None;
    }
    Some(subcommand)
}

fn supports_dependency_import_consumers(symbols: &Value) -> bool {
    symbols
        .pointer("/meta/supports/dependencyImportConsumers")
        .and_then(Value::as_bool)
        == Some(true)
        && symbols
            .get("dependencyImportConsumers")
            .and_then(Value::as_array)
            .is_some()
}

fn scan_range_from_inputs(request: &UnusedDepsProducerRequest) -> ScanRange {
    if let Some(source_range) = request
        .symbols
        .pointer("/meta/scanRange")
        .and_then(Value::as_object)
    {
        return ScanRange {
            root: source_range
                .get("root")
                .and_then(Value::as_str)
                .unwrap_or(&request.root)
                .to_string(),
            include_tests: source_range
                .get("includeTests")
                .and_then(Value::as_bool)
                .unwrap_or(request.include_tests),
            exclude: source_range
                .get("exclude")
                .and_then(Value::as_array)
                .map(|values| {
                    values
                        .iter()
                        .filter_map(Value::as_str)
                        .map(ToOwned::to_owned)
                        .collect()
                })
                .unwrap_or_else(|| request.exclude.clone()),
            source: "symbols.meta.scanRange".to_string(),
        };
    }
    ScanRange {
        root: request.root.clone(),
        include_tests: request.include_tests,
        exclude: request.exclude.clone(),
        source: "producer-cli".to_string(),
    }
}

fn input_summary(symbols: &Value, supported: bool) -> InputSummary {
    let scan_range_source = if symbols.pointer("/meta/scanRange").is_some() {
        "symbols.meta.scanRange"
    } else {
        "producer-cli"
    };
    InputSummary {
        symbols: SymbolInputSummary {
            artifact: "symbols.json",
            supports_dependency_import_consumers: supported,
            scan_range_source: scan_range_source.to_string(),
        },
    }
}

fn unavailable_artifact(
    root: String,
    scan_range: ScanRange,
    inputs: InputSummary,
    reason: &str,
) -> UnusedDepsArtifact {
    UnusedDepsArtifact {
        schema_version: UNUSED_DEPS_SCHEMA_VERSION,
        policy_version: UNUSED_DEPS_POLICY_VERSION,
        status: "unavailable".to_string(),
        reason: Some(reason.to_string()),
        root,
        scan_range,
        inputs,
        summary: UnusedDepsSummary::default(),
        packages: Vec::new(),
    }
}

fn package_name(package_record: &PackageRecord) -> Option<String> {
    package_record
        .package_json
        .get("name")
        .and_then(Value::as_str)
        .filter(|name| !name.is_empty())
        .map(ToOwned::to_owned)
}

fn manifest_path(package_rel_root: &str) -> String {
    if package_rel_root == "." {
        "package.json".to_string()
    } else {
        format!(
            "{}/package.json",
            slash_path(package_rel_root).trim_end_matches('/')
        )
    }
}

fn collect_declarations(package_record: &PackageRecord) -> Vec<Declaration> {
    let mut declarations = Vec::new();
    for field in DEP_FIELDS {
        let Some(entries) = package_record
            .package_json
            .get(*field)
            .and_then(Value::as_object)
        else {
            continue;
        };
        for (name, range) in entries {
            declarations.push(Declaration {
                name: name.clone(),
                field: (*field).to_string(),
                range: range
                    .as_str()
                    .map(ToOwned::to_owned)
                    .unwrap_or_else(|| range.to_string()),
            });
        }
    }
    declarations.sort_by(|left, right| {
        left.name.cmp(&right.name).then_with(|| {
            declaration_field_rank(&left.field).cmp(&declaration_field_rank(&right.field))
        })
    });
    declarations
}

fn declaration_field_rank(field: &str) -> usize {
    DEP_FIELDS
        .iter()
        .position(|candidate| *candidate == field)
        .unwrap_or(DEP_FIELDS.len())
}

fn build_observed_consumer_index(
    root: &str,
    package_record: &PackageRecord,
    symbols: &Value,
    all_package_rel_roots: &[String],
) -> BTreeMap<String, Vec<ObservedConsumer>> {
    let mut by_name: BTreeMap<String, Vec<ObservedConsumer>> = BTreeMap::new();
    let Some(consumers) = symbols
        .get("dependencyImportConsumers")
        .and_then(Value::as_array)
    else {
        return by_name;
    };
    for consumer in consumers {
        let dep_name = consumer
            .get("depRoot")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .or_else(|| {
                consumer
                    .get("fromSpec")
                    .and_then(Value::as_str)
                    .and_then(package_name_from_specifier)
            });
        let Some(dep_name) = dep_name else {
            continue;
        };
        let Some(file) = consumer
            .get("file")
            .and_then(Value::as_str)
            .and_then(|file| normalize_package_file(root, file))
        else {
            continue;
        };
        if !file_belongs_to_package(&package_record.rel_root, &file, all_package_rel_roots) {
            continue;
        }
        by_name.entry(dep_name).or_default().push(ObservedConsumer {
            file,
            from_spec: consumer.get("fromSpec").cloned().unwrap_or(Value::Null),
            kind: consumer
                .get("kind")
                .and_then(Value::as_str)
                .unwrap_or("import")
                .to_string(),
            source: consumer
                .get("source")
                .and_then(Value::as_str)
                .unwrap_or("symbols.json.dependencyImportConsumers")
                .to_string(),
            type_only: consumer.get("typeOnly").and_then(Value::as_bool),
        });
    }
    for entries in by_name.values_mut() {
        entries.sort_by(|left, right| {
            format!(
                "{}|{}|{}",
                left.file,
                value_string(&left.from_spec),
                left.kind
            )
            .cmp(&format!(
                "{}|{}|{}",
                right.file,
                value_string(&right.from_spec),
                right.kind
            ))
        });
    }
    by_name
}

fn normalize_package_file(root: &str, file: &str) -> Option<String> {
    if file.is_empty() {
        return None;
    }
    let normalized = slash_path(file);
    let normalized_root = slash_path(root).trim_end_matches('/').to_string();
    if (normalized.starts_with('/') || looks_like_windows_absolute_path(&normalized))
        && !normalized_root.is_empty()
    {
        if normalized == normalized_root {
            return Some(".".to_string());
        }
        if let Some(stripped) = normalized.strip_prefix(&(normalized_root + "/")) {
            return Some(stripped.to_string());
        }
    }
    Some(normalized.trim_start_matches("./").to_string())
}

fn file_belongs_to_package(
    package_rel_root: &str,
    consumer_file: &str,
    all_package_rel_roots: &[String],
) -> bool {
    let package = package_scope_root(package_rel_root);
    let file = slash_path(consumer_file);
    let child_roots = all_package_rel_roots
        .iter()
        .filter(|root| root.as_str() != package_rel_root)
        .map(|root| package_scope_root(root))
        .filter(|root| !root.is_empty())
        .collect::<Vec<_>>();

    if package.is_empty() {
        return !child_roots
            .iter()
            .any(|child| file == *child || file.starts_with(&format!("{child}/")));
    }
    file == package || file.starts_with(&format!("{package}/"))
}

fn package_scope_root(package_rel_root: &str) -> String {
    if package_rel_root == "." {
        String::new()
    } else {
        slash_path(package_rel_root)
            .trim_end_matches('/')
            .to_string()
    }
}

fn classify_dependency(
    declaration: Declaration,
    package_name: Option<&str>,
    workspace_package_names: &BTreeSet<String>,
    observed_consumers: &BTreeMap<String, Vec<ObservedConsumer>>,
    script_evidence: &[ScriptToolEvidence],
) -> DependencyReport {
    let consumers = observed_consumers
        .get(&declaration.name)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    if !consumers.is_empty() {
        return DependencyReport {
            name: declaration.name,
            field: declaration.field,
            range: declaration.range,
            status: "used".to_string(),
            reason: "external-import-consumer".to_string(),
            confidence: "grounded".to_string(),
            observed_import_count: consumers.len(),
            evidence: consumers.iter().take(10).map(consumer_evidence).collect(),
        };
    }

    let scripts = script_evidence
        .iter()
        .filter(|entry| entry.tool == declaration.name)
        .take(10)
        .map(|entry| serde_json::to_value(entry).unwrap_or(Value::Null))
        .collect::<Vec<_>>();
    if !scripts.is_empty() {
        return classified_report(
            declaration,
            "muted",
            "package-script-tool",
            "grounded",
            scripts,
        );
    }

    if declaration.field == "peerDependencies" {
        return classified_report(declaration, "muted", "peer-contract", "review", Vec::new());
    }
    if declaration.field == "optionalDependencies" {
        return classified_report(
            declaration,
            "muted",
            "optional-runtime",
            "review",
            Vec::new(),
        );
    }
    if declaration.name.starts_with("@types/") {
        return classified_report(declaration, "muted", "ambient-types", "review", Vec::new());
    }
    if workspace_package_names.contains(&declaration.name)
        && Some(declaration.name.as_str()) != package_name
    {
        return classified_report(
            declaration,
            "muted",
            "workspace-internal",
            "review",
            Vec::new(),
        );
    }
    classified_report(
        declaration,
        "review-unused",
        "no-observed-consumer",
        "review",
        Vec::new(),
    )
}

fn classified_report(
    declaration: Declaration,
    status: &str,
    reason: &str,
    confidence: &str,
    evidence: Vec<Value>,
) -> DependencyReport {
    DependencyReport {
        name: declaration.name,
        field: declaration.field,
        range: declaration.range,
        status: status.to_string(),
        reason: reason.to_string(),
        confidence: confidence.to_string(),
        observed_import_count: 0,
        evidence,
    }
}

fn consumer_evidence(consumer: &ObservedConsumer) -> Value {
    let mut evidence = json!({
        "kind": consumer.kind,
        "file": consumer.file,
        "fromSpec": consumer.from_spec,
        "source": consumer.source,
    });
    if let Some(object) = evidence.as_object_mut() {
        if let Some(type_only) = consumer.type_only {
            object.insert("typeOnly".to_string(), Value::Bool(type_only));
        }
    }
    evidence
}

fn summarize(packages: &[PackageDependencyReport]) -> UnusedDepsSummary {
    let mut summary = UnusedDepsSummary {
        package_count: packages.len(),
        ..UnusedDepsSummary::default()
    };
    for package in packages {
        for dependency in &package.dependencies {
            summary.declared_dependency_count += 1;
            *summary
                .by_reason
                .entry(dependency.reason.clone())
                .or_insert(0) += 1;
            match dependency.status.as_str() {
                "used" => summary.used_count += 1,
                "muted" => summary.muted_count += 1,
                "review-unused" => summary.review_unused_count += 1,
                "confidence-limited" => summary.confidence_limited_count += 1,
                "unavailable" => summary.unavailable_count += 1,
                _ => {}
            }
        }
    }
    summary
}

fn value_string(value: &Value) -> String {
    value.as_str().unwrap_or("").to_string()
}
