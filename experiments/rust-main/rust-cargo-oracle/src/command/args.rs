pub(crate) fn cargo_check_args(features: Option<&str>, package_name: Option<&str>) -> Vec<String> {
    let package_names = package_name
        .map(|package_name| vec![package_name.to_string()])
        .unwrap_or_default();
    cargo_check_args_for_packages(features, &package_names)
}

pub(crate) fn cargo_check_args_for_packages(
    features: Option<&str>,
    package_names: &[String],
) -> Vec<String> {
    let mut args = vec!["check".to_string(), "--message-format=json".to_string()];
    if package_names.len() > 1 {
        args.push("--keep-going".to_string());
    }
    for package_name in package_names {
        args.push("--package".to_string());
        args.push(package_name.to_string());
    }
    if let Some(features) = features {
        args.push("--features".to_string());
        args.push(features.to_string());
    }
    args
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum MetadataDependencyMode {
    IncludeDependencies,
    WorkspaceOnly,
}

pub(super) fn cargo_metadata_args(
    features: Option<&str>,
    dependency_mode: MetadataDependencyMode,
) -> Vec<String> {
    let mut args = vec!["metadata".to_string(), "--format-version=1".to_string()];
    if dependency_mode == MetadataDependencyMode::WorkspaceOnly {
        args.push("--no-deps".to_string());
    }
    if let Some(features) = features {
        args.push("--features".to_string());
        args.push(features.to_string());
    }
    args
}

#[cfg(test)]
mod tests {
    use super::{cargo_check_args_for_packages, cargo_metadata_args, MetadataDependencyMode};

    #[test]
    fn cargo_metadata_args_follow_check_feature_selection() {
        assert_eq!(
            cargo_metadata_args(
                Some("bad,extra"),
                MetadataDependencyMode::IncludeDependencies
            ),
            vec![
                "metadata".to_string(),
                "--format-version=1".to_string(),
                "--features".to_string(),
                "bad,extra".to_string(),
            ]
        );
    }

    #[test]
    fn cargo_metadata_args_omit_features_when_unselected() {
        assert_eq!(
            cargo_metadata_args(None, MetadataDependencyMode::IncludeDependencies),
            vec!["metadata".to_string(), "--format-version=1".to_string()]
        );
    }

    #[test]
    fn cargo_metadata_args_can_skip_dependency_graph_for_metadata_only_mode() {
        assert_eq!(
            cargo_metadata_args(None, MetadataDependencyMode::WorkspaceOnly),
            vec![
                "metadata".to_string(),
                "--format-version=1".to_string(),
                "--no-deps".to_string(),
            ]
        );
    }

    #[test]
    fn cargo_check_args_allow_multiple_selected_packages() {
        assert_eq!(
            cargo_check_args_for_packages(Some("fast"), &["app".to_string(), "util".to_string()]),
            vec![
                "check".to_string(),
                "--message-format=json".to_string(),
                "--keep-going".to_string(),
                "--package".to_string(),
                "app".to_string(),
                "--package".to_string(),
                "util".to_string(),
                "--features".to_string(),
                "fast".to_string(),
            ]
        );
    }
}
