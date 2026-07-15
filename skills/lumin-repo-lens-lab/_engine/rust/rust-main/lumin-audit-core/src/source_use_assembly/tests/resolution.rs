use super::*;

#[test]
fn exposes_relative_resolution_targets_for_js_embedding() {
    let response = response(must_request(json!({
        "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
        "root": "C:/repo",
        "sourceFiles": ["C:/repo/src/consumer.ts", "C:/repo/src/dep.ts"],
        "records": [{
            "recordId": "src/consumer.ts#0",
            "consumerFile": "C:/repo/src/consumer.ts",
            "fromSpec": "./dep",
            "name": "value",
            "kind": "import",
            "resolverStage": "relative"
        }]
    })));

    assert_eq!(response.summary.handled_count, 1);
    assert_eq!(response.counters.rust_resolved_relative_uses, 1);
    assert_eq!(response.resolved_record_targets.len(), 1);
    assert_eq!(
        response.resolved_record_targets[0].record_id,
        "src/consumer.ts#0"
    );
    assert_eq!(
        response.resolved_record_targets[0].resolved_file,
        "C:/repo/src/dep.ts"
    );
}

#[test]
fn resolves_relative_targets_from_source_files_when_resolved_file_is_absent() {
    let request = must_request(json!({
        "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
        "root": "C:/repo",
        "sourceFiles": [
            "C:/repo/src/consumer.ts",
            "C:/repo/src/dep.ts"
        ],
        "records": [
            {
                "recordId": "src/consumer.ts#0",
                "consumerFile": "C:/repo/src/consumer.ts",
                "fromSpec": "./dep",
                "name": "value",
                "kind": "import"
            }
        ]
    }));
    let response = response(request);

    assert_eq!(response.summary.handled_count, 1);
    assert_eq!(response.resolved_internal_edges[0].to, "src/dep.ts");
    assert_eq!(response.direct_consumers[0].symbol, "value");
}

#[test]
fn resolves_absolute_consumers_from_root_relative_source_files() {
    let request = must_request(json!({
        "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
        "root": "C:/repo",
        "sourceFiles": [
            "src/consumer.ts",
            "src/dep.ts"
        ],
        "records": [
            {
                "recordId": "src/consumer.ts#0",
                "consumerFile": "C:/repo/src/consumer.ts",
                "fromSpec": "./dep",
                "name": "value",
                "kind": "import"
            }
        ]
    }));
    let response = response(request);

    assert_eq!(response.summary.handled_count, 1);
    assert_eq!(response.resolved_internal_edges[0].to, "src/dep.ts");
    assert_eq!(
        response.resolved_record_targets[0].resolved_file,
        "C:/repo/src/dep.ts"
    );
}

#[test]
fn accepts_pre_resolved_non_relative_dotted_aliases_as_source_modules() {
    let request = must_request(json!({
        "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
        "root": "C:/repo",
        "records": [
            {
                "recordId": "app/(doc)/layout.tsx#0",
                "consumerFile": "C:/repo/app/(doc)/layout.tsx",
                "resolvedFile": "C:/repo/app/layout.config.ts",
                "fromSpec": "@/app/layout.config",
                "name": "baseOptions",
                "kind": "import",
                "resolverStage": "resolved-internal"
            }
        ]
    }));
    let response = response(request);

    assert_eq!(response.summary.handled_count, 1);
    assert!(response.skipped_records.is_empty());
    assert_eq!(
        response.resolved_internal_edges[0].from,
        "app/(doc)/layout.tsx"
    );
    assert_eq!(
        response.resolved_internal_edges[0].to,
        "app/layout.config.ts"
    );
}

#[test]
fn jsx_output_import_preserves_jsx_to_tsx_swap_order() {
    let request = must_request(json!({
        "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
        "root": "C:/repo",
        "sourceFiles": [
            "C:/repo/src/consumer.ts",
            "C:/repo/src/view.ts",
            "C:/repo/src/view.tsx"
        ],
        "records": [
            {
                "recordId": "src/consumer.ts#0",
                "consumerFile": "C:/repo/src/consumer.ts",
                "fromSpec": "./view.jsx",
                "name": "view",
                "kind": "import"
            }
        ]
    }));
    let response = response(request);

    assert_eq!(response.summary.handled_count, 1);
    assert_eq!(response.resolved_internal_edges[0].to, "src/view.tsx");
}

#[test]
fn jsx_output_import_falls_back_to_ts_when_tsx_source_is_absent() {
    let request = must_request(json!({
        "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
        "root": "C:/repo",
        "sourceFiles": [
            "C:/repo/src/consumer.ts",
            "C:/repo/src/view.ts"
        ],
        "records": [
            {
                "recordId": "src/consumer.ts#0",
                "consumerFile": "C:/repo/src/consumer.ts",
                "fromSpec": "./view.jsx",
                "name": "view",
                "kind": "import"
            }
        ]
    }));
    let response = response(request);

    assert_eq!(response.summary.handled_count, 1);
    assert_eq!(response.resolved_internal_edges[0].to, "src/view.ts");
}

#[test]
fn unresolved_relative_targets_are_left_for_js_fallback() {
    let request = must_request(json!({
        "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
        "root": "C:/repo",
        "sourceFiles": ["C:/repo/src/consumer.ts"],
        "records": [
            {
                "recordId": "src/consumer.ts#0",
                "consumerFile": "C:/repo/src/consumer.ts",
                "fromSpec": "./missing",
                "name": "value",
                "kind": "import"
            }
        ]
    }));
    let response = response(request);

    assert_eq!(response.summary.handled_count, 0);
    assert_eq!(
        response.skipped_records[0].reason,
        "relative-target-missing"
    );
}

#[test]
fn embedded_relative_target_missing_becomes_unresolved_evidence() {
    let request = must_request(json!({
        "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
        "root": "C:/repo",
        "sourceFiles": ["C:/repo/src/consumer.ts"],
        "records": [
            {
                "recordId": "src/consumer.ts#0",
                "consumerFile": "C:/repo/src/consumer.ts",
                "fromSpec": "./missing",
                "name": "value",
                "kind": "import"
            }
        ]
    }));
    let response = match build_embedded_source_use_assembly_response(request) {
        Ok(response) => response,
        Err(error) => panic!("embedded test response must build: {error}"),
    };

    assert_eq!(response.summary.handled_count, 1);
    assert!(response.skipped_records.is_empty());
    assert_eq!(response.counters.unresolved_uses, 1);
    assert_eq!(response.counters.unresolved_internal_uses, 1);
    assert!(response
        .unresolved_internal_specifiers
        .contains("./missing"));
    assert_eq!(response.unresolved_internal_specifier_records.len(), 1);
    assert_eq!(
        response.unresolved_internal_specifier_records[0]["reason"],
        "relative-target-missing"
    );
    assert_eq!(
        response.unresolved_internal_specifier_records[0]["resolverStage"],
        "relative"
    );
}
