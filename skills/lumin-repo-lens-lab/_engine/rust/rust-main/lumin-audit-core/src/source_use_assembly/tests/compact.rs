use super::*;

#[test]
fn assembles_path_table_compacted_record_paths() {
    let response = response(must_request(json!({
        "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
        "root": "C:/repo",
        "pathTable": ["src/a.ts", "src/b.ts"],
        "records": [{
            "recordId": "r0",
            "consumerFileId": 0,
            "resolvedFileId": 1,
            "fromSpec": "./b",
            "name": "thing",
            "kind": "import",
            "resolverStage": "resolved-internal"
        }]
    })));

    assert_eq!(response.summary.handled_count, 1);
    assert_eq!(response.resolved_internal_edges[0].from, "src/a.ts");
    assert_eq!(response.resolved_internal_edges[0].to, "src/b.ts");
    assert_eq!(response.direct_consumers[0].consumer_file, "src/a.ts");
    assert_eq!(
        response.resolved_record_targets[0].resolved_file,
        "src/b.ts"
    );
}

#[test]
fn synthesizes_missing_record_ids_by_input_order() {
    let response = response(must_request(json!({
        "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
        "root": "C:/repo",
        "pathTable": ["src/a.ts", "src/b.ts"],
        "records": [{
            "consumerFileId": 0,
            "resolvedFileId": 1,
            "fromSpec": "./b",
            "name": "thing",
            "kind": "import",
            "resolverStage": "resolved-internal"
        }]
    })));

    assert_eq!(response.summary.handled_count, 1);
    assert_eq!(response.handled_record_ids[0], "r0");
    assert_eq!(response.resolved_record_targets[0].record_id, "r0");
    assert_eq!(
        response.resolved_record_targets[0].resolved_file,
        "src/b.ts"
    );
}

#[test]
fn assembles_enum_table_compacted_record_fields() {
    let response = response(must_request(json!({
        "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
        "root": "C:/repo",
        "pathTable": ["src/a.ts", "src/b.ts"],
        "kindTable": ["import"],
        "resolverStageTable": ["resolved-internal"],
        "consumerSourceTable": ["mdx-import"],
        "specifierTable": ["./b"],
        "records": [{
            "recordId": "r0",
            "consumerFileId": 0,
            "resolvedFileId": 1,
            "fromSpecId": 0,
            "name": "thing",
            "kindId": 0,
            "resolverStageId": 0,
            "consumerSourceId": 0
        }]
    })));

    assert_eq!(response.summary.handled_count, 1);
    assert_eq!(response.counters.mdx_consumer_uses, 1);
    assert_eq!(response.resolved_internal_edges[0].kind, "import-named");
    assert_eq!(response.resolved_internal_edges[0].from, "src/a.ts");
    assert_eq!(
        response.resolved_internal_edges[0].source.as_deref(),
        Some("./b")
    );
    assert_eq!(response.direct_consumers[0].symbol, "thing");
}

#[test]
fn assembles_compact_record_rows() {
    let response = response(must_request(json!({
        "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
        "root": "C:/repo",
        "pathTable": ["src/a.ts", "src/b.ts"],
        "kindTable": ["import"],
        "resolverStageTable": ["resolved-internal"],
        "specifierTable": ["./b"],
        "nameTable": ["thing"],
        "recordRowFields": [
            "consumerFileId",
            "resolvedFileId",
            "fromSpecId",
            "nameId",
            "kindId",
            "typeOnlyState",
            "line",
            "resolverStageId"
        ],
        "recordRows": [[0, 1, 0, 0, 0, 1, 7, 0]]
    })));

    assert_eq!(response.summary.record_count, 1);
    assert_eq!(response.summary.handled_count, 1);
    assert_eq!(response.handled_record_ids[0], "r0");
    assert_eq!(response.resolved_internal_edges[0].from, "src/a.ts");
    assert_eq!(response.resolved_internal_edges[0].to, "src/b.ts");
    assert_eq!(response.resolved_internal_edges[0].line, Some(7));
    assert!(!response.resolved_internal_edges[0].type_only);
    assert_eq!(response.direct_consumers[0].symbol, "thing");
}

#[test]
fn resolves_relative_targets_from_compacted_source_file_ids() {
    let response = response(must_request(json!({
        "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
        "root": "C:/repo",
        "pathTable": ["src/consumer.ts", "src/dep.ts"],
        "sourceFileIds": [0, 1],
        "records": [{
            "recordId": "src/consumer.ts#0",
            "consumerFileId": 0,
            "fromSpec": "./dep",
            "name": "value",
            "kind": "import",
            "resolverStage": "relative"
        }]
    })));

    assert_eq!(response.summary.handled_count, 1);
    assert_eq!(response.counters.rust_resolved_relative_uses, 1);
    assert_eq!(response.resolved_internal_edges[0].to, "src/dep.ts");
    assert_eq!(
        response.resolved_record_targets[0].resolved_file,
        "C:/repo/src/dep.ts"
    );
}
