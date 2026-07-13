use serde_json::{json, Value};
use std::collections::BTreeMap;

const SAFE: &str = "SAFE_CUE";
const REVIEW: &str = "AGENT_REVIEW_CUE";
const MUTED: &str = "MUTED_CUE";

pub(super) struct CueProjection {
    pub(super) cue_policy: Value,
    pub(super) cue_cards: Vec<Value>,
    pub(super) suppressed_cues: Vec<Value>,
    pub(super) unavailable_evidence: Vec<Value>,
}

pub(super) fn project(lookups: &[Value]) -> CueProjection {
    let mut cards = BTreeMap::<String, Value>::new();
    let mut suppressed = Vec::new();
    let mut unavailable = Vec::new();

    for lookup in lookups {
        match lookup.get("kind").and_then(Value::as_str) {
            Some("name") => add_name_lookup(lookup, &mut cards, &mut suppressed),
            Some("file") => add_file_lookup(lookup, &mut cards, &mut suppressed),
            Some("shape") => {
                add_shape_lookup(lookup, &mut cards, &mut suppressed, &mut unavailable)
            }
            Some("inline-pattern") => {
                add_inline_lookup(lookup, &mut cards, &mut suppressed, &mut unavailable)
            }
            _ => {}
        }
    }

    let mut cue_cards = cards.into_values().collect::<Vec<_>>();
    cue_cards.sort_by(|left, right| {
        tier_rank(left)
            .cmp(&tier_rank(right))
            .then_with(|| {
                candidate_string(left, "ownerFile").cmp(candidate_string(right, "ownerFile"))
            })
            .then_with(|| {
                candidate_string(left, "exportedName").cmp(candidate_string(right, "exportedName"))
            })
            .then_with(|| {
                candidate_string(left, "identity").cmp(candidate_string(right, "identity"))
            })
    });
    suppressed.sort_by(|left, right| {
        string_at(left, "reason")
            .cmp(string_at(right, "reason"))
            .then_with(|| string_at(left, "ownerFile").cmp(string_at(right, "ownerFile")))
            .then_with(|| string_at(left, "exportedName").cmp(string_at(right, "exportedName")))
    });
    unavailable.sort_by(|left, right| {
        string_at(left, "evidenceLane")
            .cmp(string_at(right, "evidenceLane"))
            .then_with(|| string_at(left, "reason").cmp(string_at(right, "reason")))
    });

    CueProjection {
        cue_policy: json!({
            "tokenizerVersion": "camel-snake-kebab-digit-v1",
            "tokenPolicyVersion": "prewrite-token-policy-v1",
            "weakCommonTokens": [
                "action", "adapter", "api", "app", "application", "client", "command",
                "config", "context", "data", "domain", "event", "factory", "handler", "item",
                "manager", "model", "module", "option", "provider", "request", "response",
                "result", "service", "state", "store", "type", "util", "value"
            ],
            "aliases": { "cfg": "config", "configuration": "config" },
        }),
        cue_cards,
        suppressed_cues: suppressed,
        unavailable_evidence: unavailable,
    }
}

