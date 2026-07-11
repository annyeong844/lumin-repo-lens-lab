use super::*;

#[test]
fn assembles_external_dependency_consumers() {
    let response = response(request(json!([
        {
            "recordId": "src/a.ts#0",
            "consumerFile": "C:/repo/src/a.ts",
            "fromSpec": "react/jsx-runtime",
            "kind": "import",
            "typeOnly": false,
            "typeOnlyPresent": true,
            "resolverStage": "external",
            "consumerSource": "source-import"
        },
        {
            "recordId": "src/b.ts#0",
            "consumerFile": "C:/repo/src/b.ts",
            "fromSpec": "@scope/pkg/subpath",
            "kind": "import",
            "typeOnly": true,
            "typeOnlyPresent": true,
            "resolverStage": "external",
            "consumerSource": "mdx-import"
        },
        {
            "recordId": "src/c.ts#0",
            "consumerFile": "C:/repo/src/c.ts",
            "fromSpec": "#internal",
            "kind": "import",
            "resolverStage": "external",
            "consumerSource": "source-import"
        },
        {
            "recordId": "src/d.ts#0",
            "consumerFile": "C:/repo/src/d.ts",
            "fromSpec": "pkg",
            "name": "api",
            "kind": "imported-namespace-escape",
            "resolverStage": "external",
            "consumerSource": "source-import"
        }
    ])));

    assert_eq!(response.summary.handled_count, 4);
    assert_eq!(response.counters.external_uses, 3);
    assert_eq!(response.counters.unresolved_uses, 3);
    assert_eq!(response.branch_counts["external"], 4);
    assert_eq!(response.branch_counts["skippedNamespaceAlias"], 1);
    assert_eq!(response.dependency_import_consumers.len(), 2);
    assert_eq!(response.dependency_import_consumers[0].dep_root, "react");
    assert_eq!(
        response.dependency_import_consumers[0].from_spec,
        "react/jsx-runtime"
    );
    assert_eq!(
        response.dependency_import_consumers[0].type_only,
        Some(false)
    );
    assert_eq!(
        response.dependency_import_consumers[1].dep_root,
        "@scope/pkg"
    );
    assert_eq!(response.dependency_import_consumers[1].source, "mdx-import");
    assert_eq!(
        response.dependency_import_consumers[1].type_only,
        Some(true)
    );
}

#[test]
fn assembles_generated_virtual_consumers_and_unresolved_exports() {
    let surface = json!({
        "id": "generated-virtual:prisma-enums:@pkg/db:enums",
        "source": "generated-virtual",
        "mode": "virtual",
        "virtual": true,
        "exports": [{
            "name": "Role",
            "kind": "prisma-enum",
            "spaces": ["value", "type"]
        }]
    });
    let response = response(request(json!([
        {
            "recordId": "src/a.ts#0",
            "consumerFile": "C:/repo/src/a.ts",
            "fromSpec": "@pkg/db/enums",
            "name": "Role",
            "kind": "import",
            "typeOnly": false,
            "typeOnlyPresent": true,
            "resolverStage": "generated-virtual",
            "generatedVirtualSurface": surface
        },
        {
            "recordId": "src/a.ts#1",
            "consumerFile": "C:/repo/src/a.ts",
            "fromSpec": "@pkg/db/enums",
            "name": "Missing",
            "kind": "import",
            "typeOnly": false,
            "typeOnlyPresent": true,
            "resolverStage": "generated-virtual",
            "generatedVirtualSurface": surface,
            "unresolvedEvidence": {
                "reason": "workspace-generated-virtual-export-missing"
            }
        }
    ])));

    assert_eq!(response.summary.handled_count, 2);
    assert_eq!(response.counters.total_uses, 1);
    assert_eq!(response.counters.resolved_internal_uses, 1);
    assert_eq!(response.counters.resolved_generated_virtual_uses, 1);
    assert_eq!(response.counters.unresolved_uses, 1);
    assert_eq!(response.counters.unresolved_internal_uses, 1);
    assert_eq!(response.generated_virtual_surfaces.len(), 1);
    assert_eq!(
        response.generated_virtual_record_ids,
        ["src/a.ts#0", "src/a.ts#1"]
    );
    assert_eq!(
        response.generated_virtual_import_consumers[0]["surfaceId"],
        "generated-virtual:prisma-enums:@pkg/db:enums"
    );
    assert_eq!(
        response.generated_virtual_import_consumers[0]["name"],
        "Role"
    );
    assert_eq!(
        response.unresolved_internal_specifier_records[0]["reason"],
        "workspace-generated-virtual-export-missing"
    );
}

#[test]
fn assembles_unresolved_internal_specifier_records() {
    let response = response(request(json!([
        {
            "recordId": "src/a.ts#0",
            "consumerFile": "C:/repo/src/a.ts",
            "fromSpec": "@/missing",
            "kind": "import",
            "typeOnly": false,
            "typeOnlyPresent": true,
            "resolverStage": "unresolved-internal",
            "unresolvedEvidence": {
                "reason": "tsconfig-path-target-missing",
                "resolverStage": "tsconfig-paths",
                "matchedPattern": "@/*",
                "targetCandidates": ["src/missing.ts"],
                "hint": "check-tsconfig-paths"
            }
        },
        {
            "recordId": "src/b.ts#0",
            "consumerFile": "C:/repo/src/b.ts",
            "fromSpec": "./missing",
            "kind": "import-side-effect",
            "resolverStage": "unresolved-relative",
            "unresolvedEvidence": {
                "reason": "relative-target-missing",
                "resolverStage": "relative"
            }
        },
        {
            "recordId": "src/c.ts#0",
            "consumerFile": "C:/repo/src/c.ts",
            "fromSpec": "@/api",
            "kind": "imported-namespace-escape",
            "resolverStage": "unresolved-internal"
        }
    ])));

    assert_eq!(response.summary.handled_count, 3);
    assert_eq!(response.counters.unresolved_uses, 2);
    assert_eq!(response.counters.unresolved_internal_uses, 2);
    assert_eq!(response.branch_counts["unresolved"], 3);
    assert_eq!(response.branch_counts["skippedNamespaceAlias"], 1);
    assert_eq!(response.unresolved_internal_by_prefix["@/"], 1);
    assert_eq!(response.prefix_examples["@/"], "@/missing");
    assert!(response
        .unresolved_internal_specifiers
        .contains("@/missing"));
    assert!(response
        .unresolved_internal_specifiers
        .contains("./missing"));
    assert_eq!(response.unresolved_internal_specifier_records.len(), 2);
    assert_eq!(
        response.unresolved_internal_specifier_records[0]["reason"],
        "tsconfig-path-target-missing"
    );
    assert_eq!(
        response.unresolved_internal_specifier_records[0]["typeOnly"],
        false
    );
    assert_eq!(
        response.unresolved_internal_specifier_records[1]["reason"],
        "relative-target-missing"
    );
}
