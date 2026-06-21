use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use lumin_rust_common::path_has_segment;

use crate::metadata::{package_root, CargoMetadata, CargoPackage};
use crate::path_util::is_inside_path;
use crate::rustc_span::RustcSpan;

use super::paths::{fallback_user_roots, longest_matching_root_len, resolve_span_path};
use super::SpanClass;

pub(crate) struct OwnershipResolver {
    root: PathBuf,
    target_directory: Option<PathBuf>,
    selected_ids: BTreeSet<String>,
    dependency_ids: BTreeSet<String>,
    selected_roots: Vec<PathBuf>,
    dependency_roots: Vec<PathBuf>,
}

impl OwnershipResolver {
    pub(crate) fn new(
        root: &Path,
        metadata: Option<&CargoMetadata>,
        selected: &[CargoPackage],
    ) -> Self {
        let selected_ids = selected
            .iter()
            .map(|pkg| pkg.id.clone())
            .collect::<BTreeSet<_>>();
        let selected_roots = if selected.is_empty() && metadata.is_none() {
            fallback_user_roots(root)
        } else {
            selected.iter().map(package_root).collect()
        };
        let dependency_ids = metadata
            .map(|metadata| {
                metadata
                    .packages
                    .iter()
                    .filter(|pkg| !selected_ids.contains(&pkg.id))
                    .map(|pkg| pkg.id.clone())
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default();
        let dependency_roots = metadata
            .map(|metadata| {
                metadata
                    .packages
                    .iter()
                    .filter(|pkg| !selected_ids.contains(&pkg.id))
                    .filter(|pkg| pkg.source.is_none())
                    .map(package_root)
                    .collect()
            })
            .unwrap_or_default();
        let target_directory = metadata
            .and_then(|metadata| metadata.target_directory.as_deref())
            .map(PathBuf::from);
        Self {
            root: root.to_path_buf(),
            target_directory,
            selected_ids,
            dependency_ids,
            selected_roots,
            dependency_roots,
        }
    }

    pub(crate) fn classify_span_for_package(
        &self,
        span: &RustcSpan,
        package_id: Option<&str>,
    ) -> SpanClass {
        let path_class = self.classify_span(span);
        match package_id {
            Some(package_id)
                if !self.selected_ids.is_empty() && self.dependency_ids.contains(package_id) =>
            {
                if path_class == SpanClass::Generated {
                    SpanClass::Generated
                } else {
                    SpanClass::Dependency
                }
            }
            Some(package_id)
                if !self.selected_ids.is_empty() && !self.selected_ids.contains(package_id) =>
            {
                if path_class == SpanClass::Generated {
                    SpanClass::Generated
                } else {
                    SpanClass::Dependency
                }
            }
            _ => path_class,
        }
    }

    pub(crate) fn classify_span(&self, span: &RustcSpan) -> SpanClass {
        for candidate in span.expansion_callsite_file_names() {
            let class = self.classify_file_name(&candidate);
            if class == SpanClass::UserCode {
                return class;
            }
        }
        span.file_name()
            .map(|file_name| self.classify_file_name(file_name))
            .unwrap_or(SpanClass::Unknown)
    }

    fn classify_file_name(&self, file_name: &str) -> SpanClass {
        let Some(path) = resolve_span_path(&self.root, file_name) else {
            return SpanClass::Unknown;
        };

        if self
            .target_directory
            .as_ref()
            .is_some_and(|target| is_inside_path(&path, target))
            || path_has_segment(&path, "OUT_DIR")
        {
            return SpanClass::Generated;
        }

        let selected_match = longest_matching_root_len(&path, &self.selected_roots);
        let dependency_match = longest_matching_root_len(&path, &self.dependency_roots);
        match (selected_match, dependency_match) {
            (Some(selected), Some(dependency)) if dependency > selected => SpanClass::Dependency,
            (Some(_), _) => SpanClass::UserCode,
            (None, Some(_)) => SpanClass::Dependency,
            (None, None) => SpanClass::Unknown,
        }
    }
}
