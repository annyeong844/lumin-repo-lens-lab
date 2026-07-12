use super::*;

#[test]
fn import_meta_glob_expansions_are_broad_namespace_consumers() {
    let response = response(request(json!([
        {
            "recordId": "src/routes.ts#0:glob:src/pages/home.ts",
            "consumerFile": "C:/repo/src/routes.ts",
            "resolvedFile": "C:/repo/src/pages/home.ts",
            "fromSpec": "./pages/*.ts",
            "kind": "dynamic-import-meta-glob",
            "resolverStage": "resolved-internal"
        }
    ])));

    assert_eq!(response.summary.handled_count, 1);
    assert_eq!(response.counters.total_uses, 1);
    assert_eq!(response.counters.resolved_internal_uses, 1);
    assert_eq!(response.resolved_internal_edges[0].from, "src/routes.ts");
    assert_eq!(response.resolved_internal_edges[0].to, "src/pages/home.ts");
    assert_eq!(
        response.resolved_internal_edges[0].kind,
        "dynamic-import-meta-glob"
    );
    assert_eq!(response.namespace_users[0].def_file, "src/pages/home.ts");
    assert_eq!(response.namespace_users[0].consumer_file, "src/routes.ts");
}

#[test]
fn expands_import_meta_glob_records_from_scanned_source_files() {
    let response = response(must_request(json!({
        "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
        "root": "C:/repo",
        "importMetaGlobCap": 64,
        "sourceFiles": [
            "C:/repo/src/pages/about.ts",
            "C:/repo/src/pages/home.ts",
            "C:/repo/src/pages/home.test.ts",
            "C:/repo/src/pages/nested/ignored.ts",
            "C:/repo/src/pages/ignored.css"
        ],
        "records": [{
            "recordId": "src/routes.ts#0",
            "consumerFile": "C:/repo/src/routes.ts",
            "fromSpec": "./pages/*.ts",
            "kind": "import-meta-glob",
            "resolverStage": "relative",
            "line": 7
        }]
    })));

    assert_eq!(response.summary.handled_count, 1);
    assert!(response
        .handled_record_ids
        .contains(&"src/routes.ts#0".to_string()));
    assert_eq!(response.counters.total_uses, 3);
    assert_eq!(response.counters.resolved_internal_uses, 3);
    assert_eq!(response.branch_counts["importMetaGlobResolved"], 1);
    assert_eq!(
        response
            .resolved_internal_edges
            .iter()
            .map(|edge| edge.to.as_str())
            .collect::<Vec<_>>(),
        vec![
            "src/pages/about.ts",
            "src/pages/home.test.ts",
            "src/pages/home.ts"
        ]
    );
    assert!(response
        .namespace_users
        .iter()
        .any(|user| user.def_file == "src/pages/home.ts" && user.consumer_file == "src/routes.ts"));
}

#[test]
fn import_meta_glob_namespace_users_are_deduplicated_with_other_broad_uses() {
    let response = response(must_request(json!({
        "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
        "root": "C:/repo",
        "sourceFiles": [
            "C:/repo/src/pages/home.ts"
        ],
        "records": [
            {
                "recordId": "src/routes.ts#0",
                "consumerFile": "C:/repo/src/routes.ts",
                "resolvedFile": "C:/repo/src/pages/home.ts",
                "fromSpec": "./pages/home",
                "name": "*",
                "kind": "namespace",
                "resolverStage": "relative"
            },
            {
                "recordId": "src/routes.ts#1",
                "consumerFile": "C:/repo/src/routes.ts",
                "fromSpec": "./pages/*.ts",
                "kind": "import-meta-glob",
                "resolverStage": "import-meta-glob"
            }
        ]
    })));

    assert_eq!(response.summary.handled_count, 2);
    assert_eq!(response.resolved_internal_edges.len(), 2);
    let matching_namespace_users = response
        .namespace_users
        .iter()
        .filter(|user| {
            user.def_file == "src/pages/home.ts" && user.consumer_file == "src/routes.ts"
        })
        .count();
    assert_eq!(matching_namespace_users, 1);
}

#[test]
fn import_meta_glob_unsupported_records_remain_unresolved_without_prefix_summary() {
    let response = response(must_request(json!({
        "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
        "root": "C:/repo",
        "sourceFiles": ["C:/repo/src/app.ts"],
        "records": [{
            "recordId": "src/app.ts#0",
            "consumerFile": "C:/repo/src/app.ts",
            "fromSpec": "./*.md",
            "kind": "import-meta-glob",
            "resolverStage": "relative"
        }]
    })));

    assert_eq!(response.summary.handled_count, 1);
    assert_eq!(response.counters.unresolved_uses, 1);
    assert_eq!(response.counters.unresolved_internal_uses, 1);
    assert_eq!(response.branch_counts["importMetaGlobUnsupported"], 1);
    assert!(response.unresolved_internal_by_prefix.is_empty());
    assert_eq!(
        response.unresolved_internal_specifier_records[0]["reason"],
        "import-meta-glob-target-extension-unsupported"
    );
    assert_eq!(
        response.unresolved_internal_specifier_records[0]["scanPolicy"],
        "scanned-source-files"
    );
}

#[test]
fn non_relative_import_meta_glob_reports_unsupported_evidence() {
    let response = response(must_request(json!({
        "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
        "root": "C:/repo",
        "sourceFiles": ["C:/repo/src/app.ts"],
        "records": [{
            "recordId": "src/app.ts#0",
            "consumerFile": "C:/repo/src/app.ts",
            "fromSpec": "@/pages/*.ts",
            "kind": "import-meta-glob",
            "resolverStage": "import-meta-glob"
        }]
    })));

    assert_eq!(response.summary.handled_count, 1);
    assert!(response.skipped_records.is_empty());
    assert_eq!(response.counters.unresolved_uses, 1);
    assert_eq!(response.counters.unresolved_internal_uses, 1);
    assert_eq!(response.branch_counts["importMetaGlobUnsupported"], 1);
    assert_eq!(
        response.unresolved_internal_specifier_records[0]["reason"],
        "import-meta-glob-nonrelative-unsupported"
    );
    assert_eq!(
        response.unresolved_internal_specifier_records[0]["resolverStage"],
        "import-meta-glob"
    );
}

#[test]
fn preserves_import_meta_glob_unsupported_evidence_without_prefix_summary() {
    let response = response(request(json!([{
        "recordId": "src/app.ts#0",
        "consumerFile": "C:/repo/src/app.ts",
        "fromSpec": "./*.md",
        "kind": "import-meta-glob",
        "resolverStage": "unresolved-relative",
        "unresolvedEvidence": {
            "reason": "import-meta-glob-target-extension-unsupported",
            "resolverStage": "import-meta-glob",
            "outputLevel": "unsupported",
            "unsupportedFamily": "dynamic-modules",
            "hint": "dynamic-module-surface",
            "scanPolicy": "scanned-source-files"
        }
    }])));

    assert_eq!(response.summary.handled_count, 1);
    assert_eq!(response.counters.unresolved_uses, 1);
    assert_eq!(response.counters.unresolved_internal_uses, 1);
    assert!(response.unresolved_internal_by_prefix.is_empty());
    assert!(response.unresolved_internal_specifiers.contains("./*.md"));
    assert_eq!(response.unresolved_internal_specifier_records.len(), 1);
    assert_eq!(
        response.unresolved_internal_specifier_records[0]["reason"],
        "import-meta-glob-target-extension-unsupported"
    );
    assert_eq!(
        response.unresolved_internal_specifier_records[0]["resolverStage"],
        "import-meta-glob"
    );
}
