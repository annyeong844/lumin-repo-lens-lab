use anyhow::Result;
use serde_json::{json, Value};
use std::fs;
use std::process::Command;

use lumin_audit_core::orchestration_events::{
    build_producer_performance_artifact, build_producer_performance_artifact_for_audit_run,
    build_producer_performance_artifact_for_audit_run_from_output,
    build_producer_performance_artifact_from_runtime, OrchestrationLedger,
    ProducerPerformanceAuditRunContext, ProducerPerformanceRuntimeInput,
    ProducerPerformanceRuntimeObservations,
};

fn ledger(value: Value) -> Result<OrchestrationLedger> {
    Ok(serde_json::from_value(value)?)
}

fn runtime_input(value: Value) -> Result<ProducerPerformanceRuntimeInput> {
    Ok(serde_json::from_value(value)?)
}

fn audit_run_context(value: Value) -> Result<ProducerPerformanceAuditRunContext> {
    Ok(serde_json::from_value(value)?)
}

fn runtime_observations(value: Value) -> Result<ProducerPerformanceRuntimeObservations> {
    Ok(serde_json::from_value(value)?)
}

#[test]
fn producer_performance_artifact_projects_ledger_without_js_summary_math() -> Result<()> {
    let ledger = ledger(json!({
        "schemaVersion": "lumin-audit-orchestration-ledger.v1",
        "generated": "2026-07-01T00:00:00.000Z",
        "root": "C:/repo",
        "output": "C:/repo/.audit",
        "profile": "full",
        "scanRange": {
            "includeTests": true,
            "production": false,
            "excludes": ["dist"],
            "autoExcludes": [".audit"]
        },
        "cache": {
            "noIncremental": false,
            "cacheRoot": "C:/repo/.audit/.cache",
            "clearIncrementalCache": false
        },
        "generatedArtifacts": { "mode": "prepared" },
        "artifactReads": {
            "schemaVersion": "artifact-read-metrics.v1",
            "measurement": "audit-repo-orchestrator-json-reads",
            "totalReadCount": 3,
            "totalReadBytes": 120,
            "totalReadMs": 9,
            "totalJsonParseMs": 4,
            "parseFailureCount": 1,
            "largestReads": [{ "name": "symbols.json", "bytes": 100, "readCount": 1 }],
            "slowestJsonParses": [{ "name": "symbols.json", "jsonParseMs": 4, "readCount": 1 }],
            "byName": { "symbols.json": { "readCount": 1 } }
        },
        "artifacts": {
            "producedCount": 2,
            "totalBytes": 2048,
            "largest": [{ "name": "symbols.json", "bytes": 2000 }],
            "byName": { "symbols.json": { "bytes": 2000 } }
        },
        "events": [
            {
                "kind": "producer",
                "name": "triage-repo.mjs",
                "status": "ok",
                "wallMs": 12,
                "phases": [{ "name": "scan", "ms": 7 }],
                "counters": { "files": 10 },
                "memory": {
                    "before": { "rssBytes": 100, "heapTotalBytes": 20 },
                    "after": { "rssBytes": 140, "heapTotalBytes": 24 },
                    "delta": { "rssBytes": 40, "heapTotalBytes": 4 }
                }
            },
            {
                "kind": "producer",
                "name": "rank-fixes.mjs",
                "status": "failed-optional",
                "wallMs": 5,
                "stderrSnippet": "boom"
            },
            {
                "kind": "skipped",
                "name": "emit-sarif.mjs",
                "reason": "not in --sarif mode"
            }
        ]
    }))?;
    let artifact = serde_json::to_value(build_producer_performance_artifact(ledger)?)?;

    assert_eq!(artifact["schemaVersion"], "producer-performance.v1");
    assert_eq!(artifact["profile"], "full");
    assert_eq!(artifact["summary"]["producerCount"], 2);
    assert_eq!(artifact["summary"]["okCount"], 1);
    assert_eq!(artifact["summary"]["failedCount"], 1);
    assert_eq!(artifact["summary"]["skippedCount"], 1);
    assert_eq!(artifact["summary"]["totalWallMs"], 17);
    assert_eq!(artifact["summary"]["artifactCount"], 2);
    assert_eq!(artifact["summary"]["totalArtifactBytes"], 2048);
    assert_eq!(artifact["summary"]["artifactReadCount"], 3);
    assert_eq!(artifact["summary"]["totalArtifactReadBytes"], 120);
    assert_eq!(artifact["summary"]["totalJsonParseMs"], 4);
    assert_eq!(artifact["summary"]["maxObservedOrchestratorRssBytes"], 140);
    assert_eq!(artifact["summary"]["phaseSupportCount"], 1);
    assert_eq!(
        artifact["memory"]["measurement"],
        "orchestrator-process-snapshots"
    );
    assert_eq!(artifact["memory"]["childPeakRssAvailable"], false);
    assert_eq!(artifact["producers"][1]["stderrSnippet"], "boom");
    assert_eq!(artifact["skipped"][0]["status"], "skipped");
    Ok(())
}

