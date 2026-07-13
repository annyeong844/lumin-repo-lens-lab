use anyhow::{Context, Result};
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

const NEAR_NAME_MAX_LENGTH_DELTA: usize = 2;
const NEAR_NAME_SHARED_PREFIX_MIN: usize = 4;
const NEAR_NAME_MAX_DISTANCE: usize = 2;
const RESULT_CAP: usize = 5;
const SEMANTIC_HINT_MIN_SCORE: usize = 2;
const DEPENDENCY_HUB_THRESHOLD: usize = 10;

const SEMANTIC_STOP_TOKENS: &[&str] = &[
    "a", "an", "and", "as", "at", "by", "for", "from", "in", "into", "of", "on", "or", "the",
    "this", "that", "to", "with", "add", "new", "helper", "function", "type", "file", "module",
    "service", "manager", "index", "main", "src", "lib", "utils", "util", "ts", "js", "mjs", "cjs",
    "tsx", "jsx",
];

const WEAK_COMMON_TOKENS: &[&str] = &[
    "action",
    "adapter",
    "api",
    "app",
    "application",
    "client",
    "command",
    "config",
    "context",
    "data",
    "domain",
    "event",
    "factory",
    "handler",
    "item",
    "manager",
    "model",
    "module",
    "option",
    "provider",
    "request",
    "response",
    "result",
    "service",
    "state",
    "store",
    "type",
    "util",
    "value",
];

#[derive(Debug, Clone)]
struct CanonicalClaim {
    name: String,
    owner_file: String,
    line: usize,
    file: String,
    section: String,
}

#[derive(Debug, Clone)]
struct SearchCandidate {
    name: String,
    owner_file: String,
    matched_field: &'static str,
    identity: Option<String>,
    definition_kind: Option<String>,
    class_name: Option<String>,
    member_kind: Option<String>,
    visibility: Option<String>,
    static_member: bool,
    line: Option<u64>,
}

pub(super) struct LookupProjection {
    pub(super) lookups: Vec<Value>,
}

pub(super) fn project(
    root: &Path,
    intent: &Value,
    evidence: &Value,
    failures: &mut Vec<Value>,
) -> Result<LookupProjection> {
    let symbols = evidence.get("symbols").unwrap_or(&Value::Null);
    let topology = evidence.get("topology").unwrap_or(&Value::Null);
    let shape_index = evidence.get("shapeIndex").unwrap_or(&Value::Null);
    let shape_normalizations = evidence
        .get("shapeIntentNormalizations")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let function_signatures = evidence.get("functionSignatures").unwrap_or(&Value::Null);
    let inline_patterns = evidence.get("inlinePatterns").unwrap_or(&Value::Null);
    let claims = load_canonical_claims(root)?;
    let mut lookups = Vec::new();

    for name in string_array(intent, "names") {
        let declaration = intent
            .get("nameDeclarations")
            .and_then(Value::as_array)
            .and_then(|values| {
                values
                    .iter()
                    .find(|entry| entry.get("name").and_then(Value::as_str) == Some(name))
            });
        lookups.push(lookup_name(name, symbols, &claims, declaration));
    }
    for file in string_array(intent, "files") {
        lookups.push(lookup_file(file, topology, symbols, root));
    }

    let package_json = read_package_json(root, failures)?;
    for dependency in string_array(intent, "dependencies") {
        lookups.push(lookup_dependency(dependency, &package_json, symbols));
    }
    for shape in intent
        .get("shapes")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
    {
        lookups.push(lookup_shape(
            shape,
            shape_index,
            shape_normalizations,
            function_signatures,
        ));
    }
    if let Some(refactor_sources) = intent.get("refactorSources").and_then(Value::as_array) {
        if !refactor_sources.is_empty() {
            lookups.push(lookup_inline_patterns(
                refactor_sources,
                inline_patterns,
                evidence.get("files").unwrap_or(&Value::Null),
            ));
        }
    }

    Ok(LookupProjection { lookups })
}

fn string_array<'a>(value: &'a Value, key: &str) -> impl Iterator<Item = &'a str> {
    value
        .get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
}

fn read_package_json(root: &Path, failures: &mut Vec<Value>) -> Result<Value> {
    let path = root.join("package.json");
    if !path.exists() {
        return Ok(json!({}));
    }
    let text = fs::read_to_string(&path)
        .with_context(|| format!("pre-write: failed to read {}", path.display()))?;
    match serde_json::from_str(&text) {
        Ok(value) => Ok(value),
        Err(error) => {
            failures.push(json!({
                "kind": "package-json-parse-error",
                "reason": error.to_string(),
            }));
            Ok(json!({}))
        }
    }
}

fn load_canonical_claims(root: &Path) -> Result<Vec<CanonicalClaim>> {
    let path = root.join("canonical").join("type-ownership.md");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(&path)
        .with_context(|| format!("pre-write: failed to read {}", path.display()))?;
    let mut claims = Vec::new();
    let mut section = "Type ownership table".to_string();
    for (index, line) in text.lines().enumerate() {
        if let Some(title) = line.strip_prefix("### ") {
            section = title
                .trim_start_matches(|ch: char| ch.is_ascii_digit() || ch == '.')
                .trim()
                .to_string();
            continue;
        }
        let cells = line
            .trim()
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect::<Vec<_>>();
        if cells.len() < 2 || !cells[0].starts_with('`') || !cells[1].starts_with('`') {
            continue;
        }
        let name = cells[0].trim_matches('`');
        let owner_file = cells[1].trim_matches('`');
        if name.is_empty()
            || owner_file.is_empty()
            || !owner_file.contains('/')
            || name.eq_ignore_ascii_case("type")
        {
            continue;
        }
        claims.push(CanonicalClaim {
            name: name.to_string(),
            owner_file: owner_file.replace('\\', "/"),
            line: index + 1,
            file: path.to_string_lossy().to_string(),
            section: section.clone(),
        });
    }
    Ok(claims)
}

fn canonical_claim_value(claim: &CanonicalClaim) -> Value {
    json!({
        "name": claim.name,
        "ownerFile": claim.owner_file,
        "line": claim.line,
        "file": claim.file,
        "section": claim.section,
    })
}