fn add_name_lookup(
    lookup: &Value,
    cards: &mut BTreeMap<String, Value>,
    suppressed: &mut Vec<Value>,
) {
    for identity in array(lookup, "identities") {
        add_cue(
            cards,
            suppressed,
            identity,
            safe_cue(
                "exact-symbol",
                "exact exported symbol exists",
                json!([{
                    "artifact": "symbols.json",
                    "matchedField": "defIndex",
                    "candidateIdentity": identity.get("identity").cloned().unwrap_or(Value::Null),
                    "algorithmVersion": "exact-symbol.v1",
                }]),
            ),
        );
    }
    for near in array(lookup, "nearNames") {
        let candidate = candidate_from_search(near);
        let class_method =
            near.get("matchedField").and_then(Value::as_str) == Some("classMethodIndex");
        add_cue(
            cards,
            suppressed,
            &candidate,
            review_cue(
                if class_method {
                    "class-method-name"
                } else {
                    "near-name"
                },
                if class_method {
                    "near class method name"
                } else {
                    "near exported name"
                },
                json!([{
                    "artifact": "symbols.json",
                    "matchedField": near.get("matchedField").cloned().unwrap_or(json!("defIndex")),
                    "algorithmVersion": "near-name.v1",
                    "distance": near.get("distance").cloned().unwrap_or(Value::Null),
                    "candidateIdentity": candidate.get("identity").cloned().unwrap_or(Value::Null),
                }]),
            ),
        );
    }
    for hint in array(lookup, "semanticHints") {
        let candidate = candidate_from_search(hint);
        let class_method =
            hint.get("matchedField").and_then(Value::as_str) == Some("classMethodIndex");
        add_cue(
            cards,
            suppressed,
            &candidate,
            review_cue(
                if class_method {
                    "class-method-name"
                } else {
                    "intent-token"
                },
                if class_method {
                    "class method intent-token overlap"
                } else {
                    "supported intent-token overlap"
                },
                json!([{
                    "artifact": "symbols.json",
                    "matchedField": hint.get("matchedField").cloned().unwrap_or(json!("defIndex")),
                    "algorithmVersion": "prewrite-token-policy-v1",
                    "tokens": hint.get("matchedTokens").cloned().unwrap_or(json!([])),
                    "candidateIdentity": candidate.get("identity").cloned().unwrap_or(Value::Null),
                }]),
            ),
        );
    }
    for (field, lane) in [
        ("suppressedNearNames", "near-name"),
        ("suppressedSemanticHints", "intent-token"),
    ] {
        for entry in array(lookup, field) {
            let candidate = candidate_from_search(entry);
            suppressed.push(json!({
                "cueTier": MUTED,
                "evidenceLane": lane,
                "reason": entry.get("reason").cloned().unwrap_or(json!("candidate-suppressed")),
                "tokens": entry.get("matchedTokens").cloned().unwrap_or(json!([])),
                "distance": entry.get("distance").cloned().unwrap_or(Value::Null),
                "lengthDelta": entry.get("lengthDelta").cloned().unwrap_or(Value::Null),
                "score": entry.get("score").cloned().unwrap_or(Value::Null),
                "locality": entry.get("locality").cloned().unwrap_or(Value::Null),
                "candidateCount": entry.get("candidateCount").cloned().unwrap_or(json!(1)),
                "tokenizerVersion": "camel-snake-kebab-digit-v1",
                "tokenPolicyVersion": "prewrite-token-policy-v1",
                "ownerFile": candidate.get("ownerFile").cloned().unwrap_or(Value::Null),
                "exportedName": candidate.get("exportedName").cloned().unwrap_or(Value::Null),
                "identity": candidate.get("identity").cloned().unwrap_or(Value::Null),
                "matchedField": entry.get("matchedField").cloned().unwrap_or(json!("defIndex")),
            }));
        }
    }
    add_service_policy(lookup, cards, suppressed);
    add_local_policy(lookup, cards, suppressed);
}

