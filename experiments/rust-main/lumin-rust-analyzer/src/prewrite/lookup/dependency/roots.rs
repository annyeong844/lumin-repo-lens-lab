use std::collections::BTreeSet;

pub(super) fn manifest_key_candidates(root: &str) -> Vec<String> {
    dedupe_candidates([
        root.to_string(),
        root.replace('_', "-"),
        root.replace('-', "_"),
    ])
}

pub(super) fn rust_code_root_candidates(root: &str) -> BTreeSet<String> {
    BTreeSet::from([root.to_string(), root.replace('-', "_")])
}

fn dedupe_candidates<const N: usize>(candidates: [String; N]) -> Vec<String> {
    let mut seen = BTreeSet::new();
    candidates
        .into_iter()
        .filter(|candidate| seen.insert(candidate.clone()))
        .collect()
}