fn lookup_name(
    intent_name: &str,
    symbols: &Value,
    claims: &[CanonicalClaim],
    declaration: Option<&Value>,
) -> Value {
    let supports = symbols.pointer("/meta/supports").unwrap_or(&Value::Null);
    let def_index = symbols.get("defIndex").and_then(Value::as_object);
    let canonical_claim = claims.iter().find(|claim| claim.name == intent_name);
    let mut citations = Vec::new();
    let mut identities = Vec::new();
    if let Some(def_index) = def_index {
        for (owner_file, definitions) in def_index {
            let Some(definition) = definitions.get(intent_name) else {
                continue;
            };
            let identity = format!("{owner_file}::{intent_name}");
            let (fan_in, fan_in_confidence, fan_in_citation) = fan_in(symbols, &identity);
            let (fan_space, fan_space_confidence, fan_space_citation) =
                fan_in_space(symbols, &identity);
            let (contamination, contamination_citation) = contamination(definition, supports);
            let (resolver_confidence, resolver_citation) = resolver_confidence(owner_file, symbols);
            citations.extend([
                fan_in_citation.clone(),
                fan_space_citation.clone(),
                contamination_citation.clone(),
            ]);
            if let Some(citation) = &resolver_citation {
                citations.push(citation.clone());
            }
            let identity_citations = [
                Some(fan_in_citation),
                Some(fan_space_citation),
                Some(contamination_citation),
                resolver_citation,
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
            identities.push(json!({
                "identity": identity,
                "ownerFile": owner_file,
                "exportedName": intent_name,
                "fanIn": fan_in,
                "fanInConfidence": fan_in_confidence,
                "fanInSpace": fan_space,
                "fanInSpaceConfidence": fan_space_confidence,
                "anyContamination": contamination,
                "resolverConfidence": resolver_confidence,
                "citations": identity_citations,
            }));
        }
    }
    identities
        .sort_by(|left, right| string_at(left, "ownerFile").cmp(string_at(right, "ownerFile")));

    let (result, canonical_status) = match (canonical_claim, identities.len()) {
        (None, 0) => ("NOT_OBSERVED", "not-consulted"),
        (None, 1) => ("EXISTS", "not-consulted"),
        (None, _) => ("EXISTS_MULTIPLE", "not-consulted"),
        (Some(_), 0) => ("CANONICAL_EXISTS_AST_ABSENT", "ast-absent"),
        (Some(claim), _)
            if identities
                .iter()
                .any(|row| row["ownerFile"] == claim.owner_file) =>
        {
            ("CANONICAL_EXISTS_AND_EXISTS", "aligned")
        }
        (Some(_), _) => ("CANONICAL_EXISTS_AST_DISAGREE", "owner-disagrees"),
    };
    if let Some(claim) = canonical_claim {
        citations.push(format!(
            "[grounded, canonical/{}:L{} declares owner '{}' for '{}']",
            Path::new(&claim.file)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("type-ownership.md"),
            claim.line,
            claim.owner_file,
            intent_name
        ));
    }

    let candidates = if identities.is_empty() {
        search_candidates(symbols)
    } else {
        Vec::new()
    };
    let owner_hint = declaration
        .and_then(|value| {
            value
                .get("ownerFile")
                .or_else(|| value.get("file"))
                .or_else(|| value.get("targetFile"))
        })
        .and_then(Value::as_str);
    let (near_names, suppressed_near, suppressed_near_count) =
        near_name_candidates(intent_name, owner_hint, &candidates);
    let intent_tokens = unique_tokens(&[
        Some(intent_name),
        declaration
            .and_then(|value| value.get("kind"))
            .and_then(Value::as_str),
        declaration
            .and_then(|value| value.get("why"))
            .and_then(Value::as_str),
    ]);
    let (semantic_hints, suppressed_semantic, suppressed_semantic_count) =
        semantic_candidates(&intent_tokens, owner_hint, &candidates);
    let service_operation_policy =
        service_operation_policy(intent_name, &suppressed_near, &suppressed_semantic);
    if !near_names.is_empty() {
        citations.push("[degraded, fuzzy-name match; source: symbols.json.defIndex/classMethodIndex name scan — search hint only, NOT a grounded reuse claim]".to_string());
    }
    if !semantic_hints.is_empty() {
        citations.push("[degraded, intent-token match; source: symbols.json.defIndex/classMethodIndex plus intent.name/intent.why tokens — search hint only, NOT a grounded reuse claim]".to_string());
    }
    if identities.is_empty()
        && supports.get("classMethodIndex").and_then(Value::as_bool) != Some(true)
    {
        citations.push("[확인 불가, reason: symbols.meta.supports.classMethodIndex is not true; class-method search unavailable]".to_string());
    }
    if identities.is_empty()
        && canonical_claim.is_none()
        && near_names.is_empty()
        && semantic_hints.is_empty()
    {
        citations.push(format!("[확인 불가, scan range: symbols.json.defIndex/classMethodIndex does not contain '{intent_name}'; no near-name or intent-token candidates either]"));
    }

    json!({
        "kind": "name",
        "intentName": intent_name,
        "result": result,
        "identities": identities,
        "canonicalClaim": canonical_claim.map(canonical_claim_value),
        "canonicalAstStatus": canonical_status,
        "intentTokens": intent_tokens,
        "nearNames": near_names,
        "semanticHints": semantic_hints,
        "suppressedNearNames": suppressed_near,
        "suppressedNearNameCount": suppressed_near_count,
        "suppressedSemanticHints": suppressed_semantic,
        "suppressedSemanticHintCount": suppressed_semantic_count,
        "serviceOperationSiblingPolicy": service_operation_policy,
        "localOperationSiblingPolicy": local_operation_policy(intent_name, owner_hint, symbols),
        "citations": citations,
    })
}

fn fan_in(symbols: &Value, identity: &str) -> (Value, &'static str, String) {
    if symbols
        .pointer("/meta/supports/identityFanIn")
        .and_then(Value::as_bool)
        != Some(true)
    {
        return (
            Value::Null,
            "unavailable",
            "[확인 불가, reason: symbols.meta.supports.identityFanIn is not true; identity fan-in not emitted by this producer]".to_string(),
        );
    }
    if let Some(value) = symbols.pointer(&format!(
        "/fanInByIdentity/{}",
        json_pointer_escape(identity)
    )) {
        return (
            value.clone(),
            "grounded",
            format!("[grounded, symbols.json.fanInByIdentity['{identity}'] = {value}]"),
        );
    }
    (
        Value::Null,
        "unavailable",
        format!("[확인 불가, reason: supports.identityFanIn=true but fanInByIdentity['{identity}'] is absent — producer contract violation. symbols.topSymbolFanIn is name-keyed and MUST NOT be substituted]"),
    )
}

fn fan_in_space(symbols: &Value, identity: &str) -> (Value, &'static str, String) {
    if symbols
        .pointer("/meta/supports/identityFanInSpace")
        .and_then(Value::as_bool)
        != Some(true)
    {
        return (
            Value::Null,
            "unavailable",
            "[확인 불가, reason: symbols.meta.supports.identityFanInSpace is not true; type/value fan-in breakdown not emitted by this producer]".to_string(),
        );
    }
    if let Some(record) = symbols.pointer(&format!(
        "/fanInByIdentitySpace/{}",
        json_pointer_escape(identity)
    )) {
        let normalized = json!({
            "value": record.get("value").and_then(Value::as_u64).unwrap_or(0),
            "type": record.get("type").and_then(Value::as_u64).unwrap_or(0),
            "broad": record.get("broad").and_then(Value::as_u64).unwrap_or(0),
        });
        return (
            normalized.clone(),
            "grounded",
            format!("[grounded, symbols.json.fanInByIdentitySpace['{identity}'] = {normalized}]"),
        );
    }
    (
        Value::Null,
        "unavailable",
        format!("[확인 불가, reason: supports.identityFanInSpace=true but fanInByIdentitySpace['{identity}'] is absent — producer contract violation]"),
    )
}

fn contamination(definition: &Value, supports: &Value) -> (Value, String) {
    if supports.get("anyContamination").and_then(Value::as_bool) != Some(true) {
        return (
            json!({ "state": "capability-absent" }),
            "[확인 불가, reason: producer did not emit anyContamination capability (symbols.meta.supports.anyContamination !== true)]".to_string(),
        );
    }
    let Some(annotation) = definition.get("anyContamination") else {
        return (
            json!({ "state": "clean" }),
            "[grounded, anyContamination annotation absent → clean]".to_string(),
        );
    };
    let labels = annotation
        .get("labels")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let has = |label: &str| labels.iter().any(|value| value.as_str() == Some(label));
    let state = if has("severely-any-contaminated") {
        "severely-any-contaminated"
    } else if has("any-contaminated") {
        "any-contaminated"
    } else if has("has-any") {
        "has-any-only"
    } else if has("unknown-surface") {
        "unknown-surface-only"
    } else {
        "clean"
    };
    let mut result = json!({
        "state": state,
        "labels": labels,
        "measurements": annotation.get("measurements").cloned().unwrap_or(Value::Null),
    });
    if matches!(state, "severely-any-contaminated" | "any-contaminated") {
        result["recommendation"] = json!({
            "action": "warn-on-reuse",
            "confidence": "low",
            "reason": format!("{state} semantic reuse caution"),
        });
    }
    let citation = if state == "clean" {
        format!("[확인 불가, reason: anyContamination annotation present but labels[] empty or unrecognized: {}]", Value::Array(labels))
    } else {
        format!(
            "[grounded, anyContamination.label = '{state}', measurements = {}]",
            annotation
                .get("measurements")
                .cloned()
                .unwrap_or_else(|| json!({}))
        )
    };
    (result, citation)
}

fn resolver_confidence(owner_file: &str, symbols: &Value) -> (&'static str, Option<String>) {
    if symbols
        .get("filesWithParseErrors")
        .and_then(Value::as_array)
        .is_some_and(|files| files.iter().any(|file| file.as_str() == Some(owner_file)))
    {
        return (
            "low",
            Some(format!("[degraded, resolver-confidence: low, taints: [\"defining-file-parse-error: '{owner_file}'\"]]")),
        );
    }
    let matching = symbols
        .get("unresolvedInternalSpecifiers")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .find(|specifier| specifier_could_match_file(specifier, owner_file));
    match matching {
        Some(specifier) => (
            "medium",
            Some(format!("[degraded, resolver-confidence: medium, taints: [\"unresolved-specifier-could-match: '{specifier}' ↔ '{owner_file}'\"]]")),
        ),
        None => ("high", None),
    }
}

fn specifier_could_match_file(specifier: &str, owner_file: &str) -> bool {
    if !specifier.starts_with('.') {
        return false;
    }
    let spec = specifier
        .trim_start_matches("./")
        .trim_end_matches(['s', 'x', 'j', 't', 'm', 'c', '.']);
    let owner = owner_file.replace('\\', "/");
    owner.contains(spec) || owner.ends_with(&format!("/{spec}"))
}

fn search_candidates(symbols: &Value) -> Vec<SearchCandidate> {
    let mut candidates = Vec::new();
    if let Some(files) = symbols.get("defIndex").and_then(Value::as_object) {
        for (file, definitions) in files {
            let Some(definitions) = definitions.as_object() else {
                continue;
            };
            for (name, definition) in definitions {
                candidates.push(SearchCandidate {
                    name: name.clone(),
                    owner_file: file.clone(),
                    matched_field: "defIndex",
                    identity: Some(format!("{file}::{name}")),
                    definition_kind: definition
                        .get("kind")
                        .and_then(Value::as_str)
                        .map(str::to_string),
                    class_name: None,
                    member_kind: None,
                    visibility: None,
                    static_member: false,
                    line: definition.get("line").and_then(Value::as_u64),
                });
            }
        }
    }
    if let Some(files) = symbols.get("classMethodIndex").and_then(Value::as_object) {
        for (file, methods) in files {
            let Some(methods) = methods.as_object() else {
                continue;
            };
            for (indexed_name, records) in methods {
                for record in records.as_array().into_iter().flatten() {
                    let name = record
                        .get("name")
                        .or_else(|| record.get("methodName"))
                        .and_then(Value::as_str)
                        .unwrap_or(indexed_name);
                    candidates.push(SearchCandidate {
                        name: name.to_string(),
                        owner_file: record
                            .get("ownerFile")
                            .and_then(Value::as_str)
                            .unwrap_or(file)
                            .to_string(),
                        matched_field: "classMethodIndex",
                        identity: record
                            .get("identity")
                            .and_then(Value::as_str)
                            .map(str::to_string),
                        definition_kind: None,
                        class_name: optional_string(record, "className"),
                        member_kind: optional_string(record, "memberKind"),
                        visibility: optional_string(record, "visibility"),
                        static_member: record.get("static").and_then(Value::as_bool) == Some(true),
                        line: record.get("line").and_then(Value::as_u64),
                    });
                }
            }
        }
    }
    candidates
}

fn near_name_candidates(
    intent_name: &str,
    owner_hint: Option<&str>,
    candidates: &[SearchCandidate],
) -> (Vec<Value>, Vec<Value>, usize) {
    let mut matches = Vec::<(usize, Value)>::new();
    let mut suppressed = Vec::<(usize, usize, Value)>::new();
    for candidate in candidates {
        if candidate.name == intent_name && candidate.matched_field != "classMethodIndex" {
            continue;
        }
        let matched_tokens = common_tokens(intent_name, &candidate.name);
        let locality = locality(candidate, owner_hint);
        if !matched_tokens.is_empty() && matched_tokens.iter().all(|token| is_weak_token(token)) {
            let mut value = candidate_value(candidate);
            extend_object(
                &mut value,
                json!({
                    "matchedTokens": matched_tokens,
                    "reason": "domain-token-overlap",
                    "locality": locality,
                }),
            );
            suppressed.push((locality_rank(&value), usize::MAX, value));
            continue;
        }
        let prefix = shared_prefix(&candidate.name, intent_name);
        let length_delta = candidate.name.len().abs_diff(intent_name.len());
        if prefix >= NEAR_NAME_SHARED_PREFIX_MIN && length_delta <= intent_name.len() {
            let distance =
                levenshtein_capped(&candidate.name, intent_name, NEAR_NAME_MAX_DISTANCE * 4);
            let mut value = candidate_value(candidate);
            extend_object(&mut value, json!({ "distance": distance }));
            matches.push((distance, value));
            continue;
        }
        if length_delta > NEAR_NAME_MAX_LENGTH_DELTA {
            if !matched_tokens.is_empty() || prefix >= NEAR_NAME_SHARED_PREFIX_MIN {
                let mut value = candidate_value(candidate);
                extend_object(
                    &mut value,
                    json!({
                        "matchedTokens": matched_tokens,
                        "lengthDelta": length_delta,
                        "reason": "near-length-delta-exceeded",
                        "locality": locality,
                    }),
                );
                suppressed.push((locality_rank(&value), length_delta, value));
            }
            continue;
        }
        let distance = levenshtein_capped(&candidate.name, intent_name, NEAR_NAME_MAX_DISTANCE);
        if distance <= NEAR_NAME_MAX_DISTANCE {
            let mut value = candidate_value(candidate);
            extend_object(&mut value, json!({ "distance": distance }));
            matches.push((distance, value));
        } else if !matched_tokens.is_empty() || prefix >= NEAR_NAME_SHARED_PREFIX_MIN {
            let mut value = candidate_value(candidate);
            extend_object(
                &mut value,
                json!({
                    "matchedTokens": matched_tokens,
                    "distance": distance,
                    "reason": "near-distance-exceeded",
                    "locality": locality,
                }),
            );
            suppressed.push((locality_rank(&value), distance, value));
        }
    }
    matches.sort_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then_with(|| {
                string_at(&left.1, "matchedField").cmp(string_at(&right.1, "matchedField"))
            })
            .then_with(|| string_at(&left.1, "name").cmp(string_at(&right.1, "name")))
            .then_with(|| string_at(&left.1, "ownerFile").cmp(string_at(&right.1, "ownerFile")))
    });
    suppressed.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| left.1.cmp(&right.1))
            .then_with(|| string_at(&left.2, "name").cmp(string_at(&right.2, "name")))
            .then_with(|| string_at(&left.2, "ownerFile").cmp(string_at(&right.2, "ownerFile")))
    });
    let suppressed_count = suppressed.len();
    let capped_suppressed = suppressed
        .into_iter()
        .take(RESULT_CAP)
        .map(|(_, _, mut value)| {
            value["candidateCount"] = json!(suppressed_count);
            value
        })
        .collect();
    (
        matches
            .into_iter()
            .take(RESULT_CAP)
            .map(|(_, value)| value)
            .collect(),
        capped_suppressed,
        suppressed_count,
    )
}

