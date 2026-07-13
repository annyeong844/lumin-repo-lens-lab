use super::*;

const DEPENDENCY_HUB_THRESHOLD: usize = 10;

pub(super) fn read_package_json(root: &Path, failures: &mut Vec<Value>) -> Result<Value> {
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

pub(super) fn lookup(dep_name: &str, package_json: &Value, symbols: &Value) -> Value {
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
