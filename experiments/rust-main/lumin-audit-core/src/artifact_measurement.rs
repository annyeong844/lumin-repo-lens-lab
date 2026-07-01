use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::orchestration_events::{ArtifactSizeBytes, ArtifactSizeEntry, ArtifactSizeSummary};

const LARGEST_ARTIFACT_LIMIT: usize = 10;

pub fn measure_artifact_sizes(output: &Path, artifacts: &[String]) -> ArtifactSizeSummary {
    let mut measured = Vec::new();

    for name in artifacts {
        let artifact_path = output.join(name);
        let Ok(metadata) = fs::metadata(&artifact_path) else {
            continue;
        };
        if !metadata.is_file() {
            continue;
        }
        measured.push(ArtifactSizeEntry {
            name: name.clone(),
            bytes: metadata.len(),
        });
    }

    measured.sort_by(|left, right| left.name.cmp(&right.name));
    let total_bytes = measured.iter().map(|entry| entry.bytes).sum();
    let mut by_name = BTreeMap::new();
    for entry in &measured {
        by_name.insert(entry.name.clone(), ArtifactSizeBytes { bytes: entry.bytes });
    }

    let mut largest_entries = measured.clone();
    largest_entries.sort_by(|left, right| {
        right
            .bytes
            .cmp(&left.bytes)
            .then_with(|| left.name.cmp(&right.name))
    });
    let largest = largest_entries
        .into_iter()
        .take(LARGEST_ARTIFACT_LIMIT)
        .collect();

    ArtifactSizeSummary {
        produced_count: measured.len() as u64,
        total_bytes,
        largest,
        by_name,
    }
}