fn add_service_policy(
    lookup: &Value,
    cards: &mut BTreeMap<String, Value>,
    suppressed: &mut Vec<Value>,
) {
    let Some(policy) = lookup
        .get("serviceOperationSiblingPolicy")
        .filter(|value| value.is_object())
    else {
        return;
    };
    for entry in array(policy, "promoted") {
        let candidate = service_candidate(entry);
        if entry.get("matchedField").and_then(Value::as_str) == Some("classMethodIndex") {
            suppressed.push(service_muted_cue(
                policy,
                entry,
                &candidate,
                "service-sibling-class-method-lane",
            ));
            continue;
        }
        add_cue(
            cards,
            suppressed,
            &candidate,
            review_cue(
                "service-operation-sibling",
                "related service operation sibling",
                json!([{
                    "artifact": "pre-write-advisory.json",
                    "matchedField": "lookups[].serviceOperationSiblingPolicy.promoted",
                    "policyId": policy.get("policyId").cloned().unwrap_or(Value::Null),
                    "policyVersion": policy.get("policyVersion").cloned().unwrap_or(Value::Null),
                    "candidateIdentity": entry.get("identity").cloned().unwrap_or(Value::Null),
                    "operationFamily": entry.get("operationFamily").cloned().unwrap_or(Value::Null),
                    "sharedDomainTokens": entry.get("sharedDomainTokens").cloned().unwrap_or(json!([])),
                    "locality": entry.get("locality").cloned().unwrap_or(Value::Null),
                    "supportingReasons": entry.get("supportingReasons").cloned().unwrap_or(json!([])),
                }]),
            ),
        );
    }
    for entry in array(policy, "muted") {
        let candidate = service_candidate(entry);
        let reason = entry
            .get("reason")
            .and_then(Value::as_str)
            .unwrap_or("service-sibling-muted");
        suppressed.push(service_muted_cue(policy, entry, &candidate, reason));
    }
}

fn service_muted_cue(policy: &Value, entry: &Value, candidate: &Value, reason: &str) -> Value {
    json!({
        "cueTier": MUTED,
        "evidenceLane": "service-operation-sibling",
        "reason": reason,
        "policyId": policy.get("policyId").cloned().unwrap_or(Value::Null),
        "policyVersion": policy.get("policyVersion").cloned().unwrap_or(Value::Null),
        "ownerFile": candidate.get("ownerFile").cloned().unwrap_or(Value::Null),
        "exportedName": candidate.get("exportedName").cloned().unwrap_or(Value::Null),
        "identity": candidate.get("identity").cloned().unwrap_or(Value::Null),
        "matchedField": entry.get("matchedField").cloned().unwrap_or(json!("defIndex")),
        "operationFamily": entry.get("operationFamily").cloned().unwrap_or(Value::Null),
        "sharedDomainTokens": entry.get("sharedDomainTokens").cloned().unwrap_or(json!([])),
        "supportingReasons": entry.get("supportingReasons").cloned().unwrap_or(json!([])),
        "locality": entry.get("locality").cloned().unwrap_or(Value::Null),
    })
}

