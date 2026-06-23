use super::super::graph::DependencyImportObservation;
use super::super::model::{DependencyImportExample, DEPENDENCY_EXAMPLE_LIMIT};

pub(super) fn dependency_examples(
    observations: &[&DependencyImportObservation],
) -> Vec<DependencyImportExample> {
    observations
        .iter()
        .take(DEPENDENCY_EXAMPLE_LIMIT)
        .map(|observation| DependencyImportExample {
            file: observation.file.clone(),
            from_spec: observation.from_spec.clone(),
        })
        .collect()
}
