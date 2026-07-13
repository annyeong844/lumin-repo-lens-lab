use super::{
    is_type_only, line_for_span, module_export_identifier_name, ReExportRecord, UseRecord,
};
use crate::relative_source_resolver::RelativeSourceResolver;
use oxc_ast::ast::{ExportAllDeclaration, ImportDeclarationSpecifier, Statement};

pub(super) fn collect_re_exports(
    statement: &Statement<'_>,
    re_exports: &mut Vec<ReExportRecord>,
    uses: &mut Vec<UseRecord>,
    line_starts: &[usize],
) {
    match statement {
        Statement::ExportNamedDeclaration(export) => {
            let Some(source) = export.source.as_ref() else {
                return;
            };
            re_exports.push(ReExportRecord {
                source: source.value.to_string(),
                line: line_for_span(line_starts, export.span),
                namespace: None,
            });
            for specifier in &export.specifiers {
                let Some(name) = module_export_identifier_name(&specifier.local)
                    .or_else(|| module_export_identifier_name(&specifier.exported))
                else {
                    continue;
                };
                uses.push(UseRecord {
                    from_spec: source.value.to_string(),
                    name,
                    member_name: None,
                    kind: "reExport".to_string(),
                    type_only: is_type_only(export.export_kind)
                        || is_type_only(specifier.export_kind),
                    line: line_for_span(line_starts, specifier.span),
                    local_name: None,
                    degraded: false,
                    resolved_file: None,
                    resolver_stage: None,
                });
            }
        }
        Statement::ExportAllDeclaration(export) => {
            collect_export_all(export, re_exports, uses, line_starts);
        }
        _ => {}
    }
}

fn collect_export_all(
    export: &ExportAllDeclaration<'_>,
    re_exports: &mut Vec<ReExportRecord>,
    uses: &mut Vec<UseRecord>,
    line_starts: &[usize],
) {
    let namespace = export
        .exported
        .as_ref()
        .and_then(module_export_identifier_name);
    re_exports.push(ReExportRecord {
        source: export.source.value.to_string(),
        line: line_for_span(line_starts, export.span),
        namespace: namespace.clone(),
    });
    uses.push(UseRecord {
        from_spec: export.source.value.to_string(),
        name: namespace.unwrap_or_else(|| "*".to_string()),
        member_name: None,
        kind: if export.exported.is_some() {
            "reExportNamespace"
        } else {
            "reExportAll"
        }
        .to_string(),
        type_only: is_type_only(export.export_kind),
        line: line_for_span(line_starts, export.span),
        local_name: None,
        degraded: false,
        resolved_file: None,
        resolver_stage: None,
    });
}

pub(super) fn collect_imports(
    statement: &Statement<'_>,
    uses: &mut Vec<UseRecord>,
    line_starts: &[usize],
) {
    let Statement::ImportDeclaration(import) = statement else {
        return;
    };
    let specifiers = import
        .specifiers
        .as_ref()
        .map_or(&[][..], |items| items.as_slice());
    if specifiers.is_empty() {
        uses.push(UseRecord {
            from_spec: import.source.value.to_string(),
            name: "*".to_string(),
            member_name: None,
            kind: "import-side-effect".to_string(),
            type_only: false,
            line: line_for_span(line_starts, import.span),
            local_name: None,
            degraded: false,
            resolved_file: None,
            resolver_stage: None,
        });
        return;
    }

    for specifier in specifiers {
        match specifier {
            ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                let imported_name = module_export_identifier_name(&specifier.imported)
                    .unwrap_or_else(|| specifier.local.name.to_string());
                let local_name = specifier.local.name.to_string();
                uses.push(UseRecord {
                    from_spec: import.source.value.to_string(),
                    name: imported_name.clone(),
                    member_name: None,
                    kind: "import".to_string(),
                    type_only: is_type_only(import.import_kind)
                        || is_type_only(specifier.import_kind),
                    line: line_for_span(line_starts, specifier.span),
                    local_name: (local_name != imported_name).then_some(local_name),
                    degraded: false,
                    resolved_file: None,
                    resolver_stage: None,
                });
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                uses.push(UseRecord {
                    from_spec: import.source.value.to_string(),
                    name: "default".to_string(),
                    member_name: None,
                    kind: "default".to_string(),
                    type_only: is_type_only(import.import_kind),
                    line: line_for_span(line_starts, specifier.span),
                    local_name: None,
                    degraded: false,
                    resolved_file: None,
                    resolver_stage: None,
                });
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                uses.push(UseRecord {
                    from_spec: import.source.value.to_string(),
                    name: "*".to_string(),
                    member_name: None,
                    kind: "namespace".to_string(),
                    type_only: is_type_only(import.import_kind),
                    line: line_for_span(line_starts, specifier.span),
                    local_name: Some(specifier.local.name.to_string()),
                    degraded: false,
                    resolved_file: None,
                    resolver_stage: None,
                });
            }
        }
    }
}

pub(super) fn annotate_relative_resolutions(
    from_file: &str,
    uses: &mut [UseRecord],
    resolver: &RelativeSourceResolver,
) {
    for use_record in uses {
        if let Some(resolved) = resolver.resolve(from_file, &use_record.from_spec) {
            use_record.resolved_file = Some(resolved);
            use_record.resolver_stage = Some("relative".to_string());
        }
    }
}