fn add_local_policy(
    lookup: &Value,
    cards: &mut BTreeMap<String, Value>,
    suppressed: &mut Vec<Value>,
) {
    let Some(policy) = lookup
        .get("localOperationSiblingPolicy")
        .filter(|value| value.is_object())
    else {
        return;
    };
    for entry in array(policy, "promoted") {
        let candidate = local_candidate(entry);
        add_cue(
            cards,
            suppressed,
            &candidate,
            review_cue(
                "local-operation-sibling",
                "related local service operation",
                json!([{
                    "artifact": "pre-write-advisory.json",
                    "matchedField": "lookups[].localOperationSiblingPolicy.promoted",
                    "policyId": policy.get("policyId").cloned().unwrap_or(Value::Null),
                    "policyVersion": policy.get("policyVersion").cloned().unwrap_or(Value::Null),
                    "candidateIdentity": entry.get("identity").cloned().unwrap_or(Value::Null),
                    "matchedFieldSource": "preWriteLocalOperationIndex",
                    "surfaceKind": "nested-local-operation",
                    "containerName": entry.get("containerName").cloned().unwrap_or(Value::Null),
                    "containerKind": entry.get("containerKind").cloned().unwrap_or(Value::Null),
                    "operationFamily": entry.get("operationFamily").cloned().unwrap_or(Value::Null),
                    "sharedDomainTokens": entry.get("sharedDomainTokens").cloned().unwrap_or(json!([])),
                    "locality": entry.get("locality").cloned().unwrap_or(Value::Null),
                    "supportingReasons": entry.get("supportingReasons").cloned().unwrap_or(json!([])),
                }]),
            ),
        );
    }
    for entry in array(policy, "muted") {
        let candidate = local_candidate(entry);
        suppressed.push(json!({
            "cueTier": MUTED,
            "evidenceLane": "local-operation-sibling",
            "reason": entry.get("reason").cloned().unwrap_or(json!("local-operation-muted")),
            "policyId": policy.get("policyId").cloned().unwrap_or(Value::Null),
            "policyVersion": policy.get("policyVersion").cloned().unwrap_or(Value::Null),
            "ownerFile": candidate.get("ownerFile").cloned().unwrap_or(Value::Null),
            "exportedName": candidate.get("exportedName").cloned().unwrap_or(Value::Null),
            "identity": candidate.get("identity").cloned().unwrap_or(Value::Null),
            "matchedField": "preWriteLocalOperationIndex",
            "surfaceKind": "nested-local-operation",
            "containerName": entry.get("containerName").cloned().unwrap_or(Value::Null),
            "containerKind": entry.get("containerKind").cloned().unwrap_or(Value::Null),
            "operationFamily": entry.get("operationFamily").cloned().unwrap_or(Value::Null),
            "sharedDomainTokens": entry.get("sharedDomainTokens").cloned().unwrap_or(json!([])),
            "supportingReasons": entry.get("supportingReasons").cloned().unwrap_or(json!([])),
            "locality": entry.get("locality").cloned().unwrap_or(Value::Null),
        }));
    }
}

fn add_file_lookup(
    lookup: &Value,
    cards: &mut BTreeMap<String, Value>,
    suppressed: &mut Vec<Value>,
) {
    if lookup.get("result").and_then(Value::as_str) != Some("FILE_EXISTS") {
        return;
    }
    let file = lookup
        .get("intentFile")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let candidate = json!({
        "identity": format!("{file}::__file__"),
        "ownerFile": file,
        "exportedName": "__file__",
    });
    add_cue(
        cards,
        suppressed,
        &candidate,
        safe_cue(
            "exact-file",
            "exact file exists",
            json!([{
                "artifact": "topology.json",
                "matchedField": "nodes",
                "file": file,
                "algorithmVersion": "exact-file.v1",
            }]),
        ),
    );
}

fn add_shape_lookup(
    lookup: &Value,
    cards: &mut BTreeMap<String, Value>,
    suppressed: &mut Vec<Value>,
    unavailable: &mut Vec<Value>,
) {
    let result = lookup.get("result").and_then(Value::as_str);
    if result == Some("UNAVAILABLE") {
        let signature =
            lookup.get("shapeHashSource").and_then(Value::as_str) == Some("functionSignature");
        unavailable.push(json!({
            "evidenceLane": if signature { "function-signature" } else { "shape-hash" },
            "status": "UNAVAILABLE",
            "reason": lookup.get("reason").cloned().unwrap_or(json!("lookup-unavailable")),
            "artifact": lookup.get("artifact").cloned().unwrap_or(json!(if signature { "function-clones.json" } else { "shape-index.json" })),
            "citations": lookup.get("citations").cloned().unwrap_or(json!([])),
        }));
        return;
    }
    if !matches!(result, Some("SHAPE_MATCH" | "SIGNATURE_MATCH")) {
        return;
    }
    let signature = result == Some("SIGNATURE_MATCH");
    for matched in array(lookup, "matches") {
        let evidence = json!([{
            "artifact": if signature { "function-clones.json" } else { "shape-index.json" },
            "matchedField": if signature { "normalizedSignatureHash" } else { "hash" },
            "algorithmVersion": if signature { "function-signature.normalized.v1" } else { "shape-hash.normalized.v1" },
            "hash": lookup.get("shapeHash").cloned().unwrap_or(Value::Null),
            "visibility": matched.get("visibility").cloned().unwrap_or(Value::Null),
            "localName": matched.get("localName").cloned().unwrap_or(Value::Null),
        }]);
        let review = signature
            && matched
                .get("visibility")
                .and_then(Value::as_str)
                .is_some_and(|visibility| visibility != "exported");
        add_cue(
            cards,
            suppressed,
            matched,
            if review {
                review_cue(
                    "function-signature",
                    "same normalized function signature",
                    evidence,
                )
            } else {
                safe_cue(
                    if signature {
                        "function-signature"
                    } else {
                        "shape-hash"
                    },
                    if signature {
                        "same normalized function signature"
                    } else {
                        "same normalized type shape"
                    },
                    evidence,
                )
            },
        );
    }
}

