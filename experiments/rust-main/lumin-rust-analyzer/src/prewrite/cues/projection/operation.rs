use std::collections::BTreeMap;

use crate::prewrite::lookup::NameLookup;

use crate::prewrite::cues::model::{CueCardBuilder, SuppressedCue};

mod local;
mod service;

pub(super) fn add_operation_cues(
    lookup: &NameLookup,
    cards: &mut BTreeMap<String, CueCardBuilder>,
    suppressed: &mut Vec<SuppressedCue>,
) {
    service::add_service_operation_sibling_policy(lookup, cards, suppressed);
    local::add_local_operation_sibling_policy(lookup, cards, suppressed);
}
