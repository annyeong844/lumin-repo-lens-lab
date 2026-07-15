use super::*;
use std::path::PathBuf;

const DEPENDENCY_HUB_THRESHOLD: usize = 10;

pub(super) enum PackageManifestSelection {
    Selected { file: String, value: Value },
    Ambiguous { files: Vec<String> },
    Unavailable { reason: String },
}

struct ImportEvidence {
    examples: Vec<Value>,
    total: usize,
    available: bool,
    unavailable_reason: String,
}

pub(super) fn select_manifest(
    root: &Path,
    intent: &Value,
    dep_name: &str,
    failures: &mut Vec<Value>,
) -> Result<PackageManifestSelection> {
    let owner_hints = intent
        .get("dependencyDeclarations")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|entry| entry.get("specifier").and_then(Value::as_str) == Some(dep_name))
        .filter_map(|entry| entry.get("ownerFile").and_then(Value::as_str))
        .collect::<BTreeSet<_>>();

    let mut manifest_files = BTreeSet::new();
    if !owner_hints.is_empty() {
        for owner_file in owner_hints {
            let Some(manifest_file) = nearest_package_manifest(root, owner_file) else {
                return Ok(PackageManifestSelection::Unavailable {
                    reason: format!("no package.json owns dependency intent file '{owner_file}'"),
                });
            };
            manifest_files.insert(manifest_file);
        }
    } else {
        for owner_file in string_array(intent, "files") {
            if let Some(manifest_file) = nearest_package_manifest(root, owner_file) {
                manifest_files.insert(manifest_file);
            }
        }
        if manifest_files.is_empty() && root.join("package.json").is_file() {
            manifest_files.insert("package.json".to_string());
        }
    }

    if manifest_files.len() > 1 {
        return Ok(PackageManifestSelection::Ambiguous {
            files: manifest_files.into_iter().collect(),
        });
    }
    let Some(file) = manifest_files.into_iter().next() else {
        return Ok(PackageManifestSelection::Unavailable {
            reason: "no package.json owns the dependency intent scope".to_string(),
        });
    };
    read_manifest(root, file, failures)
}

fn read_manifest(
    root: &Path,
    file: String,
    failures: &mut Vec<Value>,
) -> Result<PackageManifestSelection> {
    let path = root.join(&file);
    let text = fs::read_to_string(&path)
        .with_context(|| format!("pre-write: failed to read {}", path.display()))?;
    match serde_json::from_str::<Value>(&text) {
        Ok(value) if value.is_object() => Ok(PackageManifestSelection::Selected { file, value }),
        Ok(_) => {
            failures.push(json!({
                "kind": "package-json-invalid-shape",
                "file": file,
                "reason": "package.json must contain a JSON object",
            }));
            Ok(PackageManifestSelection::Unavailable {
                reason: format!("owner manifest '{file}' is not a JSON object"),
            })
        }
        Err(error) => {
            failures.push(json!({
                "kind": "package-json-parse-error",
                "file": file,
                "reason": error.to_string(),
            }));
            Ok(PackageManifestSelection::Unavailable {
                reason: format!("owner manifest '{file}' is invalid JSON"),
            })
        }
    }
}

pub(super) fn lookup(
    dep_name: &str,
    manifest: &PackageManifestSelection,
    symbols: &Value,
) -> Value {
    let dep_root = package_root(dep_name).unwrap_or(dep_name);
    let imports = collect_import_evidence(dep_root, symbols);
    let (manifest_file, package_json) = match manifest {
        PackageManifestSelection::Selected { file, value } => (file, value),
        PackageManifestSelection::Ambiguous { files } => {
            return lookup_without_manifest(
                dep_name,
                "DEPENDENCY_OWNER_AMBIGUOUS",
                format!(
                    "dependency owner is ambiguous across manifests: {}",
                    files.join(", ")
                ),
                files,
                &imports,
            );
        }
        PackageManifestSelection::Unavailable { reason } => {
            return lookup_without_manifest(
                dep_name,
                "DEPENDENCY_MANIFEST_UNAVAILABLE",
                reason.clone(),
                &[],
                &imports,
            );
        }
    };
    let declaration = ["dependencies", "devDependencies", "peerDependencies"]
        .into_iter()
        .find_map(|bucket| {
            package_json
                .get(bucket)
                .and_then(Value::as_object)
                .and_then(|values| values.get(dep_root))
                .map(|version| (bucket, version))
        });
    let mut citations = Vec::new();
    let result = match (declaration, imports.available) {
        (Some((bucket, version)), false) => {
            citations.push(format!(
                "[grounded, {manifest_file}.{bucket}['{dep_root}'] = '{version}']"
            ));
            citations.push(format!(
                "[확인 불가, reason: {}; observed static-import consumer count unavailable for '{dep_root}']",
                imports.unavailable_reason
            ));
            "DEPENDENCY_AVAILABLE_IMPORT_GRAPH_UNAVAILABLE"
        }
        (Some((bucket, version)), true) if imports.total > 0 => {
            citations.push(format!(
                "[grounded, {manifest_file}.{bucket}['{dep_root}'] = '{version}']"
            ));
            citations.push(format!(
                "[grounded, symbols.json.dependencyImportConsumers fromSpec matches '{dep_root}' → {} observed static-import consumer{}]",
                imports.total,
                if imports.total == 1 { "" } else { "s" }
            ));
            "DEPENDENCY_AVAILABLE"
        }
        (Some((bucket, version)), true) => {
            citations.push(format!(
                "[grounded, {manifest_file}.{bucket}['{dep_root}'] = '{version}']"
            ));
            citations.push(format!("[확인 불가, scan range: import graph only — '{dep_root}' may still be consumed by scripts, config, runtime plugins, or build steps outside static imports]"));
            "DEPENDENCY_AVAILABLE_NO_OBSERVED_IMPORTS"
        }
        (None, _) => {
            citations.push(format!("[grounded, {manifest_file}.{{dependencies, devDependencies, peerDependencies}} does not contain '{dep_root}']"));
            "NEW_PACKAGE"
        }
    };
    json!({
        "kind": "dependency",
        "depName": dep_name,
        "manifestFile": manifest_file,
        "declaredIn": declaration.map(|(bucket, _)| bucket),
        "result": result,
        "existingImports": import_evidence_value(&imports),
        "citations": citations,
    })
}

