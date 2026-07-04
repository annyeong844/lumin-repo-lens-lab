use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

pub const FRAMEWORK_RESOURCE_SURFACE_SCHEMA_VERSION: &str = "framework-resource-surfaces.v1";
pub const FRAMEWORK_RESOURCE_SURFACE_POLICY_VERSION: &str = "framework-resource-surface-policy-v1";
pub const FRAMEWORK_RESOURCE_SURFACE_REQUEST_SCHEMA_VERSION: &str =
    "lumin-framework-resource-surfaces-producer-request.v1";

const DEPENDENCY_SECTIONS: &[&str] = &[
    "dependencies",
    "devDependencies",
    "peerDependencies",
    "optionalDependencies",
];

const JS_LIKE_EXTENSIONS: &[&str] = &[
    "js", "jsx", "ts", "tsx", "mjs", "mjsx", "mts", "mtsx", "cjs", "cjsx", "cts", "ctsx",
];

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameworkResourceSurfacesRequest {
    pub schema_version: String,
    pub root: String,
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub package_records: Vec<PackageRecord>,
    #[serde(default)]
    pub contents_by_file: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageRecord {
    #[serde(default)]
    pub root: String,
    #[serde(default = "default_package_rel_root")]
    pub rel_root: String,
    #[serde(default)]
    pub package_json: Value,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameworkResourceSurfacesArtifact {
    pub schema_version: &'static str,
    pub policy_version: &'static str,
    pub root: String,
    pub files: Vec<FrameworkResourceFile>,
    pub summary: FrameworkResourceSummary,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameworkResourceFile {
    pub file: String,
    pub package_root: String,
    pub surface_lanes: Vec<SurfaceLane>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SurfaceLane {
    pub lane: &'static str,
    pub capability_pack: &'static str,
    pub confidence: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub framework: Option<&'static str>,
    pub reason: &'static str,
    pub default_action: &'static str,
    pub affects_absence_claims: bool,
    pub evidence: Vec<LaneEvidence>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaneEvidence {
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameworkResourceSummary {
    pub total_files_with_surfaces: usize,
    pub total_surface_lanes: usize,
    pub by_lane: BTreeMap<String, usize>,
    pub by_capability_pack: BTreeMap<String, usize>,
    pub by_confidence: BTreeMap<String, usize>,
    pub by_reason: BTreeMap<String, usize>,
    pub by_framework: BTreeMap<String, usize>,
    pub top_examples: Vec<FrameworkResourceTopExample>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameworkResourceTopExample {
    pub file: String,
    pub lanes: Vec<&'static str>,
    pub capability_packs: Vec<&'static str>,
    pub reasons: Vec<&'static str>,
}

#[derive(Debug, Clone)]
struct NormalizedPackageRecord {
    rel_root: String,
    package_json: Value,
}

pub fn build_framework_resource_surfaces_artifact(
    request: FrameworkResourceSurfacesRequest,
) -> Result<FrameworkResourceSurfacesArtifact> {
    if request.schema_version != FRAMEWORK_RESOURCE_SURFACE_REQUEST_SCHEMA_VERSION {
        bail!(
            "framework-resource-surfaces-artifact: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }

    let packages = sort_package_records(request.package_records)?;
    let contents_by_file = normalize_contents_by_file(&request.contents_by_file);
    let mut files = Vec::new();

    for raw_file in &request.files {
        let file = normalize_request_file(raw_file)?;
        let package = nearest_package(&packages, &file)
            .context("framework-resource-surfaces-artifact: no package records available")?;
        let rel_file = package_relative(&file, package);
        let content = contents_by_file
            .get(&file)
            .map(String::as_str)
            .unwrap_or("");
        let mut surface_lanes = [
            storybook_lane(package, &rel_file),
            strapi_lane(package, &rel_file),
            generated_declaration_lane(&rel_file),
            bundled_build_lane(&rel_file, content),
            scaffold_template_lane(&rel_file),
            codemod_resource_lane(&rel_file),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        if surface_lanes.is_empty() {
            continue;
        }

        surface_lanes.sort_by(|left, right| {
            left.lane
                .cmp(right.lane)
                .then_with(|| left.reason.cmp(right.reason))
        });
        files.push(FrameworkResourceFile {
            file,
            package_root: package.rel_root.clone(),
            surface_lanes,
        });
    }

    files.sort_by(|left, right| left.file.cmp(&right.file));
    let summary = build_summary(&files);
    Ok(FrameworkResourceSurfacesArtifact {
        schema_version: FRAMEWORK_RESOURCE_SURFACE_SCHEMA_VERSION,
        policy_version: FRAMEWORK_RESOURCE_SURFACE_POLICY_VERSION,
        root: request.root,
        files,
        summary,
    })
}

fn default_package_rel_root() -> String {
    ".".to_string()
}

fn normalize_contents_by_file(contents: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for (file, content) in contents {
        if let Ok(normalized) = normalize_request_file(file) {
            out.insert(normalized, content.clone());
        }
    }
    out
}

fn sort_package_records(records: Vec<PackageRecord>) -> Result<Vec<NormalizedPackageRecord>> {
    let records = if records.is_empty() {
        vec![PackageRecord {
            root: String::new(),
            rel_root: ".".to_string(),
            package_json: Value::Object(Default::default()),
        }]
    } else {
        records
    };

    let mut packages = records
        .into_iter()
        .map(|record| {
            let _root = record.root;
            Ok(NormalizedPackageRecord {
                rel_root: normalize_package_root(&record.rel_root)?,
                package_json: record.package_json,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    packages.sort_by(|left, right| {
        package_depth(&right.rel_root)
            .cmp(&package_depth(&left.rel_root))
            .then_with(|| left.rel_root.cmp(&right.rel_root))
    });
    Ok(packages)
}

fn normalize_request_file(value: &str) -> Result<String> {
    normalize_relative_path(value, false)
}

fn normalize_package_root(value: &str) -> Result<String> {
    normalize_relative_path(value, true)
}

fn normalize_relative_path(value: &str, allow_empty_root: bool) -> Result<String> {
    let normalized = value.replace('\\', "/");
    if normalized.is_empty() {
        if allow_empty_root {
            return Ok(".".to_string());
        }
        bail!("invalid normalized path: empty path");
    }
    if normalized.starts_with('/') || looks_like_windows_absolute_path(&normalized) {
        bail!("invalid normalized path '{value}': absolute paths are not accepted");
    }

    let mut parts = Vec::new();
    for part in normalized.split('/') {
        match part {
            "" | "." => {}
            ".." => bail!("invalid normalized path '{value}': traversal is not accepted"),
            segment => parts.push(segment),
        }
    }

    if parts.is_empty() {
        if allow_empty_root || normalized == "." {
            return Ok(".".to_string());
        }
        bail!("invalid normalized path '{value}': empty path");
    }
    Ok(parts.join("/"))
}

fn looks_like_windows_absolute_path(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && matches!(bytes[2], b'/' | b'\\')
}

fn package_depth(rel_root: &str) -> usize {
    if rel_root == "." {
        0
    } else {
        rel_root.split('/').count()
    }
}

fn nearest_package<'a>(
    packages: &'a [NormalizedPackageRecord],
    file: &str,
) -> Option<&'a NormalizedPackageRecord> {
    packages
        .iter()
        .find(|package| {
            package.rel_root == "."
                || file == package.rel_root
                || file.starts_with(&format!("{}/", package.rel_root))
        })
        .or_else(|| packages.last())
}

fn package_relative(file: &str, package: &NormalizedPackageRecord) -> String {
    if package.rel_root == "." {
        return file.to_string();
    }
    if file == package.rel_root {
        return ".".to_string();
    }
    file.strip_prefix(&format!("{}/", package.rel_root))
        .unwrap_or(file)
        .to_string()
}

fn storybook_lane(package: &NormalizedPackageRecord, rel_file: &str) -> Option<SurfaceLane> {
    if !is_storybook_story_file(rel_file) {
        return None;
    }
    let deps = storybook_dependency_evidence(package);
    Some(surface_lane(
        "framework-dispatch-entry",
        "framework.storybook",
        if deps.is_empty() {
            "path-shaped-review"
        } else {
            "grounded"
        },
        Some("storybook"),
        "storybook-story-file",
        deps.into_iter()
            .chain([path_evidence("*.stories.*")])
            .collect(),
    ))
}

fn strapi_lane(package: &NormalizedPackageRecord, rel_file: &str) -> Option<SurfaceLane> {
    if !is_strapi_api_file(rel_file) {
        return None;
    }
    let deps = strapi_dependency_evidence(package);
    Some(surface_lane(
        "framework-dispatch-entry",
        "framework.strapi",
        if deps.is_empty() {
            "path-shaped-review"
        } else {
            "grounded"
        },
        Some("strapi"),
        "strapi-filesystem-api",
        deps.into_iter()
            .chain([path_evidence("src/api/*/{controllers,routes,services}/**")])
            .collect(),
    ))
}

fn generated_declaration_lane(rel_file: &str) -> Option<SurfaceLane> {
    if !rel_file.ends_with(".d.ts") || !has_path_segment(rel_file, "generated") {
        return None;
    }
    Some(surface_lane(
        "generated-declaration-surface",
        "surface.generated-declaration",
        "generated-output-review",
        None,
        "generated-declaration-path",
        vec![path_evidence("**/generated/**/*.d.ts")],
    ))
}

fn bundled_build_lane(rel_file: &str, content: &str) -> Option<SurfaceLane> {
    let base = file_name(rel_file);
    let emscripten =
        content.contains("@ts-nocheck") && content.to_ascii_lowercase().contains("emscripten");
    let reason = if emscripten {
        "emscripten-generated-header"
    } else if base == "vendor.js" {
        "vendor-bundle-name"
    } else if has_js_like_marker(base, ".bundle.") {
        "bundle-file-name"
    } else if has_js_like_marker(base, ".min.") {
        "minified-file-name"
    } else {
        return None;
    };
    Some(surface_lane(
        "bundled-build-artifact",
        "surface.bundled-build-artifact",
        "generated-output-review",
        None,
        reason,
        vec![if emscripten {
            LaneEvidence {
                kind: "file-header",
                field: None,
                matched: Some("@ts-nocheck + Emscripten".to_string()),
            }
        } else {
            path_evidence(base)
        }],
    ))
}

fn scaffold_template_lane(rel_file: &str) -> Option<SurfaceLane> {
    let is_hbs = rel_file.ends_with(".hbs");
    if !is_hbs && !has_path_segment(rel_file, "templates") {
        return None;
    }
    Some(surface_lane(
        "scaffold-template-resource",
        "surface.scaffold-template",
        "resource-only",
        None,
        if is_hbs {
            "handlebars-template-resource"
        } else {
            "templates-directory-resource"
        },
        vec![path_evidence(if is_hbs { "*.hbs" } else { "templates/**" })],
    ))
}

fn codemod_resource_lane(rel_file: &str) -> Option<SurfaceLane> {
    let matched = if contains_path_sequence(rel_file, &["resources", "codemods"]) {
        "resources/codemods/**"
    } else if has_path_segment(rel_file, "codemods") {
        "codemods/**"
    } else if has_path_segment(rel_file, "__testfixtures__") {
        "__testfixtures__/**"
    } else {
        return None;
    };
    Some(surface_lane(
        "codemod-resource",
        "surface.codemod-resource",
        "resource-only",
        None,
        if matched == "__testfixtures__/**" {
            "testfixture-resource"
        } else {
            "codemod-resource-path"
        },
        vec![path_evidence(matched)],
    ))
}

fn surface_lane(
    lane: &'static str,
    capability_pack: &'static str,
    confidence: &'static str,
    framework: Option<&'static str>,
    reason: &'static str,
    mut evidence: Vec<LaneEvidence>,
) -> SurfaceLane {
    evidence.sort_by(|left, right| {
        left.kind
            .cmp(right.kind)
            .then_with(|| {
                left.field
                    .as_deref()
                    .unwrap_or("")
                    .cmp(right.field.as_deref().unwrap_or(""))
            })
            .then_with(|| {
                left.matched
                    .as_deref()
                    .unwrap_or("")
                    .cmp(right.matched.as_deref().unwrap_or(""))
            })
    });
    SurfaceLane {
        lane,
        capability_pack,
        confidence,
        framework,
        reason,
        default_action: "review-hint",
        affects_absence_claims: true,
        evidence,
    }
}

fn storybook_dependency_evidence(package: &NormalizedPackageRecord) -> Vec<LaneEvidence> {
    dependency_evidence(package, |name| {
        name == "storybook" || name.starts_with("@storybook/")
    })
}

fn strapi_dependency_evidence(package: &NormalizedPackageRecord) -> Vec<LaneEvidence> {
    dependency_evidence(package, |name| name == "strapi" || name == "@strapi/strapi")
}

fn dependency_evidence(
    package: &NormalizedPackageRecord,
    predicate: impl Fn(&str) -> bool,
) -> Vec<LaneEvidence> {
    let mut out = Vec::new();
    for section in DEPENDENCY_SECTIONS {
        let Some(table) = package
            .package_json
            .get(*section)
            .and_then(Value::as_object)
        else {
            continue;
        };
        for name in table.keys().filter(|name| predicate(name)) {
            out.push(LaneEvidence {
                kind: "dependency",
                field: Some(format!("{section}.{name}")),
                matched: None,
            });
        }
    }
    out.sort_by(|left, right| left.field.cmp(&right.field));
    out
}

fn path_evidence(matched: &str) -> LaneEvidence {
    LaneEvidence {
        kind: "path-convention",
        field: None,
        matched: Some(matched.to_string()),
    }
}

fn is_storybook_story_file(rel_file: &str) -> bool {
    let base = file_name(rel_file);
    let Some((prefix, extension)) = base.rsplit_once(".stories.") else {
        return false;
    };
    !prefix.is_empty() && JS_LIKE_EXTENSIONS.contains(&extension)
}

fn is_strapi_api_file(rel_file: &str) -> bool {
    let parts = rel_file.split('/').collect::<Vec<_>>();
    parts.len() >= 5
        && parts[0] == "src"
        && parts[1] == "api"
        && matches!(parts[3], "controllers" | "routes" | "services")
}

fn has_js_like_marker(base: &str, marker: &str) -> bool {
    let Some((_, extension)) = base.rsplit_once(marker) else {
        return false;
    };
    JS_LIKE_EXTENSIONS.contains(&extension)
}

fn file_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

fn has_path_segment(path: &str, segment: &str) -> bool {
    path.split('/').any(|part| part == segment)
}

fn contains_path_sequence(path: &str, sequence: &[&str]) -> bool {
    let parts = path.split('/').collect::<Vec<_>>();
    parts
        .windows(sequence.len())
        .any(|window| window == sequence)
}

fn build_summary(files: &[FrameworkResourceFile]) -> FrameworkResourceSummary {
    let mut by_lane = BTreeMap::new();
    let mut by_capability_pack = BTreeMap::new();
    let mut by_confidence = BTreeMap::new();
    let mut by_reason = BTreeMap::new();
    let mut by_framework = BTreeMap::new();
    let mut total_surface_lanes = 0;

    for file in files {
        for lane in &file.surface_lanes {
            total_surface_lanes += 1;
            increment(&mut by_lane, lane.lane);
            increment(&mut by_capability_pack, lane.capability_pack);
            increment(&mut by_confidence, lane.confidence);
            increment(&mut by_reason, lane.reason);
            if let Some(framework) = lane.framework {
                increment(&mut by_framework, framework);
            }
        }
    }

    FrameworkResourceSummary {
        total_files_with_surfaces: files.len(),
        total_surface_lanes,
        by_lane,
        by_capability_pack,
        by_confidence,
        by_reason,
        by_framework,
        top_examples: files
            .iter()
            .take(10)
            .map(|file| FrameworkResourceTopExample {
                file: file.file.clone(),
                lanes: file.surface_lanes.iter().map(|lane| lane.lane).collect(),
                capability_packs: file
                    .surface_lanes
                    .iter()
                    .map(|lane| lane.capability_pack)
                    .collect(),
                reasons: file.surface_lanes.iter().map(|lane| lane.reason).collect(),
            })
            .collect(),
    }
}

fn increment(map: &mut BTreeMap<String, usize>, key: &str) {
    *map.entry(key.to_string()).or_default() += 1;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn request(files: Vec<&str>, package_json: Value) -> FrameworkResourceSurfacesRequest {
        FrameworkResourceSurfacesRequest {
            schema_version: FRAMEWORK_RESOURCE_SURFACE_REQUEST_SCHEMA_VERSION.to_string(),
            root: "/repo".to_string(),
            files: files.into_iter().map(ToOwned::to_owned).collect(),
            package_records: vec![PackageRecord {
                root: "/repo".to_string(),
                rel_root: ".".to_string(),
                package_json,
            }],
            contents_by_file: BTreeMap::new(),
        }
    }

    fn first_lane<'a>(
        artifact: &'a FrameworkResourceSurfacesArtifact,
        file: &str,
    ) -> Result<&'a SurfaceLane> {
        artifact
            .files
            .iter()
            .find(|entry| entry.file == file)
            .and_then(|entry| entry.surface_lanes.first())
            .with_context(|| format!("expected surface lane for {file}"))
    }

    #[test]
    fn grounded_storybook_and_strapi_match_checked_shape() -> Result<()> {
        let artifact = build_framework_resource_surfaces_artifact(request(
            vec![
                "src/Button.stories.tsx",
                "src/api/article/controllers/article.ts",
                "src/api/article/routes/article.ts",
                "src/plain.ts",
            ],
            json!({
                "devDependencies": { "@storybook/react": "^8.0.0" },
                "dependencies": { "@strapi/strapi": "^5.0.0" }
            }),
        ))?;

        assert_eq!(
            artifact.schema_version,
            FRAMEWORK_RESOURCE_SURFACE_SCHEMA_VERSION
        );
        assert_eq!(
            artifact.policy_version,
            FRAMEWORK_RESOURCE_SURFACE_POLICY_VERSION
        );
        assert_eq!(artifact.summary.total_files_with_surfaces, 3);
        assert_eq!(
            artifact.summary.by_lane.get("framework-dispatch-entry"),
            Some(&3)
        );

        let storybook = first_lane(&artifact, "src/Button.stories.tsx")?;
        assert_eq!(storybook.capability_pack, "framework.storybook");
        assert_eq!(storybook.confidence, "grounded");
        assert_eq!(storybook.framework, Some("storybook"));
        assert_eq!(storybook.reason, "storybook-story-file");
        assert_eq!(storybook.evidence.len(), 2);
        assert_eq!(
            storybook.evidence[0].field.as_deref(),
            Some("devDependencies.@storybook/react")
        );

        let strapi = first_lane(&artifact, "src/api/article/controllers/article.ts")?;
        assert_eq!(strapi.capability_pack, "framework.strapi");
        assert_eq!(strapi.confidence, "grounded");
        assert_eq!(strapi.framework, Some("strapi"));
        Ok(())
    }

    #[test]
    fn path_only_frameworks_are_review_visible_not_grounded() -> Result<()> {
        let artifact = build_framework_resource_surfaces_artifact(request(
            vec![
                "src/Button.stories.tsx",
                "src/Button.story.tsx",
                "src/api/article/services/article.ts",
            ],
            json!({ "dependencies": {} }),
        ))?;

        assert!(artifact
            .files
            .iter()
            .all(|entry| entry.file != "src/Button.story.tsx"));
        for file in [
            "src/Button.stories.tsx",
            "src/api/article/services/article.ts",
        ] {
            let lane = first_lane(&artifact, file)?;
            assert_eq!(lane.lane, "framework-dispatch-entry");
            assert_eq!(lane.confidence, "path-shaped-review");
        }
        Ok(())
    }

    #[test]
    fn generated_bundle_template_and_codemod_lanes_are_deterministic() -> Result<()> {
        let mut request = request(
            vec![
                "templates/z.hbs",
                "types/generated/contentTypes.d.ts",
                "public/vendor.js",
                "dist/app.bundle.js",
                "dist/app.min.js",
                "src/emscripten-bindings.js",
                "resources/codemods/rename/input.ts",
                "__testfixtures__/fixture.ts",
            ],
            json!({}),
        );
        request.contents_by_file.insert(
            "src/emscripten-bindings.js".to_string(),
            "// @ts-nocheck\n// Generated by Emscripten\n".to_string(),
        );

        let artifact = build_framework_resource_surfaces_artifact(request)?;
        assert_eq!(
            artifact
                .files
                .iter()
                .map(|entry| entry.file.as_str())
                .collect::<Vec<_>>(),
            vec![
                "__testfixtures__/fixture.ts",
                "dist/app.bundle.js",
                "dist/app.min.js",
                "public/vendor.js",
                "resources/codemods/rename/input.ts",
                "src/emscripten-bindings.js",
                "templates/z.hbs",
                "types/generated/contentTypes.d.ts",
            ]
        );
        assert_eq!(
            artifact.summary.by_lane.get("bundled-build-artifact"),
            Some(&4)
        );
        assert_eq!(
            first_lane(&artifact, "src/emscripten-bindings.js")?.reason,
            "emscripten-generated-header"
        );
        Ok(())
    }

    #[test]
    fn nearest_package_controls_dependency_grounding() -> Result<()> {
        let artifact =
            build_framework_resource_surfaces_artifact(FrameworkResourceSurfacesRequest {
                schema_version: FRAMEWORK_RESOURCE_SURFACE_REQUEST_SCHEMA_VERSION.to_string(),
                root: "/repo".to_string(),
                files: vec![
                    "apps/web/src/Button.stories.tsx".to_string(),
                    "packages/ui/src/Button.stories.tsx".to_string(),
                ],
                package_records: vec![
                    PackageRecord {
                        root: "/repo".to_string(),
                        rel_root: ".".to_string(),
                        package_json: json!({ "devDependencies": { "storybook": "^8" } }),
                    },
                    PackageRecord {
                        root: "/repo/apps/web".to_string(),
                        rel_root: "apps/web".to_string(),
                        package_json: json!({ "devDependencies": { "@storybook/react": "^8" } }),
                    },
                    PackageRecord {
                        root: "/repo/packages/ui".to_string(),
                        rel_root: "packages/ui".to_string(),
                        package_json: json!({}),
                    },
                ],
                contents_by_file: BTreeMap::new(),
            })?;

        let app = artifact
            .files
            .iter()
            .find(|entry| entry.file == "apps/web/src/Button.stories.tsx")
            .context("apps/web story should be surfaced")?;
        assert_eq!(app.package_root, "apps/web");
        assert_eq!(app.surface_lanes[0].confidence, "grounded");

        let ui = artifact
            .files
            .iter()
            .find(|entry| entry.file == "packages/ui/src/Button.stories.tsx")
            .context("packages/ui story should be surfaced")?;
        assert_eq!(ui.package_root, "packages/ui");
        assert_eq!(ui.surface_lanes[0].confidence, "path-shaped-review");
        Ok(())
    }

    #[test]
    fn invalid_file_paths_are_rejected_at_request_boundary() -> Result<()> {
        let err = build_framework_resource_surfaces_artifact(request(
            vec!["../outside/Button.stories.tsx"],
            json!({}),
        ))
        .err()
        .context("invalid path should be rejected")?;
        assert!(err.to_string().contains("traversal"));
        Ok(())
    }
}