fn semantic_candidates(
    query_tokens: &[String],
    owner_hint: Option<&str>,
    candidates: &[SearchCandidate],
) -> (Vec<Value>, Vec<Value>, usize) {
    let query = query_tokens.iter().cloned().collect::<BTreeSet<_>>();
    let mut matches = Vec::new();
    let mut suppressed = Vec::new();
    for candidate in candidates {
        let name_tokens = unique_tokens(&[Some(candidate.name.as_str())]);
        let support_tokens = unique_tokens(&[
            candidate.definition_kind.as_deref(),
            candidate.class_name.as_deref(),
            candidate.member_kind.as_deref(),
        ]);
        let candidate_tokens = name_tokens
            .iter()
            .chain(&support_tokens)
            .cloned()
            .collect::<BTreeSet<_>>();
        let matched = candidate_tokens
            .intersection(&query)
            .cloned()
            .collect::<Vec<_>>();
        if matched.is_empty() {
            continue;
        }
        let matched_name = name_tokens
            .iter()
            .filter(|token| query.contains(*token))
            .cloned()
            .collect::<Vec<_>>();
        let strong_name = matched_name
            .iter()
            .filter(|token| !is_weak_token(token))
            .cloned()
            .collect::<Vec<_>>();
        let strong_support = support_tokens
            .iter()
            .filter(|token| {
                query.contains(*token) && !is_weak_token(token) && !strong_name.contains(token)
            })
            .cloned()
            .collect::<Vec<_>>();
        let mut value = candidate_value(candidate);
        let score = matched.len();
        extend_object(
            &mut value,
            json!({
                "matchedTokens": matched,
                "matchedNameTokens": matched_name,
                "matchedSupportTokens": strong_support,
                "score": score,
                "locality": locality(candidate, owner_hint),
            }),
        );
        if score < SEMANTIC_HINT_MIN_SCORE
            || !(strong_name.len() >= 2 || (strong_name.len() == 1 && !strong_support.is_empty()))
        {
            let reason = if value["matchedTokens"].as_array().is_some_and(|tokens| {
                tokens
                    .iter()
                    .all(|token| token.as_str().is_some_and(is_weak_token))
            }) {
                "domain-token-overlap"
            } else if score < SEMANTIC_HINT_MIN_SCORE {
                "single-non-weak-token-only"
            } else {
                "insufficient-non-weak-support"
            };
            value["reason"] = json!(reason);
            suppressed.push(value);
        } else {
            matches.push(value);
        }
    }
    let sort_values = |values: &mut Vec<Value>| {
        values.sort_by(|left, right| {
            locality_rank(right)
                .cmp(&locality_rank(left))
                .then_with(|| {
                    right
                        .get("score")
                        .and_then(Value::as_u64)
                        .unwrap_or(0)
                        .cmp(&left.get("score").and_then(Value::as_u64).unwrap_or(0))
                })
                .then_with(|| string_at(left, "name").cmp(string_at(right, "name")))
                .then_with(|| string_at(left, "ownerFile").cmp(string_at(right, "ownerFile")))
        });
    };
    sort_values(&mut matches);
    sort_values(&mut suppressed);
    let suppressed_count = suppressed.len();
    for value in &mut suppressed {
        value["candidateCount"] = json!(suppressed_count);
    }
    (
        matches.into_iter().take(RESULT_CAP).collect(),
        suppressed.into_iter().take(RESULT_CAP).collect(),
        suppressed_count,
    )
}

fn candidate_value(candidate: &SearchCandidate) -> Value {
    let mut object = Map::new();
    object.insert("name".to_string(), json!(candidate.name));
    object.insert("ownerFile".to_string(), json!(candidate.owner_file));
    object.insert("matchedField".to_string(), json!(candidate.matched_field));
    insert_option(&mut object, "identity", candidate.identity.as_deref());
    insert_option(
        &mut object,
        "definitionKind",
        candidate.definition_kind.as_deref(),
    );
    insert_option(&mut object, "className", candidate.class_name.as_deref());
    insert_option(&mut object, "memberKind", candidate.member_kind.as_deref());
    insert_option(&mut object, "visibility", candidate.visibility.as_deref());
    if candidate.matched_field == "classMethodIndex" {
        object.insert("exportedName".to_string(), json!(candidate.name));
    }
    if candidate.static_member {
        object.insert("static".to_string(), json!(true));
    }
    if let Some(line) = candidate.line {
        object.insert("line".to_string(), json!(line));
    }
    Value::Object(object)
}

