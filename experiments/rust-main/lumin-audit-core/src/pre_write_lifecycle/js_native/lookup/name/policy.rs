use super::*;

pub(super) fn local_operations(
    intent_name: &str,
    owner_hint: Option<&str>,
    symbols: &Value,
) -> Value {
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

pub(super) fn service_operations(
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
            if search::locality_rank(entry) > search::locality_rank(candidate) {
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
    let tokens = search::unique_tokens(&[Some(name)]);
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

fn sort_policy_entries(values: &mut [Value]) {
    values.sort_by(|left, right| {
        search::locality_rank(right)
            .cmp(&search::locality_rank(left))
            .then_with(|| {
                string_at(left, "operationFamily").cmp(string_at(right, "operationFamily"))
            })
            .then_with(|| string_at(left, "name").cmp(string_at(right, "name")))
            .then_with(|| string_at(left, "ownerFile").cmp(string_at(right, "ownerFile")))
    });
}
