use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION: &str = "lumin-source-use-assembly-request.v1";
pub const SOURCE_USE_ASSEMBLY_RESPONSE_SCHEMA_VERSION: &str =
    "lumin-source-use-assembly-response.v1";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceUseAssemblyRequest {
    pub schema_version: String,
    pub root: String,
    #[serde(default)]
    pub source_files: Vec<String>,
    #[serde(default)]
    pub records: Vec<SourceUseAssemblyRecord>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceUseAssemblyRecord {
    pub record_id: String,
    pub consumer_file: String,
    #[serde(default)]
    pub resolved_file: Option<String>,
    #[serde(default)]
    pub from_spec: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub type_only: bool,
    #[serde(default)]
    pub line: Option<u64>,
    #[serde(default)]
    pub sfc_language: Option<String>,
    #[serde(default)]
    pub resolver_stage: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceUseAssemblyResponse {
    pub schema_version: &'static str,
    pub root: String,
    pub summary: SourceUseAssemblySummary,
    pub handled_record_ids: Vec<String>,
    pub skipped_records: Vec<SkippedSourceUseRecord>,
    pub counters: SourceUseAssemblyCounters,
    pub branch_counts: BTreeMap<String, usize>,
    pub resolved_internal_edges: Vec<ResolvedInternalEdge>,
    pub direct_consumers: Vec<DirectConsumerAddition>,
    pub namespace_users: Vec<NamespaceUserAddition>,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceUseAssemblySummary {
    pub record_count: usize,
    pub handled_count: usize,
    pub skipped_count: usize,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceUseAssemblyCounters {
    pub total_uses: usize,
    pub resolved_internal_uses: usize,
    pub rust_resolved_relative_uses: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedInternalEdge {
    pub from: String,
    pub to: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub type_only: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sfc_language: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectConsumerAddition {
    pub def_file: String,
    pub symbol: String,
    pub consumer_file: String,
    pub space: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NamespaceUserAddition {
    pub def_file: String,
    pub consumer_file: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkippedSourceUseRecord {
    pub record_id: String,
    pub reason: &'static str,
}

pub fn build_source_use_assembly_response(
    request: SourceUseAssemblyRequest,
) -> Result<SourceUseAssemblyResponse> {
    if request.schema_version != SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION {
        bail!(
            "source-use-assembly-artifact: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }

    let mut response = SourceUseAssemblyResponse {
        schema_version: SOURCE_USE_ASSEMBLY_RESPONSE_SCHEMA_VERSION,
        root: request.root.clone(),
        summary: SourceUseAssemblySummary {
            record_count: request.records.len(),
            ..SourceUseAssemblySummary::default()
        },
        handled_record_ids: Vec::new(),
        skipped_records: Vec::new(),
        counters: SourceUseAssemblyCounters::default(),
        branch_counts: BTreeMap::new(),
        resolved_internal_edges: Vec::new(),
        direct_consumers: Vec::new(),
        namespace_users: Vec::new(),
    };

    let root = normalize_path_text(&request.root);
    let resolver = RelativeSourceResolver::new(request.source_files);
    let mut namespace_users_seen = BTreeSet::new();

    for record in request.records {
        if record
            .resolver_stage
            .as_deref()
            .is_some_and(|stage| stage != "relative")
        {
            skip(
                &mut response,
                record.record_id,
                "non-relative-resolver-stage",
            );
            continue;
        }
        let from_spec = record.from_spec.as_deref().unwrap_or_default();
        if !is_relative_spec(from_spec) {
            skip(&mut response, record.record_id, "non-relative-specifier");
            continue;
        }
        let resolved_file = record
            .resolved_file
            .as_deref()
            .filter(|path| !path.is_empty())
            .map(ToString::to_string)
            .or_else(|| resolver.resolve(&record.consumer_file, from_spec));
        let Some(resolved_file) = resolved_file else {
            skip(&mut response, record.record_id, "relative-target-missing");
            continue;
        };

        let kind = record.kind.as_deref().unwrap_or("import");
        if is_namespace_reexport_use(kind) {
            skip(
                &mut response,
                record.record_id,
                "namespace-reexport-required",
            );
            continue;
        }
        if kind == "import-meta-glob" {
            skip(
                &mut response,
                record.record_id,
                "import-meta-glob-expansion-required",
            );
            continue;
        }
        if requires_symbol_name(kind) && record.name.as_deref().map(str::is_empty).unwrap_or(true) {
            skip(&mut response, record.record_id, "missing-symbol-name");
            continue;
        }

        let from = root_relative(&root, &record.consumer_file);
        let to = root_relative(&root, &resolved_file);
        let record_id = record.record_id;
        let source = record.from_spec.clone();

        response.handled_record_ids.push(record_id);
        response.counters.total_uses += 1;
        response.counters.resolved_internal_uses += 1;
        response.counters.rust_resolved_relative_uses += 1;
        increment_branch(&mut response.branch_counts, "resolvedInternal");
        response.resolved_internal_edges.push(ResolvedInternalEdge {
            from: from.clone(),
            to: to.clone(),
            kind: edge_kind_for_use(kind).to_string(),
            source,
            type_only: record.type_only,
            line: record.line,
            sfc_language: record.sfc_language,
        });

        if kind == "cjs-side-effect-only" || kind == "import-side-effect" {
            increment_branch(&mut response.branch_counts, "sideEffectOnly");
            continue;
        }
        if kind == "reExportNamespace" {
            increment_branch(&mut response.branch_counts, "reExportNamespaceSkip");
            continue;
        }
        if is_broad_namespace_use(kind) {
            increment_branch(&mut response.branch_counts, "broadNamespace");
            if namespace_users_seen.insert((to.clone(), from.clone())) {
                response.namespace_users.push(NamespaceUserAddition {
                    def_file: to,
                    consumer_file: from,
                });
            }
            continue;
        }

        let symbol = record.name.unwrap_or_default();
        increment_branch(&mut response.branch_counts, "directConsumer");
        response.direct_consumers.push(DirectConsumerAddition {
            def_file: to,
            symbol,
            consumer_file: from,
            space: if record.type_only { "type" } else { "value" },
        });
    }

    response.summary.handled_count = response.handled_record_ids.len();
    response.summary.skipped_count = response.skipped_records.len();
    Ok(response)
}

fn skip(response: &mut SourceUseAssemblyResponse, record_id: String, reason: &'static str) {
    response
        .skipped_records
        .push(SkippedSourceUseRecord { record_id, reason });
}

fn increment_branch(branch_counts: &mut BTreeMap<String, usize>, name: &str) {
    *branch_counts.entry(name.to_string()).or_insert(0) += 1;
}

fn is_namespace_reexport_use(kind: &str) -> bool {
    kind == "imported-namespace-member" || kind == "imported-namespace-escape"
}

fn is_relative_spec(spec: &str) -> bool {
    spec.starts_with("./") || spec.starts_with("../")
}

fn is_broad_namespace_use(kind: &str) -> bool {
    matches!(
        kind,
        "namespace" | "reExportAll" | "dynamic" | "cjs-namespace-escape" | "cjs-reexport-broad"
    )
}

fn requires_symbol_name(kind: &str) -> bool {
    !matches!(
        kind,
        "cjs-side-effect-only"
            | "import-side-effect"
            | "reExportNamespace"
            | "namespace"
            | "reExportAll"
            | "dynamic"
            | "cjs-namespace-escape"
            | "cjs-reexport-broad"
    )
}

fn edge_kind_for_use(kind: &str) -> &str {
    match kind {
        "import" => "import-named",
        "default" => "import-default",
        "namespace" | "namespace-member" => "import-namespace",
        "import-side-effect" => "import-side-effect",
        "reExport" => "reexport-named",
        "reExportAll" => "reexport-broad",
        "reExportNamespace" => "reexport-namespace",
        "imported-namespace-member" => "reexport-namespace-member",
        "imported-namespace-escape" => "reexport-namespace-escape",
        "dynamic" | "dynamic-member" => "dynamic-literal",
        "cjs-side-effect-only" => "cjs-side-effect",
        "cjs-require-exact" => "cjs-require-exact",
        "cjs-namespace-member" => "cjs-namespace-member",
        "cjs-namespace-escape" => "cjs-namespace-escape",
        "cjs-reexport-broad" => "cjs-reexport-broad",
        other => other,
    }
}

fn root_relative(root: &str, path: &str) -> String {
    let normalized = normalize_path_text(path);
    let trimmed_root = root.trim_end_matches('/');
    if let Some(rest) = normalized.strip_prefix(&format!("{trimmed_root}/")) {
        return rest.to_string();
    }
    if normalized == trimmed_root {
        return ".".to_string();
    }
    normalized
}

const RESOLVE_FILE_EXTS: &[&str] = &[
    "", ".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs", ".mts", ".cts", ".d.ts", ".d.mts", ".d.cts",
];

const RESOLVE_INDEX_EXTS: &[&str] = &[
    "/index.ts",
    "/index.tsx",
    "/index.js",
    "/index.jsx",
    "/index.mjs",
    "/index.cjs",
    "/index.mts",
    "/index.cts",
    "/index.d.ts",
    "/index.d.mts",
    "/index.d.cts",
];

#[derive(Debug)]
struct RelativeSourceResolver {
    source_files: BTreeMap<String, String>,
}

impl RelativeSourceResolver {
    fn new(source_files: Vec<String>) -> Self {
        let mut out = BTreeMap::new();
        for source_file in source_files {
            out.entry(normalize_path_text(&source_file))
                .or_insert(source_file);
        }
        Self { source_files: out }
    }

    fn resolve(&self, from_file: &str, spec: &str) -> Option<String> {
        if !is_relative_spec(spec) {
            return None;
        }
        let base = join_relative_spec(dirname_text(from_file), spec);
        for ext in RESOLVE_FILE_EXTS {
            if let Some(resolved) = self.source_file(&format!("{base}{ext}")) {
                return Some(resolved);
            }
        }
        for ext in RESOLVE_INDEX_EXTS {
            if let Some(resolved) = self.source_file(&format!("{base}{ext}")) {
                return Some(resolved);
            }
        }
        if js_output_extension(spec) {
            for alt in [".ts", ".tsx", ".mts", ".cts"] {
                if let Some(swapped) = replace_js_output_extension(spec, alt) {
                    let candidate = join_relative_spec(dirname_text(from_file), &swapped);
                    if let Some(resolved) = self.source_file(&candidate) {
                        return Some(resolved);
                    }
                }
            }
        }
        if js_output_extension(spec) {
            if let Some(stripped) = strip_js_output_extension(&base) {
                for ext in RESOLVE_INDEX_EXTS {
                    if let Some(resolved) = self.source_file(&format!("{stripped}{ext}")) {
                        return Some(resolved);
                    }
                }
            }
        }
        None
    }

    fn source_file(&self, candidate: &str) -> Option<String> {
        self.source_files
            .get(&normalize_path_text(candidate))
            .cloned()
    }
}

fn dirname_text(path: &str) -> &str {
    let normalized = path.rfind(['/', '\\']);
    normalized.map_or("", |index| &path[..index])
}

fn join_relative_spec(base: &str, spec: &str) -> String {
    let joined = if base.is_empty() {
        spec.to_string()
    } else {
        format!("{base}/{spec}")
    };
    normalize_path_text(&joined)
}

fn normalize_path_text(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    let (prefix, rest) = split_path_prefix(&normalized);
    let absolute = rest.starts_with('/');
    let mut parts = Vec::new();
    for part in rest.split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." {
            if let Some(last) = parts.last() {
                if last != &".." {
                    parts.pop();
                    continue;
                }
            }
            if !absolute {
                parts.push(part);
            }
            continue;
        }
        parts.push(part);
    }

    let body = parts.join("/");
    match (prefix.is_empty(), absolute, body.is_empty()) {
        (false, _, false) => format!("{prefix}/{body}"),
        (false, _, true) => prefix.to_string(),
        (true, true, false) => format!("/{body}"),
        (true, true, true) => "/".to_string(),
        (true, false, false) => body,
        (true, false, true) => ".".to_string(),
    }
}

fn split_path_prefix(path: &str) -> (&str, &str) {
    let bytes = path.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
        let prefix = &path[..2];
        let rest = path.get(2..).unwrap_or_default();
        return (prefix, rest);
    }
    ("", path)
}

fn js_output_extension(spec: &str) -> bool {
    [".mjs", ".cjs", ".js", ".jsx"]
        .iter()
        .any(|ext| spec.ends_with(ext))
}

fn replace_js_output_extension(spec: &str, alt: &str) -> Option<String> {
    for ext in [".mjs", ".cjs", ".js", ".jsx"] {
        if let Some(replaced) = replace_suffix(spec, ext, alt) {
            return Some(replaced);
        }
    }
    None
}

fn replace_suffix(value: &str, suffix: &str, replacement: &str) -> Option<String> {
    value
        .strip_suffix(suffix)
        .map(|prefix| format!("{prefix}{replacement}"))
}

fn strip_js_output_extension(spec: &str) -> Option<&str> {
    for ext in [".mjs", ".cjs", ".js", ".jsx"] {
        if let Some(prefix) = spec.strip_suffix(ext) {
            return Some(prefix);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn must_request(value: serde_json::Value) -> SourceUseAssemblyRequest {
        match serde_json::from_value(value) {
            Ok(request) => request,
            Err(error) => panic!("test request must deserialize: {error}"),
        }
    }

    fn request(records: serde_json::Value) -> SourceUseAssemblyRequest {
        must_request(json!({
            "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
            "root": "C:/repo",
            "records": records
        }))
    }

    fn response(request: SourceUseAssemblyRequest) -> SourceUseAssemblyResponse {
        match build_source_use_assembly_response(request) {
            Ok(response) => response,
            Err(error) => panic!("test response must build: {error}"),
        }
    }

    #[test]
    fn assembles_direct_and_namespace_relative_uses() {
        let response = response(request(json!([
            {
                "recordId": "src/a.ts#0",
                "consumerFile": "C:/repo/src/a.ts",
                "resolvedFile": "C:/repo/src/b.ts",
                "fromSpec": "./b",
                "name": "thing",
                "kind": "import",
                "typeOnly": false,
                "line": 3,
                "resolverStage": "relative"
            },
            {
                "recordId": "src/c.ts#0",
                "consumerFile": "C:/repo/src/c.ts",
                "resolvedFile": "C:/repo/src/d.ts",
                "fromSpec": "./d",
                "kind": "namespace",
                "resolverStage": "relative"
            }
        ])));

        assert_eq!(response.summary.handled_count, 2);
        assert_eq!(response.counters.total_uses, 2);
        assert_eq!(response.counters.resolved_internal_uses, 2);
        assert_eq!(response.branch_counts["resolvedInternal"], 2);
        assert_eq!(response.branch_counts["directConsumer"], 1);
        assert_eq!(response.branch_counts["broadNamespace"], 1);
        assert_eq!(response.resolved_internal_edges[0].from, "src/a.ts");
        assert_eq!(response.resolved_internal_edges[0].to, "src/b.ts");
        assert_eq!(response.resolved_internal_edges[0].kind, "import-named");
        assert_eq!(response.direct_consumers[0].symbol, "thing");
        assert_eq!(response.namespace_users[0].def_file, "src/d.ts");
    }

    #[test]
    fn skips_namespace_reexport_and_non_relative_records_for_js_fallback() {
        let response = response(request(json!([
            {
                "recordId": "a",
                "consumerFile": "C:/repo/src/a.ts",
                "resolvedFile": "C:/repo/src/b.ts",
                "fromSpec": "./b",
                "kind": "imported-namespace-member",
                "resolverStage": "relative"
            },
            {
                "recordId": "b",
                "consumerFile": "C:/repo/src/a.ts",
                "resolvedFile": "C:/repo/src/b.ts",
                "fromSpec": "./b",
                "kind": "import",
                "resolverStage": "alias"
            }
        ])));

        assert_eq!(response.summary.handled_count, 0);
        assert_eq!(response.summary.skipped_count, 2);
        assert_eq!(
            response.skipped_records[0].reason,
            "namespace-reexport-required"
        );
        assert_eq!(
            response.skipped_records[1].reason,
            "non-relative-resolver-stage"
        );
    }

    #[test]
    fn side_effect_uses_keep_edges_without_consumers() {
        let response = response(request(json!([
            {
                "recordId": "a",
                "consumerFile": "C:/repo/src/a.ts",
                "resolvedFile": "C:/repo/src/setup.ts",
                "fromSpec": "./setup",
                "kind": "import-side-effect",
                "resolverStage": "relative"
            }
        ])));

        assert_eq!(response.summary.handled_count, 1);
        assert_eq!(response.branch_counts["sideEffectOnly"], 1);
        assert_eq!(
            response.resolved_internal_edges[0].kind,
            "import-side-effect"
        );
        assert!(response.direct_consumers.is_empty());
        assert!(response.namespace_users.is_empty());
    }

    #[test]
    fn resolves_relative_targets_from_source_files_when_resolved_file_is_absent() {
        let request = must_request(json!({
            "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
            "root": "C:/repo",
            "sourceFiles": [
                "C:/repo/src/consumer.ts",
                "C:/repo/src/dep.ts"
            ],
            "records": [
                {
                    "recordId": "src/consumer.ts#0",
                    "consumerFile": "C:/repo/src/consumer.ts",
                    "fromSpec": "./dep",
                    "name": "value",
                    "kind": "import"
                }
            ]
        }));
        let response = response(request);

        assert_eq!(response.summary.handled_count, 1);
        assert_eq!(response.resolved_internal_edges[0].to, "src/dep.ts");
        assert_eq!(response.direct_consumers[0].symbol, "value");
    }

    #[test]
    fn jsx_output_import_preserves_js_relative_resolver_swap_order() {
        let request = must_request(json!({
            "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
            "root": "C:/repo",
            "sourceFiles": [
                "C:/repo/src/consumer.ts",
                "C:/repo/src/view.ts",
                "C:/repo/src/view.tsx"
            ],
            "records": [
                {
                    "recordId": "src/consumer.ts#0",
                    "consumerFile": "C:/repo/src/consumer.ts",
                    "fromSpec": "./view.jsx",
                    "name": "view",
                    "kind": "import"
                }
            ]
        }));
        let response = response(request);

        assert_eq!(response.summary.handled_count, 1);
        assert_eq!(response.resolved_internal_edges[0].to, "src/view.ts");
    }

    #[test]
    fn unresolved_relative_targets_are_left_for_js_fallback() {
        let request = must_request(json!({
            "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
            "root": "C:/repo",
            "sourceFiles": ["C:/repo/src/consumer.ts"],
            "records": [
                {
                    "recordId": "src/consumer.ts#0",
                    "consumerFile": "C:/repo/src/consumer.ts",
                    "fromSpec": "./missing",
                    "name": "value",
                    "kind": "import"
                }
            ]
        }));
        let response = response(request);

        assert_eq!(response.summary.handled_count, 0);
        assert_eq!(
            response.skipped_records[0].reason,
            "relative-target-missing"
        );
    }
}