fn local_operation_policy(intent_name: &str, owner_hint: Option<&str>, symbols: &Value) -> Value {
    let Some(index) = symbols.get("preWriteLocalOperationIndex") else {
        return empty_local_policy("not-run", Some("pre-write-local-operation-index-missing"));
    };
    if index.get("status").and_then(Value::as_str) != Some("complete") {
        return empty_local_policy(
            index
                .get("status")
                .and_then(Value::as_str)
                .unwrap_or("unavailable"),
            index.get("reason").and_then(Value::as_str),
        );
    }
    let Some(owner_hint) = owner_hint else {
        return empty_local_policy("complete", Some("intent-owner-file-missing"));
    };
    let entries = index
        .pointer(&format!("/byOwnerFile/{}", json_pointer_escape(owner_hint)))
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let intent_operation = operation_info(intent_name);
    let mut promoted = Vec::new();
    let mut muted = Vec::new();
    for entry in entries {
        let name = entry.get("name").and_then(Value::as_str).unwrap_or("");
        let candidate_operation = operation_info(name);
        let shared = intent_operation
            .1
            .intersection(&candidate_operation.1)
            .cloned()
            .collect::<Vec<_>>();
        let same_file = entry.get("ownerFile").and_then(Value::as_str) == Some(owner_hint);
        let reason = if entry.get("identity").and_then(Value::as_str).is_none()
            || name.is_empty()
            || entry.get("ownerFile").and_then(Value::as_str).is_none()
        {
            Some("local-operation-insufficient-metadata")
        } else if !same_file {
            Some("local-operation-locality-mismatch")
        } else if intent_operation.0.is_none() || candidate_operation.0.is_none() {
            Some("local-operation-unknown-operation")
        } else if shared.is_empty() {
            Some("local-operation-domain-mismatch")
        } else if intent_operation.0 != candidate_operation.0 {
            Some("local-operation-family-mismatch")
        } else if intent_operation.0.as_deref() != Some("read-query") {
            Some("local-operation-family-not-promotable")
        } else {
            None
        };
        let mut projected = json!({
            "identity": entry.get("identity").cloned().unwrap_or(Value::Null),
            "name": name,
            "ownerFile": entry.get("ownerFile").cloned().unwrap_or(Value::Null),
            "matchedField": "preWriteLocalOperationIndex",
            "surfaceKind": "nested-local-operation",
            "operationFamily": candidate_operation.0,
            "sharedDomainTokens": shared,
            "locality": { "sameDir": same_file, "sameFile": same_file },
            "eligibleForDeadExportRanking": entry.get("eligibleForDeadExportRanking").and_then(Value::as_bool) == Some(true),
            "eligibleForSafeFix": entry.get("eligibleForSafeFix").and_then(Value::as_bool) == Some(true),
            "signatureSupport": { "status": "unavailable", "reason": "no-signature-facts" },
        });
        for field in [
            "containerName",
            "containerKind",
            "line",
            "containerLine",
            "domainTokens",
        ] {
            if let Some(value) = entry.get(field) {
                projected[field] = value.clone();
            }
        }
        if let Some(reason) = reason {
            projected["reason"] = json!(reason);
            muted.push(projected);
        } else {
            projected["supportingReasons"] = json!(["local-operation-same-file-domain-overlap"]);
            promoted.push(projected);
        }
    }
    sort_policy_entries(&mut promoted);
    sort_policy_entries(&mut muted);
    json!({
        "policyId": "prewrite-local-operation-sibling",
        "policyVersion": "prewrite-local-operation-sibling-v1",
        "status": "complete",
        "evaluatedCandidateCount": promoted.len() + muted.len(),
        "promotedCandidateCount": promoted.len(),
        "mutedCandidateCount": muted.len(),
        "promoted": promoted.into_iter().take(RESULT_CAP).collect::<Vec<_>>(),
        "muted": muted.into_iter().take(RESULT_CAP).collect::<Vec<_>>(),
    })
}

fn service_operation_policy(
    intent_name: &str,
    suppressed_near: &[Value],
    suppressed_semantic: &[Value],
) -> Value {
    let mut merged = BTreeMap::<String, Value>::new();
    for (entries, lane) in [
        (suppressed_near, "near-name"),
        (suppressed_semantic, "semantic"),
    ] {
        for entry in entries {
            let identity = entry
                .get("identity")
                .and_then(Value::as_str)
                .map(str::to_string)
                .or_else(|| {
                    Some(format!(
                        "{}::{}",
                        entry.get("ownerFile")?.as_str()?,
                        entry.get("name")?.as_str()?
                    ))
                });
            let Some(identity) = identity else {
                continue;
            };
            let candidate = merged.entry(identity.clone()).or_insert_with(|| {
                json!({
                    "identity": identity,
                    "name": entry.get("name").cloned().unwrap_or(Value::Null),
                    "ownerFile": entry.get("ownerFile").cloned().unwrap_or(Value::Null),
                    "matchedField": entry.get("matchedField").cloned().unwrap_or(Value::Null),
                    "definitionKind": entry.get("definitionKind").cloned().unwrap_or(Value::Null),
                    "locality": entry.get("locality").cloned().unwrap_or_else(|| json!({ "sameDir": false, "sameFile": false })),
                    "supportingReasons": [],
                    "matchedTokens": [],
                    "suppressedLanes": [],
                })
            });
            if locality_rank(entry) > locality_rank(candidate) {
                candidate["locality"] = entry
                    .get("locality")
                    .cloned()
                    .unwrap_or_else(|| candidate["locality"].clone());
            }
            push_unique_string(
                candidate,
                "supportingReasons",
                entry.get("reason").and_then(Value::as_str),
            );
            for token in entry
                .get("matchedTokens")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
            {
                push_unique_string(candidate, "matchedTokens", Some(token));
            }
            push_unique_string(candidate, "suppressedLanes", Some(lane));
            for field in ["distance", "lengthDelta", "score"] {
                if let Some(value) = entry.get(field) {
                    candidate[field] = value.clone();
                }
            }
        }
    }

    let intent_operation = operation_info(intent_name);
    let mut promoted = Vec::new();
    let mut muted = Vec::new();
    for mut candidate in merged.into_values() {
        sort_string_array(&mut candidate, "supportingReasons", supporting_reason_rank);
        sort_string_array(&mut candidate, "suppressedLanes", |_| 0);
        let candidate_name = candidate
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let candidate_owner = candidate
            .get("ownerFile")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let candidate_operation = operation_info(&candidate_name);
        let shared = intent_operation
            .1
            .intersection(&candidate_operation.1)
            .cloned()
            .collect::<Vec<_>>();
        candidate["operationFamily"] = candidate_operation
            .0
            .clone()
            .map_or(Value::Null, Value::String);
        candidate["sharedDomainTokens"] = json!(shared);
        candidate["signatureSupport"] =
            json!({ "status": "unavailable", "reason": "no-signature-facts" });

        let has_promotable_support = candidate
            .get("supportingReasons")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .any(|reason| {
                matches!(
                    reason,
                    "single-non-weak-token-only"
                        | "near-distance-exceeded"
                        | "near-length-delta-exceeded"
                )
            });
        let locality = candidate.get("locality").unwrap_or(&Value::Null);
        let matched_field = candidate.get("matchedField").and_then(Value::as_str);
        let definition_kind = candidate.get("definitionKind").and_then(Value::as_str);
        let reason = if candidate_name.is_empty()
            || candidate_owner.is_empty()
            || candidate.get("identity").and_then(Value::as_str).is_none()
        {
            Some("service-sibling-insufficient-metadata")
        } else if service_policy_excluded(&candidate_owner) {
            Some("service-sibling-policy-excluded")
        } else if matched_field.is_some_and(|field| field != "defIndex") {
            Some("service-sibling-surface-kind-unsupported")
        } else if definition_kind.is_some_and(is_non_callable_service_definition) {
            Some("service-sibling-non-callable-definition")
        } else if !has_promotable_support {
            Some("service-sibling-insufficient-suppressed-support")
        } else if locality.get("sameFile").and_then(Value::as_bool) != Some(true)
            && locality.get("sameDir").and_then(Value::as_bool) != Some(true)
        {
            Some("service-sibling-locality-mismatch")
        } else if intent_operation.0.is_none() || candidate_operation.0.is_none() {
            Some("service-sibling-unknown-operation")
        } else if intent_operation.1.is_empty()
            || candidate["sharedDomainTokens"]
                .as_array()
                .is_none_or(Vec::is_empty)
        {
            Some("service-sibling-domain-mismatch")
        } else if intent_operation.0 != candidate_operation.0 {
            Some("service-sibling-operation-family-mismatch")
        } else if intent_operation.0.as_deref() != Some("read-query") {
            Some("service-sibling-family-not-promotable")
        } else {
            None
        };
        if let Some(reason) = reason {
            candidate["reason"] = json!(reason);
            muted.push(candidate);
        } else {
            promoted.push(candidate);
        }
    }
    sort_policy_entries(&mut promoted);
    sort_policy_entries(&mut muted);
    json!({
        "policyId": "prewrite-service-operation-sibling-cue",
        "policyVersion": "prewrite-service-operation-sibling-cue-v1",
        "evaluatedCandidateCount": promoted.len() + muted.len(),
        "promotedCandidateCount": promoted.len(),
        "mutedCandidateCount": muted.len(),
        "promoted": promoted.into_iter().take(RESULT_CAP).collect::<Vec<_>>(),
        "muted": muted.into_iter().take(RESULT_CAP).collect::<Vec<_>>(),
    })
}

