use super::*;

fn extract_source_with_file_path(
    file_path: &str,
    source: &str,
    source_files: Vec<&str>,
) -> Result<JsTsExtractResponse> {
    build_js_ts_extract_response(JsTsExtractRequest {
        schema_version: JS_TS_EXTRACT_REQUEST_SCHEMA_VERSION.to_string(),
        source_files: source_files.into_iter().map(str::to_string).collect(),
        files: vec![JsTsExtractInputFile {
            file_path: file_path.to_string(),
            artifact_file_path: None,
            source: Some(source.to_string()),
        }],
    })
}

fn extract_source_with_source_files(
    source: &str,
    source_files: Vec<&str>,
) -> Result<JsTsExtractResponse> {
    extract_source_with_file_path("C:/repo/src/consumer.ts", source, source_files)
}

fn extract_with_source_files(source_files: Vec<&str>) -> Result<JsTsExtractResponse> {
    extract_source_with_source_files(
        "import { view } from './view.jsx';\nconsole.log(view);\n",
        source_files,
    )
}

#[test]
fn jsx_output_import_prefers_tsx_before_ts() -> Result<()> {
    let response = extract_with_source_files(vec![
        "C:/repo/src/consumer.ts",
        "C:/repo/src/view.ts",
        "C:/repo/src/view.tsx",
    ])?;

    assert_eq!(
        response.files[0].uses[0].resolved_file.as_deref(),
        Some("C:/repo/src/view.tsx")
    );
    Ok(())
}

#[test]
fn jsx_output_import_falls_back_to_ts_when_tsx_source_is_absent() -> Result<()> {
    let response =
        extract_with_source_files(vec!["C:/repo/src/consumer.ts", "C:/repo/src/view.ts"])?;

    assert_eq!(
        response.files[0].uses[0].resolved_file.as_deref(),
        Some("C:/repo/src/view.ts")
    );
    Ok(())
}

#[test]
fn literal_dynamic_import_emits_broad_consumer_use() -> Result<()> {
    let response = extract_source_with_source_files(
        "export async function load() {\n  return import('./lazy');\n}\n",
        vec!["C:/repo/src/consumer.ts", "C:/repo/src/lazy.ts"],
    )?;

    let dynamic_use = response.files[0]
        .uses
        .iter()
        .find(|use_record| use_record.kind == "dynamic")
        .ok_or_else(|| anyhow::anyhow!("dynamic import use should be emitted"))?;
    assert_eq!(dynamic_use.from_spec, "./lazy");
    assert_eq!(dynamic_use.name, "*");
    assert!(dynamic_use.degraded);
    assert_eq!(
        dynamic_use.resolved_file.as_deref(),
        Some("C:/repo/src/lazy.ts")
    );
    Ok(())
}

#[test]
fn literal_dynamic_import_in_mjs_emits_broad_consumer_use() -> Result<()> {
    let response = extract_source_with_file_path(
        "C:/repo/src/consumer.mjs",
        "export async function load() {\n  return import('./lazy.mjs');\n}\n",
        vec!["C:/repo/src/consumer.mjs", "C:/repo/src/lazy.mjs"],
    )?;

    let dynamic_use = response.files[0]
        .uses
        .iter()
        .find(|use_record| use_record.kind == "dynamic")
        .ok_or_else(|| anyhow::anyhow!("dynamic import use should be emitted for mjs"))?;
    assert_eq!(dynamic_use.from_spec, "./lazy.mjs");
    assert_eq!(
        dynamic_use.resolved_file.as_deref(),
        Some("C:/repo/src/lazy.mjs")
    );
    Ok(())
}

#[test]
fn assigned_dynamic_import_preserves_broad_consumer_when_member_escapes() -> Result<()> {
    let response = extract_source_with_source_files(
            "export async function load() {\n  const mod = await import('web-tree-sitter');\n  Parser = mod.Parser;\n}\n",
            vec!["C:/repo/src/consumer.ts"],
        )?;

    let dynamic_use = response.files[0]
        .uses
        .iter()
        .find(|use_record| use_record.kind == "dynamic")
        .ok_or_else(|| anyhow::anyhow!("assigned dynamic import should be broad"))?;
    assert_eq!(dynamic_use.from_spec, "web-tree-sitter");
    assert_eq!(dynamic_use.name, "*");
    assert_eq!(dynamic_use.local_name.as_deref(), Some("mod"));
    assert!(dynamic_use.degraded);
    Ok(())
}