#[test]
fn producer_performance_artifact_rejects_wrong_ledger_schema() -> Result<()> {
    let ledger = ledger(json!({
        "schemaVersion": "old-ledger",
        "generated": "2026-07-01T00:00:00.000Z",
        "root": "C:/repo",
        "output": "C:/repo/.audit",
        "profile": "quick",
        "scanRange": { "includeTests": true, "production": false },
        "cache": {
            "noIncremental": false,
            "cacheRoot": "C:/repo/.audit/.cache",
            "clearIncrementalCache": false
        },
        "generatedArtifacts": { "mode": "default" },
        "artifactReads": {
            "schemaVersion": "artifact-read-metrics.v1",
            "measurement": "audit-repo-orchestrator-json-reads",
            "totalReadCount": 0,
            "totalReadBytes": 0,
            "totalReadMs": 0,
            "totalJsonParseMs": 0,
            "parseFailureCount": 0
        },
        "artifacts": { "producedCount": 0, "totalBytes": 0 },
        "events": []
    }))?;
    let error = match build_producer_performance_artifact(ledger) {
        Ok(_) => anyhow::bail!("wrong ledger schema should be rejected"),
        Err(error) => error,
    };

    assert!(error
        .to_string()
        .contains("unsupported ledger schemaVersion"));
    Ok(())
}

#[test]
fn runtime_input_projects_phase_sidecars_artifact_sizes_and_runtime_events() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let output = temp.path().join("out");
    fs::create_dir_all(output.join(".producer-phases"))?;
    fs::write(output.join("triage.json"), "{}")?;
    fs::write(
        output.join(".producer-phases").join("triage-repo.mjs.json"),
        serde_json::to_vec(&json!({
            "schemaVersion": "producer-phase-timing.v1",
            "producer": "triage-repo.mjs",
            "phases": [
                { "name": "scan", "wallMs": 7.4 },
                { "name": "", "wallMs": -2 }
            ],
            "counters": { "files": 10.2, "ignored": "bad" }
        }))?,
    )?;

    let input = runtime_input(json!({
        "schemaVersion": "lumin-audit-producer-performance-runtime.v1",
        "generated": "2026-07-01T00:00:00.000Z",
        "root": temp.path().to_string_lossy(),
        "output": output.to_string_lossy(),
        "profile": "quick",
        "scanRange": { "includeTests": true, "production": false },
        "cache": {
            "noIncremental": false,
            "cacheRoot": output.join(".cache").to_string_lossy(),
            "clearIncrementalCache": false
        },
        "generatedArtifacts": { "mode": "default" },
        "artifactReads": {
            "schemaVersion": "artifact-read-metrics.v1",
            "measurement": "audit-repo-orchestrator-json-reads",
            "totalReadCount": 1,
            "totalReadBytes": 2,
            "totalReadMs": 3,
            "totalJsonParseMs": 4,
            "parseFailureCount": 0,
            "largestReads": [],
            "slowestJsonParses": [],
            "byName": {
                "manifest.json": {
                    "readCount": 1,
                    "totalBytes": 2,
                    "totalReadMs": 3,
                    "totalJsonParseMs": 4,
                    "parseFailureCount": 0
                }
            }
        },
        "artifactsProduced": ["triage.json", "missing.json"],
        "commandsRun": [
            {
                "step": "triage-repo.mjs",
                "status": "ok",
                "ms": 12,
                "memory": {
                    "before": { "rssBytes": 100 },
                    "after": { "rssBytes": 120 },
                    "delta": { "rssBytes": 20 }
                }
            }
        ],
        "skipped": [
            { "step": "emit-sarif.mjs", "reason": "not in --sarif mode" }
        ]
    }))?;

    let artifact = serde_json::to_value(build_producer_performance_artifact_from_runtime(input)?)?;

    assert_eq!(artifact["schemaVersion"], "producer-performance.v1");
    assert_eq!(artifact["summary"]["producerCount"], 1);
    assert_eq!(artifact["summary"]["skippedCount"], 1);
    assert_eq!(artifact["summary"]["artifactCount"], 1);
    assert_eq!(artifact["summary"]["artifactReadCount"], 2);
    assert_eq!(artifact["summary"]["phaseSupportCount"], 1);
    assert_eq!(artifact["producers"][0]["phases"][0]["name"], "scan");
    assert_eq!(artifact["producers"][0]["phases"][0]["wallMs"], 7);
    assert_eq!(artifact["producers"][0]["counters"]["files"], 10);
    assert_eq!(
        artifact["artifactReads"]["byName"][".producer-phases/triage-repo.mjs.json"]["readCount"],
        1
    );
    assert_eq!(artifact["skipped"][0]["name"], "emit-sarif.mjs");
    Ok(())
}