fn push_unique_string(value: &mut Value, field: &str, item: Option<&str>) {
    let Some(item) = item else {
        return;
    };
    let Some(items) = value.get_mut(field).and_then(Value::as_array_mut) else {
        return;
    };
    if !items.iter().any(|value| value.as_str() == Some(item)) {
        items.push(json!(item));
    }
}

fn sort_string_array(value: &mut Value, field: &str, rank: fn(&str) -> usize) {
    if let Some(items) = value.get_mut(field).and_then(Value::as_array_mut) {
        items.sort_by(|left, right| {
            let left = left.as_str().unwrap_or("");
            let right = right.as_str().unwrap_or("");
            rank(left).cmp(&rank(right)).then_with(|| left.cmp(right))
        });
    }
}

fn supporting_reason_rank(reason: &str) -> usize {
    match reason {
        "single-non-weak-token-only" => 0,
        "near-distance-exceeded" => 1,
        "near-length-delta-exceeded" => 2,
        "domain-token-overlap" => 3,
        _ => 10,
    }
}

fn service_policy_excluded(owner_file: &str) -> bool {
    let normalized = owner_file.replace('\\', "/");
    normalized.split('/').any(|segment| {
        matches!(
            segment,
            "__generated__"
                | "build"
                | "coverage"
                | "dist"
                | "generated"
                | "node_modules"
                | "vendor"
                | "vendors"
        )
    }) || normalized.contains(".bundle.")
        || normalized
            .rsplit_once('/')
            .map_or(normalized.starts_with("vendor."), |(_, file)| {
                file.starts_with("vendor.")
            })
}

fn is_non_callable_service_definition(kind: &str) -> bool {
    matches!(
        kind,
        "TSInterfaceDeclaration"
            | "TSTypeAliasDeclaration"
            | "TSEnumDeclaration"
            | "TSModuleDeclaration"
    )
}

fn empty_local_policy(status: &str, reason: Option<&str>) -> Value {
    let mut value = json!({
        "policyId": "prewrite-local-operation-sibling",
        "policyVersion": "prewrite-local-operation-sibling-v1",
        "status": status,
        "evaluatedCandidateCount": 0,
        "promotedCandidateCount": 0,
        "mutedCandidateCount": 0,
        "promoted": [],
        "muted": [],
    });
    if let Some(reason) = reason {
        value["reason"] = json!(reason);
    }
    value
}

fn operation_info(name: &str) -> (Option<String>, BTreeSet<String>) {
    let tokens = unique_tokens(&[Some(name)]);
    let verb = tokens.first().map(String::as_str);
    let family = match verb {
        Some(
            "fetch" | "find" | "get" | "list" | "load" | "lookup" | "query" | "read" | "resolve"
            | "retrieve" | "search",
        ) => Some("read-query".to_string()),
        Some("add" | "create") => Some("mutation-create".to_string()),
        Some("delete" | "destroy" | "remove") => Some("mutation-delete".to_string()),
        Some("dispatch" | "emit" | "send") => Some("mutation-send".to_string()),
        Some("patch" | "set" | "update") => Some("mutation-update".to_string()),
        Some("save" | "upsert" | "write") => Some("mutation-save".to_string()),
        _ => None,
    };
    let domain = tokens
        .into_iter()
        .skip(1)
        .filter(|token| !is_operation_verb(token))
        .map(|token| normalize_domain_token(&token))
        .filter(|token| !token.is_empty())
        .collect();
    (family, domain)
}

fn is_operation_verb(token: &str) -> bool {
    matches!(
        token,
        "fetch"
            | "find"
            | "get"
            | "list"
            | "load"
            | "lookup"
            | "query"
            | "read"
            | "resolve"
            | "retrieve"
            | "search"
            | "add"
            | "create"
            | "delete"
            | "destroy"
            | "dispatch"
            | "emit"
            | "patch"
            | "remove"
            | "save"
            | "send"
            | "set"
            | "update"
            | "upsert"
            | "write"
    )
}

fn lookup_file(intent_file: &str, topology: &Value, symbols: &Value, _root: &Path) -> Value {
    let intent_file = intent_file.replace('\\', "/");
    let in_topology = topology
        .get("nodes")
        .and_then(Value::as_object)
        .is_some_and(|nodes| nodes.contains_key(&intent_file));
    let in_def_index = symbols
        .get("defIndex")
        .and_then(Value::as_object)
        .is_some_and(|files| files.contains_key(&intent_file));
    let parse_error = symbols
        .get("filesWithParseErrors")
        .and_then(Value::as_array)
        .is_some_and(|files| files.iter().any(|file| file.as_str() == Some(&intent_file)));
    let complete = topology.pointer("/meta/complete").and_then(Value::as_bool) == Some(true);
    let edges = topology.get("edges").and_then(Value::as_array);
    let inbound = edges.map(|edges| {
        edges
            .iter()
            .filter(|edge| edge.get("to").and_then(Value::as_str) == Some(&intent_file))
            .count()
    });
    let loc = topology
        .pointer(&format!("/nodes/{}/loc", json_pointer_escape(&intent_file)))
        .cloned()
        .unwrap_or(Value::Null);
    let (result, mut citations) = if in_topology {
        (
            "FILE_EXISTS",
            vec![format!(
                "[grounded, topology.json.nodes['{intent_file}'] present{}]",
                if loc.is_null() {
                    String::new()
                } else {
                    format!(", loc = {loc}")
                }
            )],
        )
    } else if in_def_index {
        (
            "FILE_EXISTS",
            vec![format!("[grounded, symbols.json.defIndex['{intent_file}'] has declared exports — file exists even if topology absent]")],
        )
    } else if parse_error {
        (
            "FILE_STATUS_UNKNOWN",
            vec![format!("[확인 불가, reason: '{intent_file}' is in symbols.filesWithParseErrors — file exists on disk but failed to parse; topology.nodes enumeration is non-authoritative here]")],
        )
    } else if complete {
        (
            "NEW_FILE",
            vec![format!("[grounded, topology.json.nodes does not contain '{intent_file}'; topology.meta.complete = true; symbols.filesWithParseErrors does not list it]")],
        )
    } else if topology.is_object() {
        (
            "FILE_STATUS_UNKNOWN",
            vec!["[확인 불가, reason: topology present but topology.meta.complete is not true; absence-from-nodes is non-authoritative]".to_string()],
        )
    } else {
        (
            "FILE_STATUS_UNKNOWN",
            vec!["[확인 불가, reason: topology absent and symbols.defIndex has no entry; file existence cannot be grounded]".to_string()],
        )
    };
    if result == "FILE_EXISTS" {
        match inbound {
            Some(count) => citations.push(format!(
                "[grounded, topology.json.edges inbound count for '{intent_file}' = {count}]"
            )),
            None => citations.push(
                "[확인 불가, reason: topology absent — inbound fan-in not countable]".to_string(),
            ),
        }
    }
    citations.push("[확인 불가, reason: P1-2 intent carries no planned-edge endpoints; boundary rules can be consulted only when both endpoints are known]".to_string());
    let tags = if is_test_like_path(&intent_file) {
        vec!["test-only"]
    } else {
        Vec::new()
    };
    let submodule = submodule_for_file(&intent_file);
    let domain_cluster = domain_cluster(&intent_file, topology);
    json!({
        "kind": "file",
        "intentFile": intent_file,
        "result": result,
        "loc": loc,
        "inboundFanIn": inbound,
        "inboundFanInConfidence": if inbound.is_some() { "grounded" } else { "unavailable" },
        "submodule": submodule,
        "boundary": { "status": "NOT_EVALUATED", "rule": Value::Null },
        "tags": tags,
        "domainCluster": domain_cluster,
        "citations": citations,
    })
}

#[derive(Debug)]
struct DomainNode {
    file: String,
    loc: Option<u64>,
}

fn submodule_for_file(file: &str) -> String {
    let parts = file
        .split('/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.len() <= 1 {
        return "root".to_string();
    }
    if parts[0] == "src" && parts.len() > 2 {
        return parts[1].to_string();
    }
    if matches!(parts[0], "apps" | "packages") && parts.len() > 2 {
        return format!("{}/{}", parts[0], parts[1]);
    }
    parts[0].to_string()
}

