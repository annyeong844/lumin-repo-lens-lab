use crate::prewrite::lookup::unavailable::UnavailableEvidence;

use super::model::ShapeLookup;

pub(in crate::prewrite) fn unavailable_evidence_from_shape_lookups(
    lookups: &[ShapeLookup],
) -> Vec<UnavailableEvidence> {
    lookups
        .iter()
        .filter(|lookup| lookup.is_unavailable())
        .map(ShapeLookup::unavailable_evidence)
        .collect()
}