#[test]
fn audit_run_context_projects_scan_cache_and_runtime_observations() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let output = temp.path().join("out");
    fs::create_dir_all(&output)?;
    fs::write(output.join("triage.json"), "{}")?;

    let context = audit_run_context(json!({
        "generated": "2026-07-01T00:00:00.000Z",
        "root": temp.path().to_string_lossy(),
        "output": output.to_string_lossy(),
        "profile": "full",
        "includeTests": false,
        "production": true,
        "excludes": ["dist"],
        "autoExcludes": [".audit"],
        "noIncremental": true,
        "cacheRoot": output.join(".cache").to_string_lossy(),
        "clearIncrementalCache": true,
        "generatedArtifactsMode": "prepared"
    }))?;
    let observations = runtime_observations(json!({
        "artifactReads": {
            "schemaVersion": "artifact-read-metrics.v1",
            "measurement": "audit-repo-orchestrator-json-reads",
            "totalReadCount": 1,
            "totalReadBytes": 2,
            "totalReadMs": 3,
            "totalJsonParseMs": 4,
            "parseFailureCount": 0,
            "byName": {}
        },
        "artifactsProduced": ["triage.json"],
        "commandsRun": [
            { "step": "triage-repo.mjs", "status": "ok", "ms": 3 }
        ],
        "skipped": [
            { "step": "emit-sarif.mjs", "reason": "not in --sarif mode" }
        ]
    }))?;

    let artifact = serde_json::to_value(build_producer_performance_artifact_for_audit_run(
        context,
        observations,
    )?)?;

    assert_eq!(artifact["schemaVersion"], "producer-performance.v1");
    assert_eq!(artifact["profile"], "full");
    assert_eq!(artifact["scanRange"]["includeTests"], false);
    assert_eq!(artifact["scanRange"]["production"], true);
    assert_eq!(artifact["scanRange"]["excludes"], json!(["dist"]));
    assert_eq!(artifact["scanRange"]["autoExcludes"], json!([".audit"]));
    assert_eq!(artifact["cache"]["noIncremental"], true);
    assert_eq!(artifact["cache"]["clearIncrementalCache"], true);
    assert_eq!(artifact["generatedArtifacts"]["mode"], "prepared");
    assert_eq!(artifact["summary"]["producerCount"], 1);
    assert_eq!(artifact["summary"]["skippedCount"], 1);
    assert_eq!(artifact["summary"]["artifactCount"], 1);
    Ok(())
}