fn domain_cluster(intent_file: &str, topology: &Value) -> Value {
    const MIN_MATCHES: usize = 2;
    const MAX_EXAMPLES: usize = 8;
    let Some(nodes) = topology.get("nodes").and_then(Value::as_object) else {
        return Value::Null;
    };
    let dir = dirname(intent_file);
    let same_dir = nodes
        .iter()
        .filter_map(|(file, info)| {
            let file = file.replace('\\', "/");
            (file != intent_file && dirname(&file) == dir).then(|| DomainNode {
                file,
                loc: info.get("loc").and_then(Value::as_u64),
            })
        })
        .collect::<Vec<_>>();
    if same_dir.is_empty() {
        return Value::Null;
    }

    for (display, key, token_count) in domain_prefix_candidates(intent_file) {
        let mut matches = same_dir
            .iter()
            .filter(|entry| {
                let base = strip_known_extension(basename(&entry.file));
                normalize_file_domain(&base).starts_with(&key)
                    || domain_token_keys(&base).contains(&key)
            })
            .collect::<Vec<_>>();
        matches.sort_by(|left, right| left.file.cmp(&right.file));
        let prefix_match_count = matches
            .iter()
            .filter(|entry| {
                normalize_file_domain(&strip_known_extension(basename(&entry.file)))
                    .starts_with(&key)
            })
            .count();
        let required_matches = if token_count >= 2 && prefix_match_count >= 1 {
            1
        } else {
            MIN_MATCHES
        };
        if matches.len() < required_matches {
            continue;
        }
        let loc_known = matches.iter().any(|entry| entry.loc.is_some());
        let total_loc = matches.iter().filter_map(|entry| entry.loc).sum::<u64>();
        let examples = matches
            .iter()
            .take(MAX_EXAMPLES)
            .map(|entry| json!({ "file": entry.file, "loc": entry.loc }))
            .collect::<Vec<_>>();
        let prefix_path = if dir == "." {
            display.clone()
        } else {
            format!("{dir}/{display}")
        };
        return json!({
            "kind": "DOMAIN_CLUSTER_DETECTED",
            "directory": dir,
            "basenamePrefix": display,
            "matchKind": if prefix_match_count == matches.len() { "prefix" } else { "domain-token" },
            "prefixPath": prefix_path,
            "matchCount": matches.len(),
            "totalLoc": if loc_known { json!(total_loc) } else { Value::Null },
            "examples": examples,
            "omittedCount": matches.len().saturating_sub(MAX_EXAMPLES),
            "citations": [format!("[grounded, topology.json.nodes matched {} files with domain key '{}' in '{}']", matches.len(), key, dir)],
        });
    }
    Value::Null
}

fn domain_prefix_candidates(intent_file: &str) -> Vec<(String, String, usize)> {
    const MIN_PREFIX_LEN: usize = 4;
    let base = strip_known_extension(basename(intent_file));
    let tokens = split_file_name_tokens(&base);
    let mut candidates = Vec::new();
    for count in (1..tokens.len()).rev() {
        let display = display_prefix(&tokens[..count]);
        let key = normalize_file_domain(&display);
        if key.len() >= MIN_PREFIX_LEN && !generic_domain_prefix(&key) {
            candidates.push((display, key, count));
        }
    }
    let whole_key = normalize_file_domain(&base);
    if whole_key.len() >= MIN_PREFIX_LEN
        && !generic_domain_prefix(&whole_key)
        && !candidates.iter().any(|(_, key, _)| key == &whole_key)
    {
        candidates.push((base, whole_key, tokens.len()));
    }
    candidates
}

fn domain_token_keys(file_name: &str) -> BTreeSet<String> {
    split_file_name_tokens(&strip_known_extension(file_name))
        .into_iter()
        .map(|token| normalize_file_domain(&token))
        .filter(|key| key.len() >= 4 && !generic_domain_prefix(key))
        .collect()
}

