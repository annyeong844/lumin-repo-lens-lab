use anyhow::Result;

use crate::prewrite::intent::model::PlannedTypeEscape;

use super::validate::require_non_empty;

pub(super) fn validate_planned_type_escapes(entries: &[PlannedTypeEscape]) -> Result<()> {
    for (index, entry) in entries.iter().enumerate() {
        require_non_empty(
            &entry.location_hint,
            &format!("plannedTypeEscapes[{index}].locationHint"),
        )?;
        require_non_empty(
            &entry.reason,
            &format!("plannedTypeEscapes[{index}].reason"),
        )?;
    }
    Ok(())
}