#[test]
fn audit_run_context_collects_artifacts_from_output_and_rust_analysis() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let output = temp.path().join("out");
    fs::create_dir_all(&output)?;
    fs::write(output.join("triage.json"), "{}")?;
    fs::write(output.join("rust-analyzer-health.latest.json"), "{}")?;

    let context = audit_run_context(json!({
        "generated": "2026-07-01T00:00:00.000Z",
        "root": temp.path().to_string_lossy(),
        "output": output.to_string_lossy(),
        "profile": "quick",
        "includeTests": true,
        "production": false,
        "noIncremental": false,
        "cacheRoot": output.join(".cache").to_string_lossy(),
        "clearIncrementalCache": false,
        "generatedArtifactsMode": "default"
    }))?;
    let observations = runtime_observations(json!({
        "artifactReads": {
            "schemaVersion": "artifact-read-metrics.v1",
            "measurement": "audit-repo-orchestrator-json-reads",
            "totalReadCount": 0,
            "totalReadBytes": 0,
            "totalReadMs": 0,
            "totalJsonParseMs": 0,
            "parseFailureCount": 0,
            "byName": {}
        },
        "rustAnalysis": {
            "status": "complete",
            "available": true
        },
        "commandsRun": [
            { "step": "triage-repo.mjs", "status": "ok", "ms": 3 }
        ],
        "skipped": []
    }))?;

    let artifact = serde_json::to_value(
        build_producer_performance_artifact_for_audit_run_from_output(context, observations)?,
    )?;

    assert_eq!(artifact["summary"]["artifactCount"], 2);
    assert_eq!(artifact["artifacts"]["byName"]["triage.json"]["bytes"], 2);
    assert_eq!(
        artifact["artifacts"]["byName"]["rust-analyzer-health.latest.json"]["bytes"],
        2
    );
    Ok(())
}

#[test]
fn runtime_input_records_malformed_phase_sidecar_as_read_failure_without_phase_claim() -> Result<()>
{
    let temp = tempfile::tempdir()?;
    let output = temp.path().join("out");
    fs::create_dir_all(output.join(".producer-phases"))?;
    fs::write(
        output.join(".producer-phases").join("triage-repo.mjs.json"),
        "{not json",
    )?;

    let input = runtime_input(json!({
        "schemaVersion": "lumin-audit-producer-performance-runtime.v1",
        "generated": "2026-07-01T00:00:00.000Z",
        "root": temp.path().to_string_lossy(),
        "output": output.to_string_lossy(),
        "profile": "quick",
        "scanRange": { "includeTests": true, "production": false },
        "cache": {
            "noIncremental": false,
            "cacheRoot": output.join(".cache").to_string_lossy(),
            "clearIncrementalCache": false
        },
        "generatedArtifacts": { "mode": "default" },
        "artifactReads": {
            "schemaVersion": "artifact-read-metrics.v1",
            "measurement": "audit-repo-orchestrator-json-reads",
            "totalReadCount": 0,
            "totalReadBytes": 0,
            "totalReadMs": 0,
            "totalJsonParseMs": 0,
            "parseFailureCount": 0,
            "byName": {}
        },
        "artifactsProduced": [],
        "commandsRun": [{ "step": "triage-repo.mjs", "status": "ok", "ms": 1 }],
        "skipped": []
    }))?;

    let artifact = serde_json::to_value(build_producer_performance_artifact_from_runtime(input)?)?;

    assert!(artifact["producers"][0].get("phases").is_none());
    assert_eq!(artifact["artifactReads"]["parseFailureCount"], 1);
    assert_eq!(
        artifact["artifactReads"]["byName"][".producer-phases/triage-repo.mjs.json"]
            ["parseFailureCount"],
        1
    );
    Ok(())
}

#[test]
fn cli_producer_performance_artifact_emits_json() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input_path = temp.path().join("ledger.json");
    fs::write(
        &input_path,
        serde_json::to_vec(&json!({
            "schemaVersion": "lumin-audit-orchestration-ledger.v1",
            "generated": "2026-07-01T00:00:00.000Z",
            "root": "C:/repo",
            "output": "C:/repo/.audit",
            "profile": "quick",
            "scanRange": { "includeTests": true, "production": false },
            "cache": {
                "noIncremental": false,
                "cacheRoot": "C:/repo/.audit/.cache",
                "clearIncrementalCache": false
            },
            "generatedArtifacts": { "mode": "default" },
            "artifactReads": {
                "schemaVersion": "artifact-read-metrics.v1",
                "measurement": "audit-repo-orchestrator-json-reads",
                "totalReadCount": 0,
                "totalReadBytes": 0,
                "totalReadMs": 0,
                "totalJsonParseMs": 0,
                "parseFailureCount": 0
            },
            "artifacts": { "producedCount": 0, "totalBytes": 0 },
            "events": [
                { "kind": "producer", "name": "triage-repo.mjs", "status": "ok", "wallMs": 3 }
            ]
        }))?,
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("producer-performance-artifact")
        .arg("--input")
        .arg(&input_path)
        .output()?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = serde_json::from_slice::<Value>(&output.stdout)?;
    assert_eq!(stdout["schemaVersion"], "producer-performance.v1");
    assert_eq!(stdout["summary"]["okCount"], 1);
    Ok(())
}