fn split_file_name_tokens(value: &str) -> Vec<String> {
    let chars = value.chars().collect::<Vec<_>>();
    let mut tokens = Vec::new();
    let mut current = String::new();
    for (index, ch) in chars.iter().copied().enumerate() {
        let previous = index.checked_sub(1).and_then(|i| chars.get(i)).copied();
        let next = chars.get(index + 1).copied();
        let separator = matches!(ch, '-' | '_' | '.' | ' ' | '\t' | '\r' | '\n');
        let camel_boundary = previous.is_some_and(|previous| {
            ch.is_ascii_uppercase()
                && (previous.is_ascii_lowercase()
                    || previous.is_ascii_digit()
                    || (previous.is_ascii_uppercase()
                        && next.is_some_and(|next| next.is_ascii_lowercase())))
        });
        if separator || camel_boundary {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
            if separator {
                continue;
            }
        }
        current.push(ch);
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

fn display_prefix(tokens: &[String]) -> String {
    let Some((first, rest)) = tokens.split_first() else {
        return String::new();
    };
    let mut value = first.clone();
    for token in rest {
        let mut chars = token.chars();
        if let Some(first) = chars.next() {
            value.push(first.to_ascii_uppercase());
            value.extend(chars);
        }
    }
    value
}

fn normalize_file_domain(value: &str) -> String {
    let mut raw = value
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .flat_map(char::to_lowercase)
        .collect::<String>();
    if raw.len() > 4 && raw.ends_with("ies") {
        raw.truncate(raw.len() - 3);
        raw.push('y');
    } else if raw.len() > 4 && raw.ends_with('s') {
        raw.pop();
    }
    raw
}

fn generic_domain_prefix(value: &str) -> bool {
    matches!(
        value,
        "index"
            | "main"
            | "test"
            | "tests"
            | "spec"
            | "helper"
            | "helpers"
            | "utils"
            | "util"
            | "types"
            | "type"
    )
}

fn strip_known_extension(file_name: &str) -> String {
    let lower = file_name.to_ascii_lowercase();
    for suffix in [
        ".d.mts", ".d.cts", ".d.ts", ".tsx", ".jsx", ".mjs", ".cjs", ".mts", ".cts", ".json",
        ".ts", ".js",
    ] {
        if lower.ends_with(suffix) {
            return file_name[..file_name.len() - suffix.len()].to_string();
        }
    }
    file_name.to_string()
}

fn basename(path: &str) -> &str {
    path.rsplit_once('/').map_or(path, |(_, name)| name)
}

fn lookup_dependency(dep_name: &str, package_json: &Value, symbols: &Value) -> Value {
    let dep_root = package_root(dep_name).unwrap_or(dep_name);
    let declaration = ["dependencies", "devDependencies", "peerDependencies"]
        .into_iter()
        .find_map(|bucket| {
            package_json
                .get(bucket)
                .and_then(Value::as_object)
                .and_then(|values| values.get(dep_root))
                .map(|version| (bucket, version))
        });
    let records = symbols
        .get("dependencyImportConsumers")
        .and_then(Value::as_array)
        .or_else(|| symbols.get("uses").and_then(Value::as_array));
    let unavailable_reason = if records.is_none() {
        if symbols
            .pointer("/meta/supports/dependencyImportConsumers")
            .and_then(Value::as_bool)
            == Some(true)
        {
            "symbols.json.dependencyImportConsumers absent or malformed"
        } else {
            "symbols.json.dependencyImportConsumers absent; producer did not emit dependencyImportConsumers capability"
        }
    } else {
        ""
    };
    let mut examples = Vec::new();
    let mut total = 0usize;
    if let Some(records) = records {
        for record in records {
            let Some(from_spec) = record.get("fromSpec").and_then(Value::as_str) else {
                continue;
            };
            if package_root(from_spec) == Some(dep_root) {
                total += 1;
                if examples.len() < 5 {
                    examples.push(json!({
                        "file": record.get("file").cloned().unwrap_or(Value::Null),
                        "fromSpec": from_spec,
                    }));
                }
            }
        }
    }
    let mut citations = Vec::new();
    let result = match (declaration, records) {
        (Some((bucket, version)), None) => {
            citations.push(format!(
                "[grounded, package.json.{bucket}['{dep_root}'] = '{version}']"
            ));
            citations.push(format!("[확인 불가, reason: {unavailable_reason}; observed static-import consumer count unavailable for '{dep_root}']"));
            "DEPENDENCY_AVAILABLE_IMPORT_GRAPH_UNAVAILABLE"
        }
        (Some((bucket, version)), Some(_)) if total > 0 => {
            citations.push(format!(
                "[grounded, package.json.{bucket}['{dep_root}'] = '{version}']"
            ));
            citations.push(format!("[grounded, symbols.json.dependencyImportConsumers fromSpec matches '{dep_root}' → {total} observed static-import consumer{}]", if total == 1 { "" } else { "s" }));
            "DEPENDENCY_AVAILABLE"
        }
        (Some((bucket, version)), Some(_)) => {
            citations.push(format!(
                "[grounded, package.json.{bucket}['{dep_root}'] = '{version}']"
            ));
            citations.push(format!("[확인 불가, scan range: import graph only — '{dep_root}' may still be consumed by scripts, config, runtime plugins, or build steps outside static imports]"));
            "DEPENDENCY_AVAILABLE_NO_OBSERVED_IMPORTS"
        }
        (None, _) => {
            citations.push(format!("[grounded, package.json.{{dependencies, devDependencies, peerDependencies}} does not contain '{dep_root}']"));
            "NEW_PACKAGE"
        }
    };
    json!({
        "kind": "dependency",
        "depName": dep_name,
        "declaredIn": declaration.map(|(bucket, _)| bucket),
        "result": result,
        "existingImports": {
            "examples": examples,
            "observedImportCount": if records.is_some() { json!(total) } else { Value::Null },
            "countConfidence": if records.is_some() { "grounded" } else { "unavailable" },
            "unavailableReason": if records.is_some() { Value::Null } else { json!(unavailable_reason) },
            "watchForEligible": records.is_some() && total >= DEPENDENCY_HUB_THRESHOLD,
        },
        "citations": citations,
    })
}

fn lookup_shape(
    shape: &Value,
    shape_index: &Value,
    normalizations: &[Value],
    function_signatures: &Value,
) -> Value {
    let type_literal = shape.get("typeLiteral").and_then(Value::as_str);
    let function_like = type_literal.is_some_and(|literal| {
        let literal = literal.trim_start();
        literal.starts_with('(') || literal.starts_with('<')
    });
    if function_like {
        let type_literal = shape.get("typeLiteral").and_then(Value::as_str);
        let normalized = type_literal.and_then(|literal| {
            normalizations
                .iter()
                .find(|entry| entry.get("typeLiteral").and_then(Value::as_str) == Some(literal))
        });
        if normalized
            .and_then(|entry| entry.get("ok"))
            .and_then(Value::as_bool)
            == Some(false)
        {
            return unavailable_shape(
                shape,
                "pre-write-evidence#functionSignatures",
                &format!(
                    "[확인 불가, function signature intent normalization failed; reason: {}]",
                    normalized
                        .and_then(|entry| entry.get("reason"))
                        .and_then(Value::as_str)
                        .unwrap_or("unsupported-function-signature")
                ),
                None,
            );
        }
        let supplied_hash = shape.get("hash").and_then(Value::as_str);
        let normalized_hash = normalized
            .and_then(|entry| entry.get("hash"))
            .and_then(Value::as_str);
        if supplied_hash.is_some() && normalized_hash.is_some() && supplied_hash != normalized_hash
        {
            return unavailable_shape(
                shape,
                "pre-write-evidence#functionSignatures",
                "[확인 불가, shape.hash does not match shape.typeLiteral normalized function-signature hash]",
                supplied_hash.map(|hash| (hash, "hash+typeLiteral:function-signature")),
            );
        }
        let Some(hash) = supplied_hash.or(normalized_hash) else {
            return unavailable_shape(
                shape,
                "pre-write-evidence#functionSignatures",
                "[확인 불가, native function-signature intent normalization has not produced a hash]",
                None,
            );
        };
        let matches = function_signatures
            .get("facts")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter(|fact| {
                fact.get("normalizedSignatureHash").and_then(Value::as_str) == Some(hash)
            })
            .map(|fact| {
                json!({
                    "identity": fact.get("identity").cloned().unwrap_or(Value::Null),
                    "ownerFile": fact.get("ownerFile").cloned().unwrap_or(Value::Null),
                    "exportedName": fact.get("exportedName").cloned().unwrap_or(Value::Null),
                    "localName": fact.get("localName").cloned().unwrap_or(Value::Null),
                    "visibility": fact.get("visibility").cloned().unwrap_or(json!("exported")),
                    "exported": fact.get("exported").and_then(Value::as_bool) != Some(false),
                    "hash": hash,
                    "signature": fact.get("signature").cloned().unwrap_or(Value::Null),
                    "confidence": fact.get("confidence").cloned().unwrap_or(json!("medium")),
                })
            })
            .collect::<Vec<_>>();
        let complete = function_signatures
            .pointer("/meta/complete")
            .and_then(Value::as_bool)
            == Some(true);
        if matches.is_empty() && !complete {
            return unavailable_shape(
                shape,
                "pre-write-evidence#functionSignatures",
                &format!("[확인 불가, current-run function signature evidence is incomplete; hash {hash} was not observed but absence is not grounded]"),
                Some((hash, "functionSignature")),
            );
        }
        let mut citations = vec![format!(
            "[grounded, current-run functionSignatures.facts[] matched {} identities for function signature {hash}]",
            matches.len()
        )];
        if !complete {
            citations.push("[degraded, current-run function signature evidence is incomplete; positive match remains grounded]".to_string());
        }
        return json!({
            "kind": "shape",
            "shape": shape,
            "shapeHash": hash,
            "shapeHashSource": "functionSignature",
            "signature": normalized.and_then(|entry| entry.get("signature")).cloned().unwrap_or(Value::Null),
            "result": if matches.is_empty() { "NOT_OBSERVED" } else { "SIGNATURE_MATCH" },
            "matches": matches,
            "citations": citations,
        });
    }

    let normalized = type_literal.and_then(|literal| {
        normalizations
            .iter()
            .find(|entry| entry.get("typeLiteral").and_then(Value::as_str) == Some(literal))
    });
    let hash = shape.get("hash").and_then(Value::as_str).or_else(|| {
        normalized
            .and_then(|entry| entry.get("hash"))
            .and_then(Value::as_str)
    });
    let Some(hash) = hash else {
        return unavailable_shape(
            shape,
            "shape-index.json",
            "[확인 불가, shape intent lacks exact sha256 shape hash or supported typeLiteral; field names alone are not structural equality evidence for P4 shape-hash lookup]",
            None,
        );
    };
    if !shape_index.is_object() {
        return unavailable_shape(
            shape,
            "shape-index.json",
            "[확인 불가, shape-index.json absent; run build-shape-index.mjs to enable P4 shape-hash lookup]",
            Some((hash, if type_literal.is_some() { "typeLiteral" } else { "hash" })),
        );
    }
    let matches = shape_index
        .get("facts")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|fact| fact.get("hash").and_then(Value::as_str) == Some(hash))
        .map(|fact| {
            json!({
                "identity": fact.get("identity").cloned().unwrap_or(Value::Null),
                "ownerFile": fact.get("ownerFile").cloned().unwrap_or(Value::Null),
                "exportedName": fact.get("exportedName").cloned().unwrap_or(Value::Null),
                "hash": hash,
                "shapeKind": fact.get("shapeKind").cloned().unwrap_or(json!("object")),
                "fields": fact.get("fields").cloned().unwrap_or(json!([])),
                "literals": fact.get("literals").cloned().unwrap_or(Value::Null),
                "confidence": fact.get("confidence").cloned().unwrap_or(json!("medium")),
            })
        })
        .collect::<Vec<_>>();
    let complete = shape_index
        .pointer("/meta/complete")
        .and_then(Value::as_bool)
        == Some(true);
    if matches.is_empty() && !complete {
        return unavailable_shape(
            shape,
            "shape-index.json",
            &format!("[확인 불가, shape-index.json is incomplete; hash {hash} was not observed but absence is not grounded]"),
            Some((hash, if type_literal.is_some() { "typeLiteral" } else { "hash" })),
        );
    }
    json!({
        "kind": "shape",
        "shape": shape,
        "shapeHash": hash,
        "shapeHashSource": if type_literal.is_some() { "typeLiteral" } else { "hash" },
        "result": if matches.is_empty() { "NOT_OBSERVED" } else { "SHAPE_MATCH" },
        "matches": matches,
        "citations": [format!("[grounded, shape-index.json facts[] matched {} identities for {hash}]", matches.len())],
    })
}

fn unavailable_shape(
    shape: &Value,
    artifact: &str,
    citation: &str,
    hash: Option<(&str, &str)>,
) -> Value {
    let mut value = json!({
        "kind": "shape",
        "shape": shape,
        "result": "UNAVAILABLE",
        "artifact": artifact,
        "citations": [citation],
    });
    if let Some((hash, source)) = hash {
        value["shapeHash"] = json!(hash);
        value["shapeHashSource"] = json!(source);
    }
    value
}

fn lookup_inline_patterns(
    refactor_sources: &[Value],
    inline_patterns: &Value,
    evidence_files: &Value,
) -> Value {
    let Some(groups) = inline_patterns.get("groups").and_then(Value::as_array) else {
        return json!({
            "kind": "inline-pattern",
            "result": "UNAVAILABLE",
            "reason": "missing-artifact",
            "artifact": "pre-write-evidence#inlinePatterns",
            "citations": ["[확인 불가, current-run inline pattern evidence is absent]"],
        });
    };
    let mut matched = groups
        .iter()
        .filter(|group| {
            group
                .get("occurrences")
                .and_then(Value::as_array)
                .is_some_and(|occurrences| {
                    refactor_sources.iter().any(|source| {
                        occurrences
                            .iter()
                            .any(|occurrence| occurrence_matches(source, occurrence))
                    })
                })
        })
        .cloned()
        .collect::<Vec<_>>();
    matched.sort_by(|left, right| {
        right
            .get("size")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            .cmp(&left.get("size").and_then(Value::as_u64).unwrap_or(0))
            .then_with(|| string_at(left, "patternHash").cmp(string_at(right, "patternHash")))
    });
    let scanned_files = evidence_files
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect::<BTreeSet<_>>();
    let affected_sources = refactor_sources
        .iter()
        .filter_map(|source| source.get("file").and_then(Value::as_str))
        .filter(|file| !scanned_files.contains(file))
        .collect::<Vec<_>>();
    let relevant_diagnostics = inline_patterns
        .get("diagnostics")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|diagnostic| {
            diagnostic
                .get("file")
                .and_then(Value::as_str)
                .is_some_and(|file| {
                    refactor_sources
                        .iter()
                        .any(|source| source.get("file").and_then(Value::as_str) == Some(file))
                })
        })
        .cloned()
        .collect::<Vec<_>>();
    if matched.is_empty() && (!affected_sources.is_empty() || !relevant_diagnostics.is_empty()) {
        return json!({
            "kind": "inline-pattern",
            "result": "UNAVAILABLE",
            "reason": "refactor-source-evidence-incomplete",
            "artifact": "pre-write-evidence#inlinePatterns",
            "unscannedFiles": affected_sources,
            "diagnostics": relevant_diagnostics,
            "citations": ["[확인 불가, one or more refactorSources were not scanned or were skipped; absence is not grounded]"],
        });
    }
    let mut citations = vec![if matched.is_empty() {
        "[grounded, current-run inlinePatterns groups contain no pattern intersecting refactorSources]".to_string()
    } else {
        format!(
            "[grounded, current-run inlinePatterns groups intersect {} refactor source{}]",
            refactor_sources.len(),
            if refactor_sources.len() == 1 { "" } else { "s" }
        )
    }];
    if !affected_sources.is_empty() || !relevant_diagnostics.is_empty() {
        citations.push("[degraded, some requested refactor source evidence was unavailable; positive inline-pattern matches remain grounded]".to_string());
    }
    json!({
        "kind": "inline-pattern",
        "result": if matched.is_empty() { "NO_INLINE_PATTERN_MATCH" } else { "INLINE_PATTERN_MATCH" },
        "groups": matched,
        "citations": citations,
    })
}

