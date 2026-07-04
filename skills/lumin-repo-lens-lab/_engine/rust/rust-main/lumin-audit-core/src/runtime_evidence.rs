use anyhow::{bail, Result};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub const RUNTIME_EVIDENCE_REQUEST_SCHEMA_VERSION: &str =
    "lumin-runtime-evidence-producer-request.v1";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeEvidenceRequest {
    pub schema_version: String,
    pub root: String,
    #[serde(default)]
    pub generated: Option<String>,
    #[serde(default)]
    pub symbols: Value,
    #[serde(default)]
    pub coverage: Value,
    #[serde(default)]
    pub coverage_source: Option<String>,
    #[serde(default)]
    pub coverage_mtime: Option<String>,
    #[serde(default)]
    pub symbols_source: Option<String>,
}

#[derive(Debug, Default)]
struct RuntimeStats {
    total: usize,
    grounded_dead: usize,
    degraded_fp_suspect: usize,
    degraded_uncovered: usize,
    degraded_type_only: usize,
    degraded_file_untested: usize,
}

#[derive(Debug)]
struct RuntimeVerdict {
    runtime_status: &'static str,
    hits_in_symbol: i64,
    stmts_in_symbol: Option<usize>,
    file_statements: usize,
    file_covered_statements: usize,
}

#[derive(Debug)]
struct CoverageIndex<'a> {
    by_abs: BTreeMap<String, &'a Value>,
    by_rel: BTreeMap<String, &'a Value>,
}