#[test]
fn cli_producer_performance_runtime_artifact_emits_json() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let output_dir = temp.path().join("out");
    fs::create_dir_all(&output_dir)?;
    let input_path = temp.path().join("runtime.json");
    fs::write(
        &input_path,
        serde_json::to_vec(&json!({
            "schemaVersion": "lumin-audit-producer-performance-runtime.v1",
            "generated": "2026-07-01T00:00:00.000Z",
            "root": temp.path().to_string_lossy(),
            "output": output_dir.to_string_lossy(),
            "profile": "quick",
            "scanRange": { "includeTests": true, "production": false },
            "cache": {
                "noIncremental": false,
                "cacheRoot": output_dir.join(".cache").to_string_lossy(),
                "clearIncrementalCache": false
            },
            "generatedArtifacts": { "mode": "default" },
            "artifactReads": {
                "schemaVersion": "artifact-read-metrics.v1",
                "measurement": "audit-repo-orchestrator-json-reads",
                "totalReadCount": 0,
                "totalReadBytes": 0,
                "totalReadMs": 0,
                "totalJsonParseMs": 0,
                "parseFailureCount": 0,
                "byName": {}
            },
            "commandsRun": [
                { "step": "triage-repo.mjs", "status": "ok", "ms": 3 }
            ],
            "skipped": []
        }))?,
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("producer-performance-runtime-artifact")
        .arg("--input")
        .arg(&input_path)
        .output()?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = serde_json::from_slice::<Value>(&output.stdout)?;
    assert_eq!(stdout["schemaVersion"], "producer-performance.v1");
    assert_eq!(stdout["summary"]["okCount"], 1);
    Ok(())
}

#[test]
fn cli_producer_performance_audit_run_artifact_emits_json() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let output_dir = temp.path().join("out");
    fs::create_dir_all(&output_dir)?;
    fs::write(output_dir.join("triage.json"), "{}")?;
    fs::write(output_dir.join("rust-analyzer-health.latest.json"), "{}")?;
    let input_path = temp.path().join("runtime-observations.json");
    fs::write(
        &input_path,
        serde_json::to_vec(&json!({
            "artifactReads": {
                "schemaVersion": "artifact-read-metrics.v1",
                "measurement": "audit-repo-orchestrator-json-reads",
                "totalReadCount": 0,
                "totalReadBytes": 0,
                "totalReadMs": 0,
                "totalJsonParseMs": 0,
                "parseFailureCount": 0,
                "byName": {}
            },
            "rustAnalysis": {
                "status": "complete",
                "available": true
            },
            "commandsRun": [
                { "step": "triage-repo.mjs", "status": "ok", "ms": 3 }
            ],
            "skipped": []
        }))?,
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("producer-performance-audit-run-artifact")
        .arg("--input")
        .arg(&input_path)
        .arg("--generated")
        .arg("2026-07-01T00:00:00.000Z")
        .arg("--root")
        .arg(temp.path())
        .arg("--output")
        .arg(&output_dir)
        .arg("--profile")
        .arg("quick")
        .arg("--include-tests")
        .arg("--no-production")
        .arg("--exclude")
        .arg("dist")
        .arg("--auto-exclude")
        .arg(".audit")
        .arg("--no-incremental")
        .arg("--cache-root")
        .arg(output_dir.join(".cache"))
        .arg("--clear-incremental-cache")
        .arg("--generated-artifacts")
        .arg("default")
        .output()?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = serde_json::from_slice::<Value>(&output.stdout)?;
    assert_eq!(stdout["schemaVersion"], "producer-performance.v1");
    assert_eq!(stdout["scanRange"]["excludes"], json!(["dist"]));
    assert_eq!(stdout["summary"]["okCount"], 1);
    assert_eq!(stdout["summary"]["artifactCount"], 2);
    assert_eq!(
        stdout["artifacts"]["byName"]["rust-analyzer-health.latest.json"]["bytes"],
        2
    );
    Ok(())
}

