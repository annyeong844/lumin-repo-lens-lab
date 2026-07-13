use super::*;

pub(super) fn lookup(
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