fn lookup_without_manifest(
    dep_name: &str,
    result: &str,
    reason: String,
    candidates: &[String],
    imports: &ImportEvidence,
) -> Value {
    json!({
        "kind": "dependency",
        "depName": dep_name,
        "manifestFile": Value::Null,
        "ownerManifestCandidates": candidates,
        "declaredIn": Value::Null,
        "result": result,
        "existingImports": import_evidence_value(imports),
        "citations": [format!("[확인 불가, reason: {reason}]")],
    })
}

fn collect_import_evidence(dep_root: &str, symbols: &Value) -> ImportEvidence {
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
    ImportEvidence {
        examples,
        total,
        available: records.is_some(),
        unavailable_reason: unavailable_reason.to_string(),
    }
}

fn import_evidence_value(imports: &ImportEvidence) -> Value {
    json!({
        "examples": imports.examples,
        "observedImportCount": if imports.available { json!(imports.total) } else { Value::Null },
        "countConfidence": if imports.available { "grounded" } else { "unavailable" },
        "unavailableReason": if imports.available { Value::Null } else { json!(imports.unavailable_reason) },
        "watchForEligible": imports.available && imports.total >= DEPENDENCY_HUB_THRESHOLD,
    })
}

fn nearest_package_manifest(root: &Path, owner_file: &str) -> Option<String> {
    let mut directory = Path::new(owner_file).parent()?.to_path_buf();
    loop {
        let candidate = if directory.as_os_str().is_empty() {
            PathBuf::from("package.json")
        } else {
            directory.join("package.json")
        };
        if root.join(&candidate).is_file() {
            return Some(candidate.to_string_lossy().replace('\\', "/"));
        }
        if !directory.pop() {
            break;
        }
    }
    None
}

fn string_array<'a>(value: &'a Value, key: &str) -> impl Iterator<Item = &'a str> {
    value
        .get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn uses_nearest_workspace_manifest_for_dependency_lookup() -> Result<()> {
        let temp = tempdir()?;
        fs::create_dir_all(temp.path().join("apps/daemon/src"))?;
        fs::write(temp.path().join("package.json"), r#"{"private":true}"#)?;
        fs::write(
            temp.path().join("apps/daemon/package.json"),
            r#"{"dependencies":{"@vscode/ripgrep":"^1.17.1"}}"#,
        )?;
        let intent = json!({
            "files": ["apps/daemon/src/home.ts"],
            "dependencyDeclarations": [{
                "specifier": "@vscode/ripgrep",
                "ownerFile": "apps/daemon/src/home.ts"
            }]
        });
        let mut failures = Vec::new();

        let manifest = select_manifest(temp.path(), &intent, "@vscode/ripgrep", &mut failures)?;
        let result = lookup(
            "@vscode/ripgrep",
            &manifest,
            &json!({"dependencyImportConsumers": []}),
        );

        assert!(failures.is_empty());
        assert_eq!(result["manifestFile"], "apps/daemon/package.json");
        assert_eq!(result["declaredIn"], "dependencies");
        assert_eq!(result["result"], "DEPENDENCY_AVAILABLE_NO_OBSERVED_IMPORTS");
        Ok(())
    }

    #[test]
    fn refuses_new_package_claim_for_ambiguous_workspace_owners() -> Result<()> {
        let temp = tempdir()?;
        for workspace in ["apps/a", "apps/b"] {
            fs::create_dir_all(temp.path().join(workspace).join("src"))?;
            fs::write(temp.path().join(workspace).join("package.json"), "{}")?;
        }
        let intent = json!({
            "files": ["apps/a/src/a.ts", "apps/b/src/b.ts"],
            "dependencyDeclarations": [
                {"specifier": "zod", "ownerFile": "apps/a/src/a.ts"},
                {"specifier": "zod", "ownerFile": "apps/b/src/b.ts"}
            ]
        });
        let mut failures = Vec::new();

        let manifest = select_manifest(temp.path(), &intent, "zod", &mut failures)?;
        let result = lookup("zod", &manifest, &json!({"dependencyImportConsumers": []}));

        assert!(failures.is_empty());
        assert_eq!(result["result"], "DEPENDENCY_OWNER_AMBIGUOUS");
        assert_eq!(
            result["ownerManifestCandidates"].as_array().map(Vec::len),
            Some(2)
        );
        Ok(())
    }
}