#[test]
fn cli_producer_performance_audit_run_artifact_defaults_optional_scan_flags() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let output_dir = temp.path().join("out");
    fs::create_dir_all(&output_dir)?;
    let input_path = temp.path().join("runtime-observations.json");
    fs::write(
        &input_path,
        serde_json::to_vec(&json!({
            "artifactReads": {
                "schemaVersion": "artifact-read-metrics.v1",
                "measurement": "audit-repo-orchestrator-json-reads",
                "totalReadCount": 0,
                "totalReadBytes": 0,
                "totalReadMs": 0,
                "totalJsonParseMs": 0,
                "parseFailureCount": 0,
                "byName": {}
            },
            "commandsRun": [],
            "skipped": []
        }))?,
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("producer-performance-audit-run-artifact")
        .arg("--input")
        .arg(&input_path)
        .arg("--generated")
        .arg("2026-07-01T00:00:00.000Z")
        .arg("--root")
        .arg(temp.path())
        .arg("--output")
        .arg(&output_dir)
        .arg("--profile")
        .arg("quick")
        .arg("--cache-root")
        .arg(output_dir.join(".cache"))
        .output()?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = serde_json::from_slice::<Value>(&output.stdout)?;
    assert_eq!(stdout["scanRange"]["includeTests"], true);
    assert_eq!(stdout["scanRange"]["production"], false);
    assert_eq!(stdout["generatedArtifacts"]["mode"], "default");
    Ok(())
}

#[test]
fn cli_producer_performance_audit_run_artifact_requires_artifact_reads() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let output_dir = temp.path().join("out");
    fs::create_dir_all(&output_dir)?;
    let input_path = temp.path().join("runtime-observations.json");
    fs::write(
        &input_path,
        serde_json::to_vec(&json!({
            "commandsRun": [],
            "skipped": []
        }))?,
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("producer-performance-audit-run-artifact")
        .arg("--input")
        .arg(&input_path)
        .arg("--generated")
        .arg("2026-07-01T00:00:00.000Z")
        .arg("--root")
        .arg(temp.path())
        .arg("--output")
        .arg(&output_dir)
        .arg("--profile")
        .arg("quick")
        .arg("--cache-root")
        .arg(output_dir.join(".cache"))
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid runtime observation shape"));
    assert!(stderr.contains("artifactReads"));
    Ok(())
}

#[test]
fn cli_producer_performance_audit_run_artifact_rejects_unknown_generated_mode() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let output_dir = temp.path().join("out");
    fs::create_dir_all(&output_dir)?;
    let input_path = temp.path().join("runtime-observations.json");
    fs::write(
        &input_path,
        serde_json::to_vec(&json!({
            "artifactReads": {
                "schemaVersion": "artifact-read-metrics.v1",
                "measurement": "audit-repo-orchestrator-json-reads",
                "totalReadCount": 0,
                "totalReadBytes": 0,
                "totalReadMs": 0,
                "totalJsonParseMs": 0,
                "parseFailureCount": 0,
                "byName": {}
            },
            "commandsRun": [],
            "skipped": []
        }))?,
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("producer-performance-audit-run-artifact")
        .arg("--input")
        .arg(&input_path)
        .arg("--generated")
        .arg("2026-07-01T00:00:00.000Z")
        .arg("--root")
        .arg(temp.path())
        .arg("--output")
        .arg(&output_dir)
        .arg("--profile")
        .arg("quick")
        .arg("--cache-root")
        .arg(output_dir.join(".cache"))
        .arg("--generated-artifacts")
        .arg("typo")
        .output()?;

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("unsupported --generated-artifacts mode: typo"));
    Ok(())
}

#[test]
fn cli_producer_performance_artifact_hard_stops_on_malformed_ledger() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input_path = temp.path().join("ledger.json");
    fs::write(
        &input_path,
        r#"{"schemaVersion":"lumin-audit-orchestration-ledger.v1"}"#,
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("producer-performance-artifact")
        .arg("--input")
        .arg(&input_path)
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("producer-performance-artifact: invalid ledger shape"));
    Ok(())
}
