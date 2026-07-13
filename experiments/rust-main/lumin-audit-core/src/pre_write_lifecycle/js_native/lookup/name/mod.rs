use super::*;

mod evidence;
mod policy;
mod search;

const RESULT_CAP: usize = 5;

pub(super) struct CanonicalClaim {
    name: String,
    owner_file: String,
    line: usize,
    file: String,
    section: String,
}

pub(super) fn load_canonical_claims(root: &Path) -> Result<Vec<CanonicalClaim>> {
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

pub(super) fn lookup(
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
            let (fan_in, fan_in_confidence, fan_in_citation) = evidence::fan_in(symbols, &identity);
            let (fan_space, fan_space_confidence, fan_space_citation) =
                evidence::fan_in_space(symbols, &identity);
            let (contamination, contamination_citation) =
                evidence::contamination(definition, supports);
            let (resolver_confidence, resolver_citation) =
                evidence::resolver_confidence(owner_file, symbols);
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
        search::candidates(symbols)
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
        search::near_names(intent_name, owner_hint, &candidates);
    let intent_tokens = search::unique_tokens(&[
        Some(intent_name),
        declaration
            .and_then(|value| value.get("kind"))
            .and_then(Value::as_str),
        declaration
            .and_then(|value| value.get("why"))
            .and_then(Value::as_str),
    ]);
    let (semantic_hints, suppressed_semantic, suppressed_semantic_count) =
        search::semantic_hints(&intent_tokens, owner_hint, &candidates);
    let service_operation_policy =
        policy::service_operations(intent_name, &suppressed_near, &suppressed_semantic);
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
        "localOperationSiblingPolicy": policy::local_operations(intent_name, owner_hint, symbols),
        "citations": citations,
    })
}