pub fn build_runtime_evidence_artifact(request: RuntimeEvidenceRequest) -> Result<Value> {
    if request.schema_version != RUNTIME_EVIDENCE_REQUEST_SCHEMA_VERSION {
        bail!(
            "runtime-evidence-artifact: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }
    if !request.symbols.is_object() {
        bail!("runtime-evidence-artifact: symbols must be an object");
    }
    if !request.coverage.is_object() {
        bail!("runtime-evidence-artifact: coverage must be an object");
    }

    let root = slash_path(&request.root);
    let dead_list = request
        .symbols
        .get("deadProdList")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let coverage = coverage_index(&root, &request.coverage);

    let mut merged = Vec::new();
    let mut stats = RuntimeStats {
        total: dead_list.len(),
        ..RuntimeStats::default()
    };
    let mut static_abs_files = Vec::new();
    let mut seen_static_abs_files = BTreeSet::new();

    for dead in &dead_list {
        let file = string_field(dead, "file").unwrap_or_default();
        let abs = absolute_key(&root, &file);
        if seen_static_abs_files.insert(abs.clone()) {
            static_abs_files.push(abs.clone());
        }

        if is_type_only(string_field(dead, "kind").as_deref()) {
            stats.degraded_type_only += 1;
            merged.push(merge_record(
                dead,
                json!({
                    "runtimeStatus": "type-only",
                    "grounding": "degraded",
                    "confidence": "medium",
                    "note": "Type-only declaration — erased at compile. Runtime evidence n/a; rely on AST.",
                }),
            ));
            continue;
        }

        let coverage_entry = coverage.by_abs.get(&abs).copied().or_else(|| {
            let rel = slash_path(&file);
            coverage.by_rel.get(&rel).copied()
        });
        let Some(entry) = coverage_entry else {
            stats.degraded_uncovered += 1;
            merged.push(merge_record(
                dead,
                json!({
                    "runtimeStatus": "uncovered",
                    "grounding": "degraded",
                    "confidence": "medium",
                    "note": "File not present in coverage output. Test range did not exercise this file.",
                }),
            ));
            continue;
        };

        let verdict = runtime_verdict_for(entry, number_field(dead, "line"));
        match verdict.runtime_status {
            "file-untested" => {
                stats.degraded_file_untested += 1;
                merged.push(merge_record(
                    dead,
                    json!({
                        "runtimeStatus": "file-untested",
                        "hitsInSymbol": 0,
                        "fileStatements": verdict.file_statements,
                        "grounding": "degraded",
                        "confidence": "medium",
                        "note": "File loaded by tests but 0 statements executed. Module-level test gap.",
                    }),
                ));
            }
            "executed" => {
                stats.degraded_fp_suspect += 1;
                merged.push(merge_record(
                    dead,
                    json!({
                        "runtimeStatus": "executed",
                        "hitsInSymbol": verdict.hits_in_symbol,
                        "stmtsInSymbol": verdict.stmts_in_symbol.unwrap_or(0),
                        "grounding": "degraded",
                        "confidence": "low",
                        "note": format!(
                            "AST says dead but runtime hit {}×. Likely dynamic use (reflection, string import, framework autowire). Probable FP — DO NOT remove without manual check.",
                            verdict.hits_in_symbol
                        ),
                    }),
                ));
            }
            _ => {
                stats.grounded_dead += 1;
                merged.push(merge_record(
                    dead,
                    json!({
                        "runtimeStatus": "dead-confirmed",
                        "hitsInSymbol": 0,
                        "stmtsInSymbol": verdict.stmts_in_symbol.unwrap_or(0),
                        "fileStatements": verdict.file_statements,
                        "fileCoveredStatements": verdict.file_covered_statements,
                        "grounding": "grounded",
                        "confidence": "high",
                        "note": "AST-dead and runtime zero-hit across covered range. Safe-to-remove with highest evidence tier.",
                    }),
                ));
            }
        }
    }

    let orphan_files: Vec<String> = static_abs_files
        .iter()
        .filter(|abs| !coverage.by_abs.contains_key(*abs))
        .map(|abs| rel_from_abs(&root, abs))
        .take(50)
        .collect();
    let orphan_static_files = static_abs_files
        .iter()
        .filter(|abs| !coverage.by_abs.contains_key(*abs))
        .count();
    let grounded_share_pct = if stats.total == 0 {
        0
    } else {
        ((stats.grounded_dead as f64 / stats.total as f64) * 100.0).round() as i64
    };

    Ok(json!({
        "meta": {
            "generated": request.generated.unwrap_or_else(|| "unknown".to_string()),
            "root": request.root,
            "tool": "merge-runtime-evidence.mjs",
            "coverageSource": request.coverage_source.unwrap_or_else(|| "unknown".to_string()),
            "coverageMtime": request.coverage_mtime.unwrap_or_else(|| "unknown".to_string()),
            "symbolsSource": request.symbols_source.unwrap_or_else(|| "symbols.json".to_string()),
        },
        "summary": {
            "total": stats.total,
            "grounded_dead": stats.grounded_dead,
            "degraded_fp_suspect": stats.degraded_fp_suspect,
            "degraded_uncovered": stats.degraded_uncovered,
            "degraded_type_only": stats.degraded_type_only,
            "degraded_file_untested": stats.degraded_file_untested,
            "coverageFileCount": coverage.by_abs.len(),
            "orphanStaticFiles": orphan_static_files,
            "groundedSharePct": grounded_share_pct,
        },
        "merged": merged,
        "orphanFilesSample": orphan_files,
    }))
}

fn runtime_verdict_for(entry: &Value, def_line: Option<i64>) -> RuntimeVerdict {
    let statement_map = entry.get("statementMap").and_then(Value::as_object);
    let statement_hits = entry.get("s").and_then(Value::as_object);
    let fn_map = entry.get("fnMap").and_then(Value::as_object);
    let fn_hits = entry.get("f").and_then(Value::as_object);

    let mut file_statements = 0;
    let mut file_covered_statements = 0;
    if let Some(statements) = statement_map {
        for id in statements.keys() {
            file_statements += 1;
            if object_number(statement_hits, id).unwrap_or(0) > 0 {
                file_covered_statements += 1;
            }
        }
    }

    if file_statements == 0 {
        return RuntimeVerdict {
            runtime_status: "file-untested",
            hits_in_symbol: 0,
            stmts_in_symbol: None,
            file_statements,
            file_covered_statements,
        };
    }

    let enclosing_fn_hits = enclosing_function_hits(fn_map, fn_hits, def_line);
    let mut hits_in_symbol = 0;
    let mut stmts_in_symbol = 0;

    if let Some(statements) = statement_map {
        for (id, statement) in statements {
            let Some(statement_start) = loc_line(statement, "start") else {
                continue;
            };
            let statement_end = loc_line(statement, "end").unwrap_or(statement_start);
            let within_fn = function_map_contains_statement_and_definition(
                fn_map,
                statement_start,
                statement_end,
                def_line,
            );
            let near_def_line = def_line
                .map(|line| (statement_start - line).abs() <= 50)
                .unwrap_or(false);
            if within_fn || (enclosing_fn_hits.is_none() && near_def_line) {
                stmts_in_symbol += 1;
                hits_in_symbol += object_number(statement_hits, id).unwrap_or(0);
            }
        }
    }

    let hits = enclosing_fn_hits
        .map(|hits| hits.hits)
        .unwrap_or(hits_in_symbol);
    RuntimeVerdict {
        runtime_status: if hits > 0 {
            "executed"
        } else {
            "dead-confirmed"
        },
        hits_in_symbol: hits,
        stmts_in_symbol: Some(stmts_in_symbol),
        file_statements,
        file_covered_statements,
    }
}

#[derive(Clone, Copy, Debug)]
struct FunctionHits {
    hits: i64,
    span: i64,
}

fn enclosing_function_hits(
    fn_map: Option<&Map<String, Value>>,
    fn_hits: Option<&Map<String, Value>>,
    def_line: Option<i64>,
) -> Option<FunctionHits> {
    let def_line = def_line?;
    let mut best = None;
    for (id, function) in fn_map.into_iter().flat_map(|map| map.iter()) {
        let Some(loc) = function.get("loc").or_else(|| function.get("decl")) else {
            continue;
        };
        let Some(start) = loc_line(loc, "start") else {
            continue;
        };
        let Some(end) = loc_line(loc, "end") else {
            continue;
        };
        if start <= def_line && def_line <= end {
            let candidate = FunctionHits {
                hits: object_number(fn_hits, id).unwrap_or(0),
                span: end - start,
            };
            if best.is_none_or(|current: FunctionHits| candidate.span < current.span) {
                best = Some(candidate);
            }
        }
    }
    best
}

fn function_map_contains_statement_and_definition(
    fn_map: Option<&Map<String, Value>>,
    statement_start: i64,
    statement_end: i64,
    def_line: Option<i64>,
) -> bool {
    let Some(def_line) = def_line else {
        return false;
    };
    fn_map
        .into_iter()
        .flat_map(|map| map.values())
        .any(|function| {
            let loc = function.get("loc").or_else(|| function.get("decl"));
            let Some(loc) = loc else {
                return false;
            };
            let Some(start) = loc_line(loc, "start") else {
                return false;
            };
            let Some(end) = loc_line(loc, "end") else {
                return false;
            };
            start <= statement_start && statement_end <= end && start <= def_line && def_line <= end
        })
}

fn coverage_index<'a>(root: &str, coverage: &'a Value) -> CoverageIndex<'a> {
    let mut by_abs = BTreeMap::new();
    let mut by_rel = BTreeMap::new();
    for (key, entry) in coverage.as_object().into_iter().flat_map(|map| map.iter()) {
        let raw_path = entry.get("path").and_then(Value::as_str).unwrap_or(key);
        let normalized_raw = slash_path(raw_path);
        let normalized_key = slash_path(key);

        by_abs.insert(absolute_key(root, &normalized_raw), entry);
        if path_is_absolute(&normalized_key) {
            by_abs.insert(slash_path(&normalized_key), entry);
        }

        if let Some(rel) = strip_root_prefix(root, &normalized_raw) {
            by_rel.insert(rel, entry);
        } else if !path_is_absolute(&normalized_raw) {
            by_rel.insert(normalized_raw.clone(), entry);
        }
        if let Some(rel) = strip_root_prefix(root, &normalized_key) {
            by_rel.insert(rel, entry);
        } else if !path_is_absolute(&normalized_key) {
            by_rel.insert(normalized_key, entry);
        }
    }
    CoverageIndex { by_abs, by_rel }
}