#[test]
fn assigned_dynamic_import_call_member_preserves_member_precision() -> Result<()> {
    let response = extract_source_with_source_files(
        "export async function load() {\n  const mod = await import('./lazy');\n  mod.boot();\n}\n",
        vec!["C:/repo/src/consumer.ts", "C:/repo/src/lazy.ts"],
    )?;

    let dynamic_use = response.files[0]
        .uses
        .iter()
        .find(|use_record| use_record.kind == "dynamic-member")
        .ok_or_else(|| anyhow::anyhow!("dynamic member use should be emitted"))?;
    assert_eq!(dynamic_use.from_spec, "./lazy");
    assert_eq!(dynamic_use.name, "boot");
    assert_eq!(dynamic_use.local_name.as_deref(), Some("mod"));
    assert!(!dynamic_use.degraded);
    assert_eq!(
        dynamic_use.resolved_file.as_deref(),
        Some("C:/repo/src/lazy.ts")
    );
    Ok(())
}

#[test]
fn nonliteral_dynamic_import_emits_opacity_evidence() -> Result<()> {
    let response = extract_source_with_source_files(
        "export async function load(target) {\n  return import(target);\n}\n",
        vec!["C:/repo/src/consumer.ts"],
    )?;

    assert!(response.files[0].uses.is_empty());
    assert_eq!(response.files[0].dynamic_import_opacity.len(), 1);
    assert_eq!(response.files[0].dynamic_import_opacity[0].line, 2);
    assert_eq!(
        response.files[0].dynamic_import_opacity[0].kind,
        "nonliteral"
    );
    Ok(())
}

#[test]
fn template_dynamic_import_emits_prefix_opacity_evidence() -> Result<()> {
    let response = extract_source_with_source_files(
        "export async function load(name) {\n  return import(`./pages/${name}.ts`);\n}\n",
        vec!["C:/repo/src/consumer.ts"],
    )?;

    assert!(response.files[0].uses.is_empty());
    assert_eq!(response.files[0].dynamic_import_opacity.len(), 1);
    assert_eq!(response.files[0].dynamic_import_opacity[0].line, 2);
    assert_eq!(
        response.files[0].dynamic_import_opacity[0].kind,
        "template-prefix"
    );
    assert_eq!(
        response.files[0].dynamic_import_opacity[0]
            .prefix
            .as_deref(),
        Some("./pages/")
    );
    Ok(())
}

#[test]
fn cjs_export_surface_records_exact_and_opaque_exports() -> Result<()> {
    let source = [
        "exports.foo = 1;",
        "module.exports.bar = 2;",
        "exports[\"quoted\"] = 3;",
        "module.exports = { baz: 4, renamed: localValue };",
        "exports[dynamicName] = 5;",
        "module.exports = makeExports();",
        "",
    ]
    .join("\n");
    let response = extract_source_with_file_path(
        "C:/repo/src/exporter.cjs",
        &source,
        vec!["C:/repo/src/exporter.cjs"],
    )?;

    let surface = response.files[0]
        .cjs_export_surface
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("cjs export surface should be emitted"))?;
    assert!(surface
        .exact
        .iter()
        .any(|entry| entry.name == "foo" && entry.kind == "exports-member"));
    assert!(surface
        .exact
        .iter()
        .any(|entry| entry.name == "bar" && entry.kind == "module-exports-member"));
    assert!(surface
        .exact
        .iter()
        .any(|entry| entry.name == "quoted" && entry.kind == "exports-member"));
    assert!(surface
        .exact
        .iter()
        .any(|entry| entry.name == "baz" && entry.kind == "module-exports-object"));
    assert!(surface
        .exact
        .iter()
        .any(|entry| entry.name == "renamed" && entry.kind == "module-exports-object"));
    assert!(surface
        .opaque
        .iter()
        .any(|entry| entry.kind == "computed-export-name"));
    assert!(surface
        .opaque
        .iter()
        .any(|entry| entry.kind == "module-exports-assignment"));
    Ok(())
}

#[test]
fn computed_cjs_destructuring_degrades_to_namespace_evidence() -> Result<()> {
    let source = [
        "const directKey = 'foo';",
        "const { [directKey]: direct } = require('./direct');",
        "const aliased = require('./aliased');",
        "const aliasKey = 'bar';",
        "const { [aliasKey]: value } = aliased;",
        "console.log(direct, value);",
        "",
    ]
    .join("\n");
    let response = extract_source_with_file_path(
        "C:/repo/src/consumer.cjs",
        &source,
        vec!["C:/repo/src/consumer.cjs"],
    )?;

    for from_spec in ["./direct", "./aliased"] {
        assert!(response.files[0].uses.iter().any(|use_record| {
            use_record.from_spec == from_spec
                && use_record.name == "*"
                && use_record.kind == "cjs-namespace-escape"
                && use_record.degraded
        }));
    }
    Ok(())
}
