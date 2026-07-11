use super::*;

#[test]
fn assembles_direct_and_namespace_relative_uses() {
    let response = response(request(json!([
        {
            "recordId": "src/a.ts#0",
            "consumerFile": "C:/repo/src/a.ts",
            "resolvedFile": "C:/repo/src/b.ts",
            "fromSpec": "./b",
            "name": "thing",
            "kind": "import",
            "typeOnly": false,
            "line": 3,
            "resolverStage": "relative"
        },
        {
            "recordId": "src/c.ts#0",
            "consumerFile": "C:/repo/src/c.ts",
            "resolvedFile": "C:/repo/src/d.ts",
            "fromSpec": "./d",
            "kind": "namespace",
            "resolverStage": "relative"
        }
    ])));

    assert_eq!(response.summary.handled_count, 2);
    assert_eq!(response.counters.total_uses, 2);
    assert_eq!(response.counters.resolved_internal_uses, 2);
    assert_eq!(response.branch_counts["resolvedInternal"], 2);
    assert_eq!(response.branch_counts["directConsumer"], 1);
    assert_eq!(response.branch_counts["broadNamespace"], 1);
    assert_eq!(response.resolved_internal_edges[0].from, "src/a.ts");
    assert_eq!(response.resolved_internal_edges[0].to, "src/b.ts");
    assert_eq!(response.resolved_internal_edges[0].kind, "import-named");
    assert_eq!(response.direct_consumers[0].symbol, "thing");
    assert_eq!(response.namespace_users[0].def_file, "src/d.ts");
    assert_eq!(response.resolved_record_targets.len(), 2);
    assert_eq!(response.resolved_record_targets[0].record_id, "src/a.ts#0");
    assert_eq!(
        response.resolved_record_targets[0].resolved_file,
        "C:/repo/src/b.ts"
    );
}

#[test]
fn assembles_pre_resolved_root_relative_record_paths() {
    let response = response(request(json!([
        {
            "recordId": "src/a.ts#0",
            "consumerFile": "src/a.ts",
            "resolvedFile": "src/b.ts",
            "fromSpec": "./b",
            "name": "thing",
            "kind": "import",
            "resolverStage": "resolved-internal"
        }
    ])));

    assert_eq!(response.summary.handled_count, 1);
    assert_eq!(response.resolved_internal_edges[0].from, "src/a.ts");
    assert_eq!(response.resolved_internal_edges[0].to, "src/b.ts");
    assert_eq!(
        response.resolved_record_targets[0].resolved_file,
        "src/b.ts"
    );
    assert_eq!(response.direct_consumers[0].consumer_file, "src/a.ts");
}

#[test]
fn counts_out_of_band_consumer_sources_in_rust_projection() {
    let response = response(request(json!([
        {
            "recordId": "mdx:0:docs/page.mdx:./widget",
            "consumerFile": "C:/repo/docs/page.mdx",
            "resolvedFile": "C:/repo/src/widget.ts",
            "fromSpec": "./widget",
            "name": "Widget",
            "kind": "import",
            "resolverStage": "relative",
            "consumerSource": "mdx-import"
        },
        {
            "recordId": "sfc:0:components/App.vue:./panel",
            "consumerFile": "C:/repo/components/App.vue",
            "resolvedFile": "C:/repo/components/panel.ts",
            "fromSpec": "./panel",
            "name": "Panel",
            "kind": "import",
            "resolverStage": "relative",
            "consumerSource": "sfc-script-import"
        },
        {
            "recordId": "sfc-script-src:0:components/App.vue:./setup",
            "consumerFile": "C:/repo/components/App.vue",
            "resolvedFile": "C:/repo/components/setup.ts",
            "fromSpec": "./setup",
            "kind": "sfc-script-src",
            "resolverStage": "relative",
            "consumerSource": "sfc-script-src"
        }
    ])));

    assert_eq!(response.summary.handled_count, 3);
    assert_eq!(response.counters.mdx_consumer_uses, 1);
    assert_eq!(response.counters.sfc_script_consumer_uses, 1);
    assert_eq!(response.counters.sfc_script_src_reachability_uses, 1);
    assert_eq!(response.branch_counts["sfcScriptSrcReachability"], 1);
}

