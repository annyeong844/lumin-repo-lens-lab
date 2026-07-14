use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

use super::protocol::{
    JsTsPreWriteEvidenceRequest, JsTsPreWriteHostTransport,
    JS_TS_PRE_WRITE_EVIDENCE_RESPONSE_SCHEMA_VERSION,
    JS_TS_PRE_WRITE_HOST_TRANSPORT_SCHEMA_VERSION,
};
use crate::runtime_contract::{
    JS_RUNTIME_BRIDGE_CONTRACT_VERSION, NATIVE_LIFECYCLE_HOST_EVIDENCE_FEATURE,
    RUNTIME_CONTRACT_SCHEMA_VERSION,
};

pub(super) fn collect(
    mut request: JsTsPreWriteEvidenceRequest,
    transport: &JsTsPreWriteHostTransport,
    local_result_dir: &Path,
    result_identity: &str,
) -> Result<Value> {
    validate_transport(transport, &request, result_identity)?;
    validate_helper_contract(transport)?;

    let local_root = request.root.clone();
    let local_cache_root = request.incremental.cache_root.clone();
    request.root = transport.root.clone();
    request.incremental.cache_root = if request.incremental.enabled {
        Some(transport.cache_root.clone().context(
            "host evidence transport requires cacheRoot when incremental reuse is enabled",
        )?)
    } else {
        None
    };

    fs::create_dir_all(local_result_dir).with_context(|| {
        format!(
            "host evidence transport failed to create {}",
            local_result_dir.display()
        )
    })?;
    let result_name = format!(
        ".lumin-host-evidence-{result_identity}-{}.json",
        std::process::id()
    );
    let local_result_path = local_result_dir.join(&result_name);
    remove_file_if_present(&local_result_path)?;
    let host_result_path = join_host_path(&transport.output, &result_name)?;
    let input = serde_json::to_vec(&request)
        .context("host evidence transport failed to serialize the evidence request")?;

    let run_result = (|| {
        let output = run_command(
            &transport.command,
            &[
                "js-ts-pre-write-evidence",
                "--input",
                "-",
                "--result-output",
                &host_result_path,
            ],
            Some(&input),
        )?;
        if !output.status.success() {
            bail!(
                "Windows host evidence helper exited with {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
        if !output.stdout.iter().all(u8::is_ascii_whitespace) {
            bail!("Windows host evidence helper wrote unexpected stdout with --result-output");
        }
        let bytes = fs::read(&local_result_path).with_context(|| {
            format!(
                "Windows host evidence helper did not write {}",
                local_result_path.display()
            )
        })?;
        let mut evidence = serde_json::from_slice::<Value>(&bytes)
            .context("Windows host evidence helper wrote malformed result JSON")?;
        if evidence.is_null() {
            bail!("Windows host evidence helper wrote JSON null");
        }
        normalize_host_evidence_paths(
            &mut evidence,
            transport,
            &local_root,
            local_cache_root.as_deref(),
        )?;
        Ok(evidence)
    })();

    let cleanup_result = remove_file_if_present(&local_result_path);
    match (run_result, cleanup_result) {
        (Ok(evidence), Ok(())) => Ok(evidence),
        (Ok(_), Err(error)) => Err(error),
        (Err(error), Ok(())) => Err(error),
        (Err(error), Err(cleanup_error)) => Err(error.context(format!(
            "host evidence result cleanup also failed: {cleanup_error:#}"
        ))),
    }
}

fn validate_transport(
    transport: &JsTsPreWriteHostTransport,
    request: &JsTsPreWriteEvidenceRequest,
    result_identity: &str,
) -> Result<()> {
    if transport.schema_version != JS_TS_PRE_WRITE_HOST_TRANSPORT_SCHEMA_VERSION {
        bail!(
            "unsupported host evidence transport schemaVersion '{}'",
            transport.schema_version
        );
    }
    if !transport.command.is_file() {
        bail!(
            "host evidence transport command is not a file: {}",
            transport.command.display()
        );
    }
    for (label, value) in [("root", &transport.root), ("output", &transport.output)] {
        if !is_windows_absolute_path(value) {
            bail!("host evidence transport {label} must be an absolute Windows path");
        }
    }
    if request.incremental.enabled {
        let cache_root = transport.cache_root.as_ref().context(
            "host evidence transport cacheRoot is required when incremental reuse is enabled",
        )?;
        if !is_windows_absolute_path(cache_root) {
            bail!("host evidence transport cacheRoot must be an absolute Windows path");
        }
    }
    if result_identity.is_empty()
        || result_identity.contains(['/', '\\'])
        || result_identity.contains("..")
    {
        bail!("host evidence transport result identity must be filename-safe");
    }
    Ok(())
}

fn validate_helper_contract(transport: &JsTsPreWriteHostTransport) -> Result<()> {
    let output = run_command(&transport.command, &["runtime-contract"], None)?;
    if !output.status.success() {
        bail!(
            "Windows host helper runtime-contract exited with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    if !output.stderr.iter().all(u8::is_ascii_whitespace) {
        bail!("Windows host helper runtime-contract wrote unexpected stderr");
    }
    let contract = serde_json::from_slice::<Value>(&output.stdout)
        .context("Windows host helper runtime-contract returned malformed JSON")?;
    if contract.get("schemaVersion").and_then(Value::as_str)
        != Some(RUNTIME_CONTRACT_SCHEMA_VERSION)
        || contract.get("contractVersion").and_then(Value::as_str)
            != Some(JS_RUNTIME_BRIDGE_CONTRACT_VERSION)
        || contract
            .pointer(&format!(
                "/features/{NATIVE_LIFECYCLE_HOST_EVIDENCE_FEATURE}"
            ))
            .and_then(Value::as_bool)
            != Some(true)
        || !array_contains(
            &contract,
            "/supportedSubcommands",
            "js-ts-pre-write-evidence",
        )
        || !array_contains(
            &contract,
            "/resultOutputSubcommands",
            "js-ts-pre-write-evidence",
        )
    {
        bail!("Windows host helper does not satisfy the current evidence transport contract");
    }
    Ok(())
}

fn run_command(command: &Path, args: &[&str], input: Option<&[u8]>) -> Result<Output> {
    let mut child = Command::new(command)
        .args(args)
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to start Windows host helper {}", command.display()))?;
    if let Some(input) = input {
        child
            .stdin
            .take()
            .context("Windows host helper stdin was unavailable")?
            .write_all(input)
            .context("failed to write Windows host helper request")?;
    }
    child
        .wait_with_output()
        .context("failed to wait for Windows host helper")
}

fn normalize_host_evidence_paths(
    evidence: &mut Value,
    transport: &JsTsPreWriteHostTransport,
    local_root: &Path,
    local_cache_root: Option<&Path>,
) -> Result<()> {
    if evidence.get("schemaVersion").and_then(Value::as_str)
        != Some(JS_TS_PRE_WRITE_EVIDENCE_RESPONSE_SCHEMA_VERSION)
        || !evidence.get("files").is_some_and(Value::is_array)
        || !evidence.get("symbols").is_some_and(Value::is_object)
        || !evidence.get("anyInventory").is_some_and(Value::is_object)
        || !evidence.get("shapeIndex").is_some_and(Value::is_object)
    {
        bail!("Windows host evidence response has an incompatible shape");
    }

    let local_root = slash_path(local_root);
    replace_exact_host_path(evidence, "/root", &transport.root, &local_root)?;
    replace_exact_host_path(
        evidence,
        "/anyInventory/meta/root",
        &transport.root,
        &local_root,
    )?;
    replace_exact_host_path(
        evidence,
        "/shapeIndex/meta/root",
        &transport.root,
        &local_root,
    )?;

    for pointer in [
        "/anyInventory/meta/incremental",
        "/shapeIndex/meta/incremental",
    ] {
        normalize_incremental_paths(
            evidence,
            pointer,
            transport.cache_root.as_deref(),
            local_cache_root,
        )?;
    }
    let runtime = evidence
        .pointer_mut("/summary/runtime")
        .and_then(Value::as_object_mut)
        .context("Windows host evidence response missing summary.runtime")?;
    runtime.insert(
        "evidenceTransport".to_string(),
        json!({
            "kind": "windows-host-audit-core",
            "contractVersion": JS_RUNTIME_BRIDGE_CONTRACT_VERSION,
            "pathMode": "caller-local",
        }),
    );
    Ok(())
}

fn normalize_incremental_paths(
    evidence: &mut Value,
    pointer: &str,
    host_cache_root: Option<&Path>,
    local_cache_root: Option<&Path>,
) -> Result<()> {
    let Some(incremental) = evidence.pointer_mut(pointer).and_then(Value::as_object_mut) else {
        bail!("Windows host evidence response missing {pointer}");
    };
    let cache_root = incremental
        .get("cacheRoot")
        .and_then(Value::as_str)
        .map(str::to_string);
    let cache_file = incremental
        .get("cacheFile")
        .and_then(Value::as_str)
        .map(str::to_string);
    match (host_cache_root, local_cache_root) {
        (Some(host_root), Some(local_root)) => {
            let returned_root = cache_root
                .as_deref()
                .context("host evidence incremental cacheRoot is missing")?;
            if !same_host_path(returned_root, &slash_path(host_root)) {
                bail!("host evidence incremental cacheRoot does not match the request");
            }
            incremental.insert(
                "cacheRoot".to_string(),
                Value::String(slash_path(local_root)),
            );
            if let Some(cache_file) = cache_file.as_deref() {
                let translated = translate_cache_file(cache_file, host_root, local_root)?;
                incremental.insert("cacheFile".to_string(), Value::String(translated));
            }
        }
        (None, None) => {
            if cache_root.is_some() || cache_file.is_some() {
                bail!("host evidence returned cache paths while incremental reuse was disabled");
            }
        }
        _ => bail!("host and caller cache roots must either both be present or both be absent"),
    }
    Ok(())
}

fn replace_exact_host_path(
    value: &mut Value,
    pointer: &str,
    expected: &Path,
    replacement: &str,
) -> Result<()> {
    let field = value
        .pointer_mut(pointer)
        .context(format!("Windows host evidence response missing {pointer}"))?;
    let observed = field.as_str().context(format!(
        "Windows host evidence response {pointer} must be a string"
    ))?;
    if !same_host_path(observed, &slash_path(expected)) {
        bail!("Windows host evidence response {pointer} does not match the request");
    }
    *field = Value::String(replacement.to_string());
    Ok(())
}

fn translate_cache_file(cache_file: &str, host_root: &Path, local_root: &Path) -> Result<String> {
    let file = normalize_slashes(cache_file);
    let root = normalize_slashes(&slash_path(host_root));
    let suffix = if file.eq_ignore_ascii_case(&root) {
        ""
    } else if file.len() > root.len()
        && file
            .get(..root.len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(&root))
        && file.as_bytes().get(root.len()) == Some(&b'/')
    {
        file.get(root.len() + 1..)
            .context("host evidence cacheFile suffix is not valid UTF-8")?
    } else {
        bail!("host evidence cacheFile escapes the requested cacheRoot");
    };
    let local = slash_path(local_root);
    Ok(if suffix.is_empty() {
        local
    } else {
        format!("{}/{suffix}", local.trim_end_matches('/'))
    })
}

fn join_host_path(root: &Path, name: &str) -> Result<String> {
    if !is_windows_absolute_path(root) {
        bail!("host result output must be an absolute Windows path");
    }
    Ok(format!(
        "{}/{}",
        slash_path(root).trim_end_matches('/'),
        name
    ))
}

fn is_windows_absolute_path(value: &Path) -> bool {
    let value = slash_path(value);
    let bytes = value.as_bytes();
    bytes.len() >= 3 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' && bytes[2] == b'/'
}

fn same_host_path(left: &str, right: &str) -> bool {
    normalize_slashes(left)
        .trim_end_matches('/')
        .eq_ignore_ascii_case(normalize_slashes(right).trim_end_matches('/'))
}

fn normalize_slashes(value: &str) -> String {
    value.replace('\\', "/")
}

fn slash_path(value: &Path) -> String {
    normalize_slashes(&value.to_string_lossy())
}

fn array_contains(value: &Value, pointer: &str, expected: &str) -> bool {
    value
        .pointer(pointer)
        .and_then(Value::as_array)
        .is_some_and(|values| values.iter().any(|value| value.as_str() == Some(expected)))
}

fn remove_file_if_present(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error).with_context(|| format!("failed to remove {}", path.display())),
    }
}

#[cfg(test)]
mod tests {
    use super::{normalize_host_evidence_paths, translate_cache_file};
    use crate::js_ts_pre_write::{
        JsTsPreWriteHostTransport, JS_TS_PRE_WRITE_EVIDENCE_RESPONSE_SCHEMA_VERSION,
        JS_TS_PRE_WRITE_HOST_TRANSPORT_SCHEMA_VERSION,
    };
    use anyhow::{anyhow, Result};
    use serde_json::json;
    use std::path::{Path, PathBuf};

    fn response(cache_file: &str) -> serde_json::Value {
        json!({
            "schemaVersion": JS_TS_PRE_WRITE_EVIDENCE_RESPONSE_SCHEMA_VERSION,
            "root": "C:/repo",
            "files": [],
            "symbols": {},
            "shapeIndex": {
                "meta": {
                    "root": "C:/repo",
                    "incremental": {
                        "cacheRoot": "C:/repo/.audit/.cache",
                        "cacheFile": cache_file,
                    },
                },
            },
            "anyInventory": {
                "meta": {
                    "root": "C:/repo",
                    "incremental": {
                        "cacheRoot": "C:/repo/.audit/.cache",
                        "cacheFile": cache_file,
                    },
                },
            },
            "summary": {"runtime": {}},
        })
    }

    fn transport() -> JsTsPreWriteHostTransport {
        JsTsPreWriteHostTransport {
            schema_version: JS_TS_PRE_WRITE_HOST_TRANSPORT_SCHEMA_VERSION.to_string(),
            command: PathBuf::from("/mnt/c/lumin-audit-core.exe"),
            root: PathBuf::from("C:/repo"),
            output: PathBuf::from("C:/repo/.audit"),
            cache_root: Some(PathBuf::from("C:/repo/.audit/.cache")),
        }
    }

    #[test]
    fn restores_caller_local_paths_after_host_execution() -> Result<()> {
        let mut evidence = response("C:/repo/.audit/.cache\\incremental\\facts.json");
        normalize_host_evidence_paths(
            &mut evidence,
            &transport(),
            Path::new("/mnt/c/repo"),
            Some(Path::new("/mnt/c/repo/.audit/.cache")),
        )?;

        assert_eq!(evidence["root"], "/mnt/c/repo");
        assert_eq!(evidence["anyInventory"]["meta"]["root"], "/mnt/c/repo");
        assert_eq!(
            evidence["shapeIndex"]["meta"]["incremental"]["cacheFile"],
            "/mnt/c/repo/.audit/.cache/incremental/facts.json"
        );
        assert_eq!(
            evidence["summary"]["runtime"]["evidenceTransport"]["kind"],
            "windows-host-audit-core"
        );
        Ok(())
    }

    #[test]
    fn rejects_cache_files_outside_the_host_cache_root() -> Result<()> {
        let error = match translate_cache_file(
            "C:/repo/outside/facts.json",
            Path::new("C:/repo/.audit/.cache"),
            Path::new("/mnt/c/repo/.audit/.cache"),
        ) {
            Ok(_) => return Err(anyhow!("escaped host cache files must fail closed")),
            Err(error) => error,
        };

        assert!(error
            .to_string()
            .contains("escapes the requested cacheRoot"));
        Ok(())
    }
}
