use super::*;

#[test]
fn derives_reexport_maps_from_current_records_before_namespace_projection() {
    let request = must_request(json!({
        "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
        "root": "C:/repo",
        "sourceFiles": [
            "src/consumer.ts",
            "src/barrel.ts",
            "src/nested.ts",
            "src/dep.ts"
        ],
        "records": [
            {
                "recordId": "consumer",
                "consumerFile": "src/consumer.ts",
                "fromSpec": "./barrel",
                "name": "api",
                "memberName": "run",
                "kind": "imported-namespace-member",
                "resolverStage": "relative"
            },
            {
                "recordId": "named-map",
                "consumerFile": "src/barrel.ts",
                "fromSpec": "./nested",
                "resolvedFile": "src/nested.ts",
                "name": "api",
                "kind": "reExport",
                "resolverStage": "resolved-internal"
            },
            {
                "recordId": "namespace-map",
                "consumerFile": "src/nested.ts",
                "fromSpec": "./dep",
                "name": "api",
                "kind": "reExportNamespace",
                "resolverStage": "relative"
            }
        ]
    }));

    let response = response(request);
    assert!(response
        .direct_consumers
        .iter()
        .any(|consumer| consumer.consumer_file == "src/consumer.ts"
            && consumer.def_file == "src/dep.ts"
            && consumer.symbol == "run"));
    assert_eq!(response.summary.skipped_count, 0);
}

#[test]
fn handles_namespace_reexport_miss_and_skips_non_relative_records() {
    let response = response(request(json!([
        {
            "recordId": "a",
            "consumerFile": "C:/repo/src/a.ts",
            "resolvedFile": "C:/repo/src/b.ts",
            "fromSpec": "./b",
            "name": "api",
            "kind": "imported-namespace-member",
            "resolverStage": "relative"
        },
        {
            "recordId": "b",
            "consumerFile": "C:/repo/src/a.ts",
            "resolvedFile": "C:/repo/src/b.ts",
            "fromSpec": "./b",
            "kind": "import",
            "resolverStage": "alias"
        }
    ])));

    assert_eq!(response.summary.handled_count, 1);
    assert_eq!(response.summary.skipped_count, 1);
    assert_eq!(response.branch_counts["namespaceReExport"], 1);
    assert_eq!(response.branch_counts["namespaceReExportMiss"], 1);
    assert_eq!(
        response.skipped_records[0].reason,
        "non-relative-resolver-stage"
    );
}

#[test]
fn resolves_namespace_reexport_after_relative_resolution_to_absolute_target() {
    let request = must_request(json!({
        "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
        "root": "C:/repo",
        "sourceFiles": [
            "src/consumer.ts",
            "src/barrel.ts",
            "src/source.ts"
        ],
        "namespaceReExports": [{
            "barrelFile": "src/barrel.ts",
            "exportedName": "ns",
            "targetFile": "src/source.ts",
            "sourceSpec": "./source"
        }],
        "records": [{
            "recordId": "src/consumer.ts#0",
            "consumerFile": "src/consumer.ts",
            "fromSpec": "./barrel",
            "name": "ns",
            "kind": "imported-namespace-escape",
            "resolverStage": "relative"
        }]
    }));
    let response = response(request);

    assert_eq!(response.summary.handled_count, 1);
    assert_eq!(response.branch_counts["namespaceReExportEscape"], 1);
    assert_eq!(response.resolved_internal_edges[0].to, "src/source.ts");
    assert_eq!(response.namespace_users[0].def_file, "src/source.ts");
    assert_eq!(
        response.namespace_re_export_diagnostics[0].target_file,
        "src/source.ts"
    );
}

#[test]
fn assembles_namespace_reexport_member_and_escape_uses() {
    let request = must_request(json!({
        "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
        "root": "C:/repo",
        "sourceFiles": [
            "C:/repo/src/consumer.ts",
            "C:/repo/src/barrel.ts",
            "C:/repo/src/nested.ts",
            "C:/repo/src/dep.ts"
        ],
        "namespaceReExports": [
            {
                "barrelFile": "C:/repo/src/nested.ts",
                "exportedName": "api",
                "targetFile": "C:/repo/src/dep.ts",
                "sourceSpec": "./dep"
            }
        ],
        "namedReExports": [
            {
                "barrelFile": "C:/repo/src/barrel.ts",
                "exportedName": "api",
                "targetFile": "C:/repo/src/nested.ts",
                "sourceSpec": "./nested"
            }
        ],
        "records": [
            {
                "recordId": "src/consumer.ts#0",
                "consumerFile": "C:/repo/src/consumer.ts",
                "fromSpec": "./barrel",
                "name": "api",
                "memberName": "run",
                "kind": "imported-namespace-member",
                "line": 4
            },
            {
                "recordId": "src/consumer.ts#1",
                "consumerFile": "C:/repo/src/consumer.ts",
                "fromSpec": "./barrel",
                "name": "api",
                "kind": "imported-namespace-escape",
                "line": 5
            },
            {
                "recordId": "src/consumer.ts#2",
                "consumerFile": "C:/repo/src/consumer.ts",
                "fromSpec": "./barrel",
                "name": "missing",
                "memberName": "nope",
                "kind": "imported-namespace-member"
            }
        ]
    }));
    let response = response(request);

    assert_eq!(response.summary.handled_count, 3);
    assert_eq!(response.counters.total_uses, 2);
    assert_eq!(response.counters.resolved_internal_uses, 2);
    assert_eq!(response.branch_counts["namespaceReExport"], 3);
    assert_eq!(response.branch_counts["namespaceReExportMember"], 1);
    assert_eq!(response.branch_counts["namespaceReExportEscape"], 1);
    assert_eq!(response.branch_counts["namespaceReExportMiss"], 1);
    assert_eq!(response.resolved_internal_edges[0].to, "src/dep.ts");
    assert_eq!(
        response.resolved_internal_edges[0].kind,
        "reexport-namespace-member"
    );
    assert_eq!(response.direct_consumers[0].def_file, "src/dep.ts");
    assert_eq!(response.direct_consumers[0].symbol, "run");
    assert_eq!(response.namespace_users[0].def_file, "src/dep.ts");
    assert_eq!(
        response.namespace_re_export_diagnostics[0].reason,
        "namespace-object-escaped"
    );
    assert_eq!(
        response.namespace_re_export_diagnostics[0].chain[0].kind,
        "named-reexport"
    );
    assert_eq!(
        response.namespace_re_export_diagnostics[0].chain[1].kind,
        "namespace-reexport"
    );
}