#[test]
fn assembles_pre_resolved_non_relative_internal_uses() {
    let response = response(request(json!([
        {
            "recordId": "src/a.ts#0",
            "consumerFile": "C:/repo/src/a.ts",
            "resolvedFile": "C:/repo/src/b.ts",
            "fromSpec": "@/b",
            "name": "thing",
            "kind": "import",
            "typeOnly": false,
            "resolverStage": "resolved-internal"
        },
        {
            "recordId": "src/c.ts#0",
            "consumerFile": "C:/repo/src/c.ts",
            "resolvedFile": "C:/repo/src/d.ts",
            "fromSpec": "@pkg/d",
            "kind": "namespace",
            "resolverStage": "resolved-internal"
        }
    ])));

    assert_eq!(response.summary.handled_count, 2);
    assert_eq!(response.counters.total_uses, 2);
    assert_eq!(response.counters.resolved_internal_uses, 2);
    assert_eq!(response.counters.rust_resolved_relative_uses, 0);
    assert_eq!(
        response.resolved_internal_edges[0].source.as_deref(),
        Some("@/b")
    );
    assert_eq!(response.resolved_internal_edges[0].to, "src/b.ts");
    assert_eq!(response.direct_consumers[0].symbol, "thing");
    assert_eq!(response.namespace_users[0].def_file, "src/d.ts");
}

#[test]
fn side_effect_uses_keep_edges_without_consumers() {
    let response = response(request(json!([
        {
            "recordId": "a",
            "consumerFile": "C:/repo/src/a.ts",
            "resolvedFile": "C:/repo/src/setup.ts",
            "fromSpec": "./setup",
            "kind": "import-side-effect",
            "resolverStage": "relative"
        }
    ])));

    assert_eq!(response.summary.handled_count, 1);
    assert_eq!(response.branch_counts["sideEffectOnly"], 1);
    assert_eq!(
        response.resolved_internal_edges[0].kind,
        "import-side-effect"
    );
    assert!(response.direct_consumers.is_empty());
    assert!(response.namespace_users.is_empty());
}

#[test]
fn sfc_script_src_uses_keep_reachability_edges_without_consumers() {
    let response = response(request(json!([
        {
            "recordId": "sfc-script-src:0:components/App.vue:../src/setup.ts",
            "consumerFile": "C:/repo/components/App.vue",
            "resolvedFile": "C:/repo/src/setup.ts",
            "fromSpec": "../src/setup.ts",
            "name": "*",
            "kind": "sfc-script-src",
            "sfcLanguage": "vue",
            "line": 2
        }
    ])));

    assert_eq!(response.summary.handled_count, 1);
    assert_eq!(response.branch_counts["sfcScriptSrcReachability"], 1);
    assert_eq!(
        response.resolved_internal_edges[0].from,
        "components/App.vue"
    );
    assert_eq!(response.resolved_internal_edges[0].to, "src/setup.ts");
    assert_eq!(response.resolved_internal_edges[0].kind, "sfc-script-src");
    assert_eq!(
        response.resolved_internal_edges[0].source.as_deref(),
        Some("../src/setup.ts")
    );
    assert!(response.direct_consumers.is_empty());
    assert!(response.namespace_users.is_empty());
}

#[test]
fn records_non_source_asset_uses_without_reachability_edges() {
    let request = must_request(json!({
        "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
        "root": "C:/repo",
        "records": [
            {
                "recordId": "sfc-script-src:0:components/App.vue:./style.css",
                "consumerFile": "C:/repo/components/App.vue",
                "fromSpec": "./style.css",
                "name": "*",
                "kind": "sfc-script-src",
                "resolverStage": "non-source-asset"
            }
        ]
    }));
    let response = response(request);

    assert_eq!(response.summary.handled_count, 1);
    assert_eq!(response.summary.skipped_count, 0);
    assert_eq!(response.counters.non_source_asset_uses, 1);
    assert_eq!(response.branch_counts["asset"], 1);
    assert!(response.resolved_internal_edges.is_empty());
    assert!(response.direct_consumers.is_empty());
    assert!(response.namespace_users.is_empty());
}
