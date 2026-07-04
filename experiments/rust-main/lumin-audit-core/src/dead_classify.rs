use anyhow::{bail, Result};
use serde::Deserialize;
use serde_json::{json, Map, Value};

pub const DEAD_CLASSIFY_REQUEST_SCHEMA_VERSION: &str = "lumin-dead-classify-producer-request.v1";

const PROVENANCE_FIELD_NAMES: &[&str] = &[
    "fileInternalUsesEvidence",
    "fileInternalRefs",
    "parseError",
    "supportedBy",
    "taintedBy",
    "resolverConfidence",
    "parseStatus",
    "declarationExportDependency",
    "declarationExportRefs",
];

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeadClassifyRequest {
    pub schema_version: String,
    #[serde(default)]
    pub classified_candidates: Vec<Value>,
    #[serde(default)]
    pub excluded_candidates: Vec<Value>,
    #[serde(default)]
    pub unprocessed_candidates: Vec<Value>,
    #[serde(default)]
    pub excluded_summary: Value,
    #[serde(default)]
    pub framework_policy: Value,
    #[serde(default)]
    pub performance: Value,
    #[serde(default)]
    pub incomplete: bool,
}

#[derive(Debug)]
pub struct DeadClassifyArtifact(pub Value);

impl serde::Serialize for DeadClassifyArtifact {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Category {
    C,
    A,
    B,
}

impl Category {
    fn from_uses(uses: i64) -> Self {
        if uses == 0 {
            Self::C
        } else if uses <= 2 {
            Self::A
        } else {
            Self::B
        }
    }
}

pub fn build_dead_classify_artifact(request: DeadClassifyRequest) -> Result<DeadClassifyArtifact> {
    if request.schema_version != DEAD_CLASSIFY_REQUEST_SCHEMA_VERSION {
        bail!(
            "dead-classify-artifact: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }

    let mut category_c = Vec::new();
    let mut category_a = Vec::new();
    let mut category_b = Vec::new();
    let mut aliased = Vec::new();
    let mut with_predicate = 0usize;

    for candidate in request.classified_candidates {
        let uses = file_internal_uses(&candidate)?;
        let category = Category::from_uses(uses);
        if candidate
            .get("predicatePartner")
            .is_some_and(|value| !value.is_null())
        {
            with_predicate += 1;
        }
        if is_aliased_spec(&candidate) {
            aliased.push((candidate, category));
        } else {
            match category {
                Category::C => category_c.push(candidate),
                Category::A => category_a.push(candidate),
                Category::B => category_b.push(candidate),
            }
        }
    }

    let category_c_count = category_c.len()
        + aliased
            .iter()
            .filter(|(_, category)| *category == Category::C)
            .count();
    let category_a_count = category_a.len()
        + aliased
            .iter()
            .filter(|(_, category)| *category == Category::A)
            .count();
    let category_b_count = category_b.len()
        + aliased
            .iter()
            .filter(|(_, category)| *category == Category::B)
            .count();
    let total = category_c_count + category_a_count + category_b_count;

    let proposal_remove_export_specifier = aliased
        .iter()
        .map(|(c, _)| specifier_proposal(c))
        .collect::<Result<Vec<_>>>()?;
    let proposal_c = category_c
        .iter()
        .map(remove_symbol_proposal)
        .collect::<Result<Vec<_>>>()?;
    let proposal_a = category_a
        .iter()
        .map(demote_to_internal_proposal)
        .collect::<Result<Vec<_>>>()?;
    let proposal_b = category_b
        .iter()
        .map(review_proposal)
        .collect::<Result<Vec<_>>>()?;
    let proposal_degraded = request
        .unprocessed_candidates
        .iter()
        .map(unprocessed_proposal)
        .collect::<Vec<_>>();

    let artifact = json!({
        "summary": {
            "total": total,
            "category_C": category_c_count,
            "category_A": category_a_count,
            "category_B": category_b_count,
            "aliased_export_specifier": proposal_remove_export_specifier.len(),
            "with_predicate": with_predicate,
            "excluded": request.excluded_summary,
            "frameworkPolicy": request.framework_policy,
            "incomplete": request.incomplete,
            "performance": request.performance,
        },
        "proposal_remove_export_specifier": proposal_remove_export_specifier,
        "proposal_C_remove_symbol": proposal_c,
        "proposal_A_demote_to_internal": proposal_a,
        "proposal_B_review": proposal_b,
        "proposal_DEGRADED_unprocessed": proposal_degraded,
        "excludedCandidates": request.excluded_candidates,
    });

    Ok(DeadClassifyArtifact(artifact))
}

fn file_internal_uses(candidate: &Value) -> Result<i64> {
    let uses = candidate
        .get("fileInternalUses")
        .and_then(Value::as_i64)
        .or_else(|| candidate.get("localInternalUses").and_then(Value::as_i64))
        .ok_or_else(|| {
            let symbol = string_field(candidate, "symbol");
            anyhow::anyhow!(
                "dead-classify-artifact: candidate '{symbol}' is missing numeric fileInternalUses"
            )
        })?;
    if uses < 0 {
        let symbol = string_field(candidate, "symbol");
        bail!("dead-classify-artifact: candidate '{symbol}' has negative fileInternalUses");
    }
    Ok(uses)
}

fn local_internal_uses(candidate: &Value) -> Result<i64> {
    let uses = candidate
        .get("localInternalUses")
        .and_then(Value::as_i64)
        .ok_or_else(|| {
            let symbol = string_field(candidate, "symbol");
            anyhow::anyhow!(
                "dead-classify-artifact: aliased candidate '{symbol}' is missing numeric localInternalUses"
            )
        })?;
    if uses < 0 {
        let symbol = string_field(candidate, "symbol");
        bail!(
            "dead-classify-artifact: aliased candidate '{symbol}' has negative localInternalUses"
        );
    }
    Ok(uses)
}

fn is_aliased_spec(candidate: &Value) -> bool {
    candidate.get("kind").and_then(Value::as_str) == Some("ExportSpecifier")
        && candidate.get("localName").and_then(Value::as_str).is_some()
        && candidate.get("localName").and_then(Value::as_str)
            != candidate.get("symbol").and_then(Value::as_str)
}

fn string_field(candidate: &Value, field: &str) -> String {
    candidate
        .get(field)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn copy_base(candidate: &Value) -> Map<String, Value> {
    let mut out = Map::new();
    copy_field(candidate, &mut out, "file");
    copy_field(candidate, &mut out, "line");
    copy_field(candidate, &mut out, "symbol");
    copy_field(candidate, &mut out, "kind");
    out
}

fn copy_field(candidate: &Value, out: &mut Map<String, Value>, field: &str) {
    if let Some(value) = candidate.get(field) {
        out.insert(field.to_string(), value.clone());
    }
}

fn copy_provenance(candidate: &Value, out: &mut Map<String, Value>) {
    for field in PROVENANCE_FIELD_NAMES {
        copy_field(candidate, out, field);
    }
}

fn specifier_proposal(candidate: &Value) -> Result<Value> {
    let local_name = string_field(candidate, "localName");
    let symbol = string_field(candidate, "symbol");
    let local_internal_uses = local_internal_uses(candidate)?;
    let local_also_dead = local_internal_uses == 0;

    let mut out = copy_base(candidate);
    out.insert("localName".to_string(), Value::String(local_name.clone()));
    out.insert(
        "localInternalUses".to_string(),
        Value::Number(local_internal_uses.into()),
    );
    out.insert("localAlsoDead".to_string(), Value::Bool(local_also_dead));
    let action = if local_also_dead {
        format!(
            "`export {{ {local_name} as {symbol} }}` 제거. 참고: `{local_name}` 도 파일 내 다른 곳에서 쓰이지 않음 — 정의도 함께 제거 후보."
        )
    } else {
        format!(
            "`export {{ {local_name} as {symbol} }}` 제거만. `{local_name}` 은 파일 내부에서 사용 중이므로 정의는 유지."
        )
    };
    out.insert("action".to_string(), Value::String(action));
    copy_provenance(candidate, &mut out);
    Ok(Value::Object(out))
}

fn remove_symbol_proposal(candidate: &Value) -> Result<Value> {
    let mut out = copy_base(candidate);
    let uses = file_internal_uses(candidate)?;
    out.insert("fileInternalUses".to_string(), Value::Number(uses.into()));
    out.insert(
        "action".to_string(),
        Value::String("정의 자체 제거 가능. 어디서도 쓰이지 않음.".to_string()),
    );
    copy_provenance(candidate, &mut out);
    Ok(Value::Object(out))
}

fn demote_to_internal_proposal(candidate: &Value) -> Result<Value> {
    let mut out = copy_base(candidate);
    let uses = file_internal_uses(candidate)?;
    out.insert("fileInternalUses".to_string(), Value::Number(uses.into()));
    out.insert(
        "action".to_string(),
        Value::String("export 제거. 파일 내부 타입/함수로 강등.".to_string()),
    );
    copy_provenance(candidate, &mut out);
    Ok(Value::Object(out))
}

fn review_proposal(candidate: &Value) -> Result<Value> {
    let predicate = candidate.get("predicatePartner").cloned();
    let mut out = copy_base(candidate);
    let uses = file_internal_uses(candidate)?;
    out.insert("fileInternalUses".to_string(), Value::Number(uses.into()));
    if let Some(predicate) = predicate.clone() {
        out.insert("predicatePartner".to_string(), predicate);
    }
    let action = predicate
        .and_then(|value| value.as_str().map(str::to_string))
        .filter(|value| !value.is_empty())
        .map(|predicate| {
            format!("predicate({predicate}) 존재. type + predicate 패턴일 가능성. 유지 권장.")
        })
        .unwrap_or_else(|| {
            "파일 내 중심 타입. 설계상 public API 의도인지 확인 후 결정.".to_string()
        });
    out.insert("action".to_string(), Value::String(action));
    copy_provenance(candidate, &mut out);
    Ok(Value::Object(out))
}

fn unprocessed_proposal(candidate: &Value) -> Value {
    let mut out = Map::new();
    copy_field(candidate, &mut out, "file");
    copy_field(candidate, &mut out, "line");
    copy_field(candidate, &mut out, "symbol");
    copy_field(candidate, &mut out, "kind");
    copy_field(candidate, &mut out, "reason");
    out.insert(
        "action".to_string(),
        Value::String(
            "classification incomplete; rerun with a larger classify time budget before making removal claims."
                .to_string(),
        ),
    );
    Value::Object(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn request(classified_candidates: Vec<Value>) -> DeadClassifyRequest {
        DeadClassifyRequest {
            schema_version: DEAD_CLASSIFY_REQUEST_SCHEMA_VERSION.to_string(),
            classified_candidates,
            excluded_candidates: vec![json!({
                "file": "src/config.ts",
                "line": 1,
                "symbol": "Config",
                "kind": "VariableDeclaration",
                "reason": "config_FP22"
            })],
            unprocessed_candidates: vec![json!({
                "file": "src/huge.ts",
                "line": 9,
                "symbol": "Huge",
                "kind": "FunctionDeclaration",
                "reason": "classify-max-file-bytes"
            })],
            excluded_summary: json!({
                "config_FP22": 1,
                "publicApi_FP23": 0,
                "scriptEntrypoint_FP45": 0,
                "htmlEntrypoint_FP47": 0,
                "frameworkSentinel_FP27": 0,
                "nuxtNitro_FP30": 0,
                "vitePress_FP46": 0,
                "declarationSidecar_FP48": 0,
                "dynamicImportOpacity_FP18": 0,
                "testConsumer_FP44": 0,
                "transitiveBarrelAdded_FP25": 0,
                "isNuxtNitroDetected": false,
                "testConsumerDiagnostics_FP44": 0
            }),
            framework_policy: json!({"total": 0}),
            performance: json!({"deadCandidatesProcessed": 5}),
            incomplete: true,
        }
    }

    #[test]
    fn classifies_counts_into_strong_c_a_b_buckets() -> Result<()> {
        let artifact = build_dead_classify_artifact(request(vec![
            json!({
                "file": "src/dead.ts",
                "line": 1,
                "symbol": "Dead",
                "kind": "FunctionDeclaration",
                "fileInternalUses": 0,
                "fileInternalUsesEvidence": "ast-ident-ref-count",
                "supportedBy": [{"kind": "ast-ident-ref-count", "count": 0}]
            }),
            json!({
                "file": "src/demote.ts",
                "line": 2,
                "symbol": "Demote",
                "kind": "TSTypeAliasDeclaration",
                "fileInternalUses": 2,
                "fileInternalRefs": {"typeRefs": 2, "valueRefs": 0}
            }),
            json!({
                "file": "src/hub.ts",
                "line": 3,
                "symbol": "Hub",
                "kind": "TSInterfaceDeclaration",
                "fileInternalUses": 3,
                "predicatePartner": "isHub"
            }),
        ]))?
        .0;

        assert_eq!(artifact["summary"]["category_C"], 1);
        assert_eq!(artifact["summary"]["category_A"], 1);
        assert_eq!(artifact["summary"]["category_B"], 1);
        assert_eq!(artifact["proposal_C_remove_symbol"][0]["symbol"], "Dead");
        assert_eq!(
            artifact["proposal_C_remove_symbol"][0]["supportedBy"][0]["count"],
            0
        );
        assert_eq!(
            artifact["proposal_A_demote_to_internal"][0]["fileInternalRefs"]["typeRefs"],
            2
        );
        assert_eq!(
            artifact["proposal_B_review"][0]["predicatePartner"],
            "isHub"
        );
        assert!(artifact["proposal_B_review"][0]["action"]
            .as_str()
            .is_some_and(|action| action.contains("predicate(isHub)")));
        Ok(())
    }

    #[test]
    fn aliased_export_specifier_keeps_local_binding_contract() -> Result<()> {
        let artifact = build_dead_classify_artifact(request(vec![json!({
            "file": "src/alias.ts",
            "line": 4,
            "symbol": "PublicAlias",
            "localName": "localImpl",
            "kind": "ExportSpecifier",
            "fileInternalUses": 1,
            "localInternalUses": 1
        })]))?
        .0;

        assert_eq!(artifact["summary"]["category_C"], 0);
        assert_eq!(artifact["summary"]["category_A"], 1);
        assert_eq!(artifact["summary"]["aliased_export_specifier"], 1);
        assert_eq!(
            artifact["proposal_remove_export_specifier"][0]["localAlsoDead"],
            false
        );
        assert!(artifact["proposal_remove_export_specifier"][0]["action"]
            .as_str()
            .is_some_and(|action| action.contains("제거만")));
        Ok(())
    }

    #[test]
    fn materializes_excluded_and_unprocessed_without_review_sink() -> Result<()> {
        let artifact = build_dead_classify_artifact(request(Vec::new()))?.0;
        assert_eq!(artifact["excludedCandidates"][0]["reason"], "config_FP22");
        assert_eq!(
            artifact["proposal_DEGRADED_unprocessed"][0]["reason"],
            "classify-max-file-bytes"
        );
        assert_eq!(
            artifact["proposal_B_review"].as_array().map_or(0, Vec::len),
            0
        );
        assert_eq!(artifact["summary"]["incomplete"], true);
        Ok(())
    }

    #[test]
    fn rejects_unknown_schema() {
        let mut request = request(Vec::new());
        request.schema_version = "bad".to_string();
        let error = match build_dead_classify_artifact(request) {
            Ok(_) => panic!("schema should reject"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("unsupported schemaVersion"));
    }

    #[test]
    fn rejects_candidate_without_count_evidence() {
        let error = match build_dead_classify_artifact(request(vec![json!({
            "file": "src/missing.ts",
            "line": 1,
            "symbol": "Missing",
            "kind": "FunctionDeclaration"
        })])) {
            Ok(_) => panic!("missing count evidence should reject"),
            Err(error) => error,
        };
        assert!(error
            .to_string()
            .contains("missing numeric fileInternalUses"));
    }
}