fn occurrence_matches(source: &Value, occurrence: &Value) -> bool {
    if source.get("file").and_then(Value::as_str) != occurrence.get("file").and_then(Value::as_str)
    {
        return false;
    }
    let Some(lines) = source.get("lines").and_then(Value::as_array) else {
        return true;
    };
    let start = occurrence.get("line").and_then(Value::as_u64);
    let end = occurrence.get("endLine").and_then(Value::as_u64).or(start);
    match (start, end) {
        (Some(start), Some(end)) => lines
            .iter()
            .filter_map(Value::as_u64)
            .any(|line| (start..=end).contains(&line)),
        _ => false,
    }
}

pub(super) fn compute_drift(lookups: &[Value]) -> Vec<Value> {
    let mut drift = Vec::new();
    for lookup in lookups {
        if lookup.get("kind").and_then(Value::as_str) != Some("name") {
            continue;
        }
        let Some(claim) = lookup
            .get("canonicalClaim")
            .filter(|value| value.is_object())
        else {
            continue;
        };
        let status = lookup.get("canonicalAstStatus").and_then(Value::as_str);
        if !matches!(status, Some("ast-absent" | "owner-disagrees")) {
            continue;
        }
        drift.push(json!({
            "intentName": lookup.get("intentName").cloned().unwrap_or(Value::Null),
            "canonicalOwner": claim.get("ownerFile").cloned().unwrap_or(Value::Null),
            "canonicalFile": claim.get("file").cloned().unwrap_or(Value::Null),
            "canonicalLine": claim.get("line").cloned().unwrap_or(Value::Null),
            "astOwners": lookup.get("identities").and_then(Value::as_array).into_iter().flatten()
                .filter_map(|identity| identity.get("ownerFile").cloned()).collect::<Vec<_>>(),
            "kind": status.unwrap_or_default(),
        }));
    }
    drift
}

fn package_root(specifier: &str) -> Option<&str> {
    if specifier.is_empty() || specifier.starts_with('.') || specifier.starts_with('/') {
        return None;
    }
    if let Some(scoped) = specifier.strip_prefix('@') {
        let second_slash = scoped.find('/')? + 1;
        let after = &specifier[second_slash + 1..];
        if after.is_empty() {
            return None;
        }
        let end = after
            .find('/')
            .map_or(specifier.len(), |index| second_slash + 1 + index);
        return Some(&specifier[..end]);
    }
    Some(specifier.split('/').next().unwrap_or(specifier))
}

fn unique_tokens(parts: &[Option<&str>]) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut tokens = Vec::new();
    for part in parts.iter().flatten() {
        for token in tokenize(part) {
            if token.len() >= 2
                && !SEMANTIC_STOP_TOKENS.contains(&token.as_str())
                && seen.insert(token.clone())
            {
                tokens.push(token);
            }
        }
    }
    tokens
}

fn tokenize(value: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let chars = value.chars().collect::<Vec<_>>();
    for (index, ch) in chars.iter().copied().enumerate() {
        let boundary = if index == 0 {
            false
        } else {
            let previous = chars[index - 1];
            (ch.is_ascii_uppercase()
                && (previous.is_ascii_lowercase() || previous.is_ascii_digit()))
                || (!ch.is_ascii_alphanumeric() && !current.is_empty())
        };
        if boundary && !current.is_empty() {
            tokens.push(normalize_token(&current));
            current.clear();
        }
        if ch.is_ascii_alphanumeric() {
            current.push(ch.to_ascii_lowercase());
        }
    }
    if !current.is_empty() {
        tokens.push(normalize_token(&current));
    }
    tokens
        .into_iter()
        .filter(|token| !token.is_empty())
        .collect()
}

fn normalize_token(token: &str) -> String {
    match token {
        "cfg" => "config".to_string(),
        "configuration" => "config".to_string(),
        otherwise => normalize_domain_token(otherwise),
    }
}

fn normalize_domain_token(token: &str) -> String {
    if token.len() > 3 && token.ends_with("ies") {
        format!("{}y", &token[..token.len() - 3])
    } else if token.len() > 3
        && token.ends_with('s')
        && !token.ends_with("ss")
        && !token.ends_with("us")
    {
        token[..token.len() - 1].to_string()
    } else {
        token.to_string()
    }
}

fn common_tokens(left: &str, right: &str) -> Vec<String> {
    let left = unique_tokens(&[Some(left)])
        .into_iter()
        .collect::<BTreeSet<_>>();
    unique_tokens(&[Some(right)])
        .into_iter()
        .filter(|token| left.contains(token))
        .collect()
}

fn is_weak_token(token: &str) -> bool {
    WEAK_COMMON_TOKENS.contains(&token)
}

fn shared_prefix(left: &str, right: &str) -> usize {
    left.chars()
        .zip(right.chars())
        .take_while(|(left, right)| left == right)
        .count()
}

fn levenshtein_capped(left: &str, right: &str, cap: usize) -> usize {
    let left = left.as_bytes();
    let right = right.as_bytes();
    if left.len().abs_diff(right.len()) > cap {
        return cap + 1;
    }
    let mut previous = (0..=right.len()).collect::<Vec<_>>();
    let mut current = vec![0; right.len() + 1];
    for (left_index, left_char) in left.iter().enumerate() {
        current[0] = left_index + 1;
        let mut row_min = current[0];
        for (right_index, right_char) in right.iter().enumerate() {
            let cost = usize::from(left_char != right_char);
            current[right_index + 1] = (current[right_index] + 1)
                .min(previous[right_index + 1] + 1)
                .min(previous[right_index] + cost);
            row_min = row_min.min(current[right_index + 1]);
        }
        if row_min > cap {
            return cap + 1;
        }
        std::mem::swap(&mut previous, &mut current);
    }
    previous[right.len()]
}

fn locality(candidate: &SearchCandidate, owner_hint: Option<&str>) -> Value {
    let same_file = owner_hint == Some(candidate.owner_file.as_str());
    let same_dir = owner_hint.is_some_and(|owner| dirname(owner) == dirname(&candidate.owner_file));
    json!({ "sameDir": same_dir, "sameFile": same_file })
}

fn locality_rank(value: &Value) -> usize {
    if value.pointer("/locality/sameFile").and_then(Value::as_bool) == Some(true) {
        2
    } else if value.pointer("/locality/sameDir").and_then(Value::as_bool) == Some(true) {
        1
    } else {
        0
    }
}

fn dirname(path: &str) -> &str {
    path.rsplit_once('/').map_or("", |(directory, _)| directory)
}

fn is_test_like_path(path: &str) -> bool {
    let path = path.to_ascii_lowercase();
    path.contains("/__tests__/")
        || path.contains("/test/")
        || path.contains("/tests/")
        || [".test.", ".spec.", "_test."]
            .iter()
            .any(|marker| path.contains(marker))
}

fn sort_policy_entries(values: &mut [Value]) {
    values.sort_by(|left, right| {
        locality_rank(right)
            .cmp(&locality_rank(left))
            .then_with(|| {
                string_at(left, "operationFamily").cmp(string_at(right, "operationFamily"))
            })
            .then_with(|| string_at(left, "name").cmp(string_at(right, "name")))
            .then_with(|| string_at(left, "ownerFile").cmp(string_at(right, "ownerFile")))
    });
}

fn optional_string(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_string)
}

fn insert_option(object: &mut Map<String, Value>, key: &str, value: Option<&str>) {
    if let Some(value) = value {
        object.insert(key.to_string(), json!(value));
    }
}

fn extend_object(target: &mut Value, extra: Value) {
    let Some(target) = target.as_object_mut() else {
        return;
    };
    let Some(extra) = extra.as_object() else {
        return;
    };
    target.extend(extra.clone());
}

fn string_at<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

fn json_pointer_escape(value: &str) -> String {
    value.replace('~', "~0").replace('/', "~1")
}
