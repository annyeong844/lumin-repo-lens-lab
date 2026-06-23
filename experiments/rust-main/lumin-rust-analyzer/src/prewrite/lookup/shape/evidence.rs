use super::model::{ShapeLookup, UnavailableEvidence};

pub(in crate::prewrite) fn unavailable_evidence_from_shape_lookups(
    lookups: &[ShapeLookup],
) -> Vec<UnavailableEvidence> {
    lookups
        .iter()
        .filter(|lookup| lookup.is_unavailable())
        .map(ShapeLookup::unavailable_evidence)
        .collect()
}
