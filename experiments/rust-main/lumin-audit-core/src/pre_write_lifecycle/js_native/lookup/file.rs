use super::*;

pub(super) fn lookup(intent_file: &str, topology: &Value, symbols: &Value, _root: &Path) -> Value {
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

fn is_test_like_path(path: &str) -> bool {
    let path = path.to_ascii_lowercase();
    path.contains("/__tests__/")
        || path.contains("/test/")
        || path.contains("/tests/")
        || [".test.", ".spec.", "_test."]
            .iter()
            .any(|marker| path.contains(marker))
}
