use std::collections::{BTreeMap, BTreeSet};

use crate::relative_source_resolver::normalize_path_text;

use super::path::root_relative;
use super::protocol::{NamespaceReExportChainEntry, SourceUseAssemblyReExport};

#[derive(Clone, Debug)]
struct ReExportTarget {
    target_file: String,
    source_spec: Option<String>,
}

#[derive(Debug)]
pub(super) struct ResolvedNamespaceReExport {
    pub(super) target_file: String,
    pub(super) chain: Vec<NamespaceReExportChainEntry>,
}

#[derive(Debug)]
pub(super) struct NamespaceReExportResolver {
    namespace: BTreeMap<(String, String), ReExportTarget>,
    named: BTreeMap<(String, String), ReExportTarget>,
}

impl NamespaceReExportResolver {
    pub(super) fn new(
        namespace_re_exports: Vec<SourceUseAssemblyReExport>,
        named_re_exports: Vec<SourceUseAssemblyReExport>,
    ) -> Self {
        Self {
            namespace: re_export_map(namespace_re_exports),
            named: re_export_map(named_re_exports),
        }
    }

    pub(super) fn resolve(
        &self,
        root: &str,
        barrel_file: &str,
        exported_name: &str,
    ) -> Option<ResolvedNamespaceReExport> {
        let mut seen = BTreeSet::new();
        self.resolve_inner(root, barrel_file, exported_name, &mut seen)
    }

    fn resolve_inner(
        &self,
        root: &str,
        barrel_file: &str,
        exported_name: &str,
        seen: &mut BTreeSet<(String, String)>,
    ) -> Option<ResolvedNamespaceReExport> {
        let normalized_barrel = normalize_path_text(barrel_file);
        let exported = exported_name.to_string();
        if !seen.insert((normalized_barrel.clone(), exported.clone())) {
            return None;
        }

        if let Some(direct) = lookup_re_export(&self.namespace, root, &normalized_barrel, &exported)
        {
            return Some(ResolvedNamespaceReExport {
                target_file: direct.target_file.clone(),
                chain: vec![NamespaceReExportChainEntry {
                    kind: "namespace-reexport",
                    file: root_relative(root, &normalized_barrel),
                    exported_name: exported,
                    target_file: root_relative(root, &direct.target_file),
                    source: direct.source_spec.clone(),
                }],
            });
        }

        let named = lookup_re_export(&self.named, root, &normalized_barrel, &exported)?;
        let nested = self.resolve_inner(root, &named.target_file, exported_name, seen)?;
        let mut chain = vec![NamespaceReExportChainEntry {
            kind: "named-reexport",
            file: root_relative(root, &normalized_barrel),
            exported_name: exported,
            target_file: root_relative(root, &named.target_file),
            source: named.source_spec.clone(),
        }];
        chain.extend(nested.chain);
        Some(ResolvedNamespaceReExport {
            target_file: nested.target_file,
            chain,
        })
    }
}

fn lookup_re_export<'a>(
    map: &'a BTreeMap<(String, String), ReExportTarget>,
    root: &str,
    barrel_file: &str,
    exported_name: &str,
) -> Option<&'a ReExportTarget> {
    let normalized = normalize_path_text(barrel_file);
    let exported = exported_name.to_string();
    map.get(&(normalized.clone(), exported.clone()))
        .or_else(|| {
            let relative = root_relative(root, &normalized);
            if relative == normalized {
                None
            } else {
                map.get(&(relative, exported))
            }
        })
}

fn re_export_map(
    re_exports: Vec<SourceUseAssemblyReExport>,
) -> BTreeMap<(String, String), ReExportTarget> {
    let mut out = BTreeMap::new();
    for re_export in re_exports {
        if re_export.exported_name.is_empty() {
            continue;
        }
        out.insert(
            (
                normalize_path_text(&re_export.barrel_file),
                re_export.exported_name,
            ),
            ReExportTarget {
                target_file: normalize_path_text(&re_export.target_file),
                source_spec: re_export.source_spec,
            },
        );
    }
    out
}