fn add_inline_lookup(
    lookup: &Value,
    cards: &mut BTreeMap<String, Value>,
    suppressed: &mut Vec<Value>,
    unavailable: &mut Vec<Value>,
) {
    if lookup.get("result").and_then(Value::as_str) == Some("UNAVAILABLE") {
        unavailable.push(json!({
            "evidenceLane": "inline-extraction",
            "status": "UNAVAILABLE",
            "reason": lookup.get("reason").cloned().unwrap_or(json!("lookup-unavailable")),
            "artifact": lookup.get("artifact").cloned().unwrap_or(json!("inline-patterns.json")),
            "citations": lookup.get("citations").cloned().unwrap_or(json!([])),
        }));
        return;
    }
    if lookup.get("result").and_then(Value::as_str) != Some("INLINE_PATTERN_MATCH") {
        return;
    }
    for group in array(lookup, "groups") {
        let owner_file = group
            .get("ownerFiles")
            .and_then(Value::as_array)
            .and_then(|files| files.first())
            .and_then(Value::as_str)
            .or_else(|| {
                group
                    .get("occurrences")
                    .and_then(Value::as_array)
                    .and_then(|occurrences| occurrences.first())
                    .and_then(|occurrence| occurrence.get("file"))
                    .and_then(Value::as_str)
            })
            .unwrap_or("unknown");
        let hash = group
            .get("patternHash")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let candidate = json!({
            "identity": format!("inline-pattern:{hash}"),
            "ownerFile": owner_file,
            "exportedName": group.get("kind").cloned().unwrap_or(json!("inline-pattern")),
        });
        add_cue(
            cards,
            suppressed,
            &candidate,
            review_cue(
                "inline-extraction",
                "repeated inline statement pattern",
                json!([{
                    "artifact": "inline-patterns.json",
                    "matchedField": "groups[].patternHash",
                    "algorithmVersion": group.get("normalizerVersion").cloned().unwrap_or(json!("inline-statement-normalizer-v1")),
                    "patternHash": hash,
                    "occurrenceCount": group.get("size").cloned().unwrap_or(Value::Null),
                    "ownerFiles": group.get("ownerFiles").cloned().unwrap_or(json!([])),
                    "reviewReason": group.get("reviewReason").cloned().unwrap_or(Value::Null),
                }]),
            ),
        );
    }
}