fn is_type_only(kind: Option<&str>) -> bool {
    matches!(
        kind,
        Some("TSInterfaceDeclaration" | "TSTypeAliasDeclaration" | "TSModuleDeclaration")
    )
}

fn merge_record(record: &Value, extras: Value) -> Value {
    let mut out = record.as_object().cloned().unwrap_or_default();
    if let Some(extra) = extras.as_object() {
        for (key, value) in extra {
            out.insert(key.clone(), value.clone());
        }
    }
    Value::Object(out)
}

fn object_number(object: Option<&Map<String, Value>>, key: &str) -> Option<i64> {
    object?.get(key).and_then(Value::as_i64).or_else(|| {
        object?
            .get(key)
            .and_then(Value::as_u64)
            .map(|value| value as i64)
    })
}

fn loc_line(value: &Value, edge: &str) -> Option<i64> {
    value.get(edge)?.get("line")?.as_i64()
}

fn string_field(value: &Value, field: &str) -> Option<String> {
    value.get(field)?.as_str().map(ToString::to_string)
}

fn number_field(value: &Value, field: &str) -> Option<i64> {
    value.get(field).and_then(Value::as_i64).or_else(|| {
        value
            .get(field)
            .and_then(Value::as_u64)
            .map(|value| value as i64)
    })
}

fn absolute_key(root: &str, file: &str) -> String {
    let file = slash_path(file);
    if path_is_absolute(&file) {
        file
    } else {
        slash_path(PathBuf::from(root).join(file).to_string_lossy())
    }
}

fn rel_from_abs(root: &str, abs: &str) -> String {
    strip_root_prefix(root, abs).unwrap_or_else(|| slash_path(abs))
}

