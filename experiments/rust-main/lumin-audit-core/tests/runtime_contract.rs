use anyhow::{Context, Result};
use serde_json::Value;
use std::process::Command;

#[test]
fn cli_runtime_contract_reports_js_bridge_capabilities() -> Result<()> {
    let output = Command::new(audit_core_bin())
        .arg("runtime-contract")
        .output()?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());

    let contract: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(
        contract["schemaVersion"],
        "lumin-audit-core-runtime-contract.v1"
    );
    assert_eq!(
        contract["contractVersion"],
        "audit-core-js-runtime-bridge.v40"
    );
    assert_eq!(contract["features"]["resultOutput"], true);
    assert_eq!(contract["features"]["resultOutputSilencesStdout"], true);
    assert_eq!(contract["features"]["jsTsExtractNamedImportEvidence"], true);
    assert_eq!(
        contract["features"]["jsTsExtractImportMetaGlobEvidence"],
        true
    );
    assert_eq!(
        contract["features"]["jsTsExtractLiteralDynamicImportEvidence"],
        true
    );
    assert_eq!(
        contract["features"]["jsTsExtractDynamicImportOpacity"],
        true
    );
    assert_eq!(contract["features"]["jsTsExtractLocalOperations"], true);
    assert_eq!(contract["features"]["sourceUseAssembly"], true);
    assert_eq!(
        contract["features"]["sourceUseAssemblyResolvedRecordTargets"],
        true
    );
    assert_eq!(
        contract["features"]["nonSourceAssetSourceUseAssembly"],
        true
    );
    assert_eq!(
        contract["features"]["sourceUseAssemblyConsumerSourceCounters"],
        true
    );
    assert_eq!(
        contract["features"]["sourceUseAssemblyRootRelativeSourceFiles"],
        true
    );
    assert_eq!(contract["features"]["sourceUseAssemblySourceFileIds"], true);
    assert_eq!(
        contract["features"]["sourceUseAssemblyRootRelativeRecordPaths"],
        true
    );
    assert_eq!(
        contract["features"]["sourceUseAssemblySyntheticRecordIds"],
        true
    );
    assert_eq!(contract["features"]["symbolGraphStrictRequestV2"], true);
    assert_eq!(
        contract["features"]["generatedVirtualSourceUseAssembly"],
        true
    );
    assert_eq!(
        contract["features"]["importMetaGlobSourceUseAssembly"],
        true
    );
    assert_eq!(contract["features"]["sfcScriptSrcSourceUseAssembly"], true);
    assert_eq!(contract["features"]["sharedSourceInventory"], true);
    assert_eq!(contract["features"]["sourceInventoryRunBinding"], true);
    assert_eq!(contract["features"]["failClosedLifecycleArtifacts"], true);
    assert_eq!(contract["features"]["postWriteOnlyBasePipelineSkip"], true);
    assert_eq!(contract["features"]["postWriteScopedBaseEvidence"], true);
    assert_eq!(contract["features"]["sourceUseAssemblyRecordRows"], true);
    assert_eq!(contract["features"]["sourceUseAssemblyNameTable"], true);
    assert_eq!(contract["features"]["sourceUseAssemblyTypeOnlyState"], true);
    assert_eq!(
        contract["features"]["sourceUseAssemblyDerivedReExportMaps"],
        true
    );
    assert_eq!(
        contract["features"]["sourceUseAssemblyTerminalRecordOutcomes"],
        true
    );

    let supported = contract["supportedSubcommands"]
        .as_array()
        .context("supportedSubcommands array")?;
    assert!(supported.iter().any(|item| item == "runtime-contract"));
    assert!(supported.iter().any(|item| item == "symbol-graph-artifact"));
    assert!(supported
        .iter()
        .any(|item| item == "execute-audit-lifecycle"));

    let result_output = contract["resultOutputSubcommands"]
        .as_array()
        .context("resultOutputSubcommands array")?;
    assert!(result_output
        .iter()
        .any(|item| item == "symbol-graph-artifact"));
    assert!(result_output
        .iter()
        .any(|item| item == "execute-audit-lifecycle"));
    assert!(result_output
        .iter()
        .any(|item| item == "execute-rust-pre-write"));
    assert!(result_output
        .iter()
        .any(|item| item == "execute-post-write"));
    assert!(result_output
        .iter()
        .any(|item| item == "framework-resource-surfaces-artifact"));
    assert!(result_output
        .iter()
        .any(|item| item == "unused-deps-artifact"));
    Ok(())
}

fn audit_core_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lumin-audit-core")
}