fn add_cue(
    cards: &mut BTreeMap<String, Value>,
    suppressed: &mut Vec<Value>,
    candidate: &Value,
    cue: Value,
) {
    let owner_file = candidate
        .get("ownerFile")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let exported_name = candidate
        .get("exportedName")
        .or_else(|| candidate.get("name"))
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let identity = candidate
        .get("identity")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| format!("{owner_file}::{exported_name}"));
    if policy_excluded(owner_file)
        || candidate.get("policyExcluded").and_then(Value::as_bool) == Some(true)
    {
        let mut muted = cue;
        if let Some(object) = muted.as_object_mut() {
            let original_cue_tier = object.get("cueTier").cloned().unwrap_or(Value::Null);
            object.insert("cueTier".to_string(), json!(MUTED));
            object.insert("originalCueTier".to_string(), original_cue_tier);
            object.insert("reason".to_string(), json!("policy-excluded"));
            object.insert(
                "policyReason".to_string(),
                json!(format!("path:{owner_file}")),
            );
            object.insert("ownerFile".to_string(), json!(owner_file));
            object.insert("exportedName".to_string(), json!(exported_name));
            object.insert("identity".to_string(), json!(identity));
        }
        suppressed.push(muted);
        return;
    }
    let card = cards.entry(identity.clone()).or_insert_with(|| {
        json!({
            "candidate": {
                "identity": identity,
                "ownerFile": owner_file,
                "exportedName": exported_name,
            },
            "renderTier": SAFE,
            "cues": [],
        })
    });
    if cue.get("cueTier").and_then(Value::as_str) == Some(REVIEW) {
        card["renderTier"] = json!(REVIEW);
    }
    if let Some(cues) = card.get_mut("cues").and_then(Value::as_array_mut) {
        cues.push(cue);
    }
}

fn safe_cue(lane: &str, claim: &str, evidence: Value) -> Value {
    json!({
        "cueTier": SAFE,
        "safeMeaning": "claim-only",
        "notSafeFor": ["semantic-equivalence", "auto-reuse", "auto-fix"],
        "evidenceLane": lane,
        "claim": claim,
        "confidence": "grounded",
        "evidence": evidence,
    })
}

fn review_cue(lane: &str, claim: &str, evidence: Value) -> Value {
    json!({
        "cueTier": REVIEW,
        "evidenceLane": lane,
        "claim": claim,
        "confidence": "heuristic-review",
        "evidence": evidence,
    })
}

fn candidate_from_search(entry: &Value) -> Value {
    let owner_file = entry.get("ownerFile").cloned().unwrap_or(json!("unknown"));
    let exported_name = entry
        .get("exportedName")
        .or_else(|| entry.get("name"))
        .cloned()
        .unwrap_or(json!("unknown"));
    let identity = entry.get("identity").cloned().unwrap_or_else(|| {
        json!(format!(
            "{}::{}",
            owner_file.as_str().unwrap_or("unknown"),
            exported_name.as_str().unwrap_or("unknown")
        ))
    });
    json!({
        "identity": identity,
        "ownerFile": owner_file,
        "exportedName": exported_name,
    })
}

fn local_candidate(entry: &Value) -> Value {
    json!({
        "identity": entry.get("identity").cloned().unwrap_or(Value::Null),
        "ownerFile": entry.get("ownerFile").cloned().unwrap_or(json!("unknown")),
        "exportedName": entry.get("name").cloned().unwrap_or(json!("unknown")),
    })
}

fn service_candidate(entry: &Value) -> Value {
    let owner = entry
        .get("ownerFile")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let name = entry
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    json!({
        "identity": entry.get("identity").cloned().unwrap_or_else(|| json!(format!("{owner}::{name}"))),
        "ownerFile": owner,
        "exportedName": name,
    })
}

fn policy_excluded(owner_file: &str) -> bool {
    owner_file.replace('\\', "/").split('/').any(|segment| {
        matches!(
            segment,
            "dist" | "build" | "coverage" | "vendor" | "generated" | "node_modules"
        )
    })
}

fn tier_rank(value: &Value) -> usize {
    match value.get("renderTier").and_then(Value::as_str) {
        Some(SAFE) => 0,
        Some(REVIEW) => 1,
        Some(MUTED) => 2,
        _ => 99,
    }
}

fn candidate_string<'a>(value: &'a Value, key: &str) -> &'a str {
    value
        .get("candidate")
        .and_then(|candidate| candidate.get(key))
        .and_then(Value::as_str)
        .unwrap_or("")
}

fn string_at<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

fn array<'a>(value: &'a Value, key: &str) -> impl Iterator<Item = &'a Value> {
    value
        .get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
}