fn strip_root_prefix(root: &str, path: &str) -> Option<String> {
    let root = slash_path(root).trim_end_matches('/').to_string();
    let path = slash_path(path);
    let root_cmp = root.to_ascii_lowercase();
    let path_cmp = path.to_ascii_lowercase();
    if path_cmp == root_cmp {
        return Some(String::new());
    }
    let prefix = format!("{root_cmp}/");
    if path_cmp.starts_with(&prefix) {
        return Some(path[root.len() + 1..].to_string());
    }
    None
}

fn path_is_absolute(path: &str) -> bool {
    Path::new(path).is_absolute() || path.as_bytes().get(1).is_some_and(|byte| *byte == b':')
}

fn slash_path(path: impl AsRef<str>) -> String {
    path.as_ref().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    fn request(symbols: Value, coverage: Value) -> RuntimeEvidenceRequest {
        RuntimeEvidenceRequest {
            schema_version: RUNTIME_EVIDENCE_REQUEST_SCHEMA_VERSION.to_string(),
            root: "C:/repo".to_string(),
            generated: Some("2026-07-03T00:00:00.000Z".to_string()),
            symbols,
            coverage,
            coverage_source: Some("coverage-final.json".to_string()),
            coverage_mtime: Some("2026-07-03T00:00:00.000Z".to_string()),
            symbols_source: Some("symbols.json".to_string()),
        }
    }

    #[test]
    fn classifies_dead_executed_uncovered_and_type_only_candidates() -> Result<()> {
        let artifact = build_runtime_evidence_artifact(request(
            json!({
                "deadProdList": [
                    { "file": "src/cold.ts", "line": 1, "symbol": "cold", "kind": "FunctionDeclaration" },
                    { "file": "src/hot.ts", "line": 1, "symbol": "hot", "kind": "FunctionDeclaration" },
                    { "file": "src/missing.ts", "line": 1, "symbol": "missing", "kind": "FunctionDeclaration" },
                    { "file": "src/types.ts", "line": 1, "symbol": "Options", "kind": "TSTypeAliasDeclaration" }
                ]
            }),
            json!({
                "C:/repo/src/cold.ts": coverage_entry("C:/repo/src/cold.ts", 0),
                "C:/repo/src/hot.ts": coverage_entry("C:/repo/src/hot.ts", 3)
            }),
        ))?;

        assert_eq!(artifact["summary"]["total"], 4);
        assert_eq!(artifact["summary"]["grounded_dead"], 1);
        assert_eq!(artifact["summary"]["degraded_fp_suspect"], 1);
        assert_eq!(artifact["summary"]["degraded_uncovered"], 1);
        assert_eq!(artifact["summary"]["degraded_type_only"], 1);
        let merged = artifact["merged"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("merged must be an array"))?;
        assert_eq!(merged[0]["runtimeStatus"], "dead-confirmed");
        assert_eq!(merged[1]["runtimeStatus"], "executed");
        assert_eq!(merged[1]["hitsInSymbol"], 3);
        assert_eq!(merged[2]["runtimeStatus"], "uncovered");
        assert_eq!(merged[3]["runtimeStatus"], "type-only");
        Ok(())
    }

    #[test]
    fn file_with_no_statements_is_file_untested() -> Result<()> {
        let artifact = build_runtime_evidence_artifact(request(
            json!({
                "deadProdList": [
                    { "file": "src/empty.ts", "line": 1, "symbol": "empty", "kind": "FunctionDeclaration" }
                ]
            }),
            json!({
                "C:/repo/src/empty.ts": {
                    "path": "C:/repo/src/empty.ts",
                    "statementMap": {},
                    "s": {},
                    "fnMap": {},
                    "f": {}
                }
            }),
        ))?;
        assert_eq!(artifact["summary"]["degraded_file_untested"], 1);
        assert_eq!(artifact["merged"][0]["runtimeStatus"], "file-untested");
        Ok(())
    }

    #[test]
    fn rejects_bad_request_shape() {
        let result = build_runtime_evidence_artifact(RuntimeEvidenceRequest {
            schema_version: "wrong".to_string(),
            root: "C:/repo".to_string(),
            generated: None,
            symbols: json!({}),
            coverage: json!({}),
            coverage_source: None,
            coverage_mtime: None,
            symbols_source: None,
        });
        assert!(result.is_err());
    }

    fn coverage_entry(path: &str, hits: i64) -> Value {
        json!({
            "path": path,
            "statementMap": {
                "0": { "start": { "line": 1 }, "end": { "line": 3 } }
            },
            "s": { "0": hits },
            "fnMap": {
                "0": { "loc": { "start": { "line": 1 }, "end": { "line": 3 } } }
            },
            "f": { "0": hits }
        })
    }
}
