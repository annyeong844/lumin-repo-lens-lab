use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::metadata::{package_root, CargoMetadata, CargoPackage};
use crate::path_util::{
    has_path_segment, has_windows_drive_prefix, is_inside_path, normalize_path_for_compare,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SpanClass {
    UserCode,
    Dependency,
    Generated,
    Unknown,
}

impl SpanClass {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            SpanClass::UserCode => "user-code",
            SpanClass::Dependency => "dependency",
            SpanClass::Generated => "generated",
            SpanClass::Unknown => "unknown",
        }
    }
}

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
        span: &Value,
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

    pub(crate) fn classify_span(&self, span: &Value) -> SpanClass {
        for candidate in span_candidates(span) {
            let class = self.classify_file_name(&candidate);
            if class == SpanClass::UserCode {
                return class;
            }
        }
        span.get("file_name")
            .and_then(Value::as_str)
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
            || has_path_segment(&path, "OUT_DIR")
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

fn fallback_user_roots(root: &Path) -> Vec<PathBuf> {
    vec![root.join("src")]
}

fn longest_matching_root_len(path: &Path, roots: &[PathBuf]) -> Option<usize> {
    roots
        .iter()
        .filter(|root| is_inside_path(path, root))
        .map(|root| normalize_path_for_compare(root).len())
        .max()
}

fn span_candidates(span: &Value) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = span.get("expansion");
    while let Some(Value::Object(map)) = current {
        if let Some(callsite) = map.get("span") {
            if let Some(file_name) = callsite.get("file_name").and_then(Value::as_str) {
                out.push(file_name.to_string());
            }
            current = callsite.get("expansion");
        } else {
            break;
        }
    }
    out
}

fn resolve_span_path(root: &Path, file_name: &str) -> Option<PathBuf> {
    if file_name.is_empty() || file_name == "<anon>" {
        return None;
    }
    let normalized = file_name.replace('\\', std::path::MAIN_SEPARATOR_STR);
    let path = PathBuf::from(&normalized);
    if path.is_absolute() || has_windows_drive_prefix(&normalized) {
        Some(path)
    } else {
        Some(root.join(path))
    }
}

#[cfg(test)]
mod tests {
    use super::{OwnershipResolver, SpanClass};
    use anyhow::Result;
    use serde_json::json;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn metadata_unavailable_fallback_keeps_root_src_diagnostic_user_code() -> Result<()> {
        let temp = TempDir::new()?;
        let root = temp.path().join("crate");
        fs::create_dir_all(root.join("src"))?;

        let resolver = OwnershipResolver::new(&root, None, &[]);
        let span = json!({"file_name": "src/lib.rs", "is_primary": true});

        assert_eq!(
            resolver.classify_span_for_package(&span, Some("unknown-package")),
            SpanClass::UserCode
        );
        Ok(())
    }

    #[test]
    fn metadata_unavailable_fallback_does_not_mark_dependency_shaped_path_user_code() -> Result<()>
    {
        let temp = TempDir::new()?;
        let root = temp.path().join("crate");
        fs::create_dir_all(root.join("src"))?;
        fs::create_dir_all(root.join("bad_dep").join("src"))?;

        let resolver = OwnershipResolver::new(&root, None, &[]);
        let span = json!({"file_name": "bad_dep/src/lib.rs", "is_primary": true});

        assert_eq!(
            resolver.classify_span_for_package(&span, Some("unknown-package")),
            SpanClass::Unknown
        );
        Ok(())
    }
}
