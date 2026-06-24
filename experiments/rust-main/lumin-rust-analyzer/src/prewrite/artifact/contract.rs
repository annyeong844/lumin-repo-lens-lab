use anyhow::{bail, Result};

use crate::prewrite::cues::{self, CueMatchedField, CueTier, EvidenceLane};

use super::{intent_has_file, PreWriteArtifact};

impl PreWriteArtifact {
    pub(crate) fn validate_contract(&self) -> Result<()> {
        self.validate_safe_cues()?;
        self.validate_suppressed_cues()?;
        self.validate_name_lookups()?;
        self.validate_shape_lookups()?;
        self.validate_file_lookups()?;
        self.validate_dependency_lookups()?;
        self.validate_inline_pattern_lookups()?;
        self.coverage.validate(&self.intent)?;
        Ok(())
    }

    fn validate_safe_cues(&self) -> Result<()> {
        for card in &self.cue_cards {
            for cue in &card.cues {
                if cue.cue_tier == CueTier::Safe {
                    match cue.evidence_lane {
                        EvidenceLane::ExactSymbol
                            if cue.evidence.iter().all(|evidence| {
                                matches!(
                                    evidence.matched_field,
                                    CueMatchedField::DefIndex
                                        | CueMatchedField::RustSourceHealthUseTrees
                                )
                            }) => {}
                        EvidenceLane::ExactFile
                            if cue.evidence.iter().all(|evidence| {
                                evidence.matched_field == CueMatchedField::RustSourceHealthFiles
                            }) => {}
                        EvidenceLane::ShapeHash
                            if cue.evidence.iter().all(|evidence| {
                                evidence.matched_field
                                    == CueMatchedField::RustSourceHealthShapeHash
                                    && evidence.hash.is_some()
                            }) => {}
                        EvidenceLane::FunctionSignature
                            if cue.evidence.iter().all(|evidence| {
                                evidence.matched_field
                                    == CueMatchedField::RustSourceHealthFunctionSignatureHash
                                    && evidence.hash.is_some()
                            }) => {}
                        EvidenceLane::InlineExtraction => bail!(
                            "blocked-artifact-contract: inline extraction cue {} entered SAFE",
                            card.candidate.identity
                        ),
                        _ => bail!(
                            "blocked-artifact-contract: SAFE cue {} is not exact source-health evidence",
                            card.candidate.identity
                        ),
                    }
                }
                if cue.evidence.iter().any(|evidence| {
                    evidence.matched_field == CueMatchedField::ImplMethodIndex
                        && cue.cue_tier == CueTier::Safe
                }) {
                    bail!(
                        "blocked-artifact-contract: impl method {} entered SAFE",
                        card.candidate.identity
                    );
                }
            }
        }
        Ok(())
    }

    fn validate_suppressed_cues(&self) -> Result<()> {
        for cue in &self.suppressed_cues {
            if cue.reason == cues::MutedReason::PolicyExcluded
                && (cue.original_cue_tier.is_none() || cue.path_classifications.is_empty())
            {
                bail!(
                    "blocked-artifact-contract: policy-muted cue {} lost original tier or path evidence",
                    cue.candidate.identity
                );
            }
        }
        Ok(())
    }

    fn validate_name_lookups(&self) -> Result<()> {
        for lookup in &self.lookups {
            if !self.intent.names.contains(&lookup.intent_name) {
                bail!(
                    "blocked-artifact-contract: lookup name {} is absent from normalized intent",
                    lookup.intent_name
                );
            }
        }
        Ok(())
    }

    fn validate_shape_lookups(&self) -> Result<()> {
        if self.shape_lookups.len() != self.intent.shapes.len() {
            bail!("blocked-artifact-contract: shape lookup count drifted from normalized intent");
        }
        for (lookup, intent_shape) in self.shape_lookups.iter().zip(&self.intent.shapes) {
            if &lookup.shape != intent_shape {
                bail!("blocked-artifact-contract: shape lookup drifted from normalized intent");
            }
            if !lookup.is_unavailable() && !lookup.is_match() {
                bail!("blocked-artifact-contract: shape lookup emitted an invalid result");
            }
        }
        let unavailable_lookup_count = self
            .shape_lookups
            .iter()
            .filter(|lookup| lookup.is_unavailable())
            .count()
            + self
                .inline_pattern_lookups
                .iter()
                .filter(|lookup| lookup.is_unavailable())
                .count();
        if self.unavailable_evidence.len() != unavailable_lookup_count {
            bail!(
                "blocked-artifact-contract: unavailable evidence drifted from unavailable lookups"
            );
        }
        Ok(())
    }

    fn validate_file_lookups(&self) -> Result<()> {
        for lookup in &self.file_lookups {
            if !intent_has_file(&self.intent, &lookup.intent_file) {
                bail!(
                    "blocked-artifact-contract: lookup file {} is absent from normalized intent",
                    lookup.intent_file
                );
            }
        }
        Ok(())
    }

    fn validate_dependency_lookups(&self) -> Result<()> {
        if self.dependency_lookups.len() != self.intent.dependencies.len() {
            bail!(
                "blocked-artifact-contract: dependency lookup count drifted from normalized intent"
            );
        }
        for (lookup, dependency) in self
            .dependency_lookups
            .iter()
            .zip(&self.intent.dependencies)
        {
            if &lookup.dep_name != dependency {
                bail!(
                    "blocked-artifact-contract: dependency lookup {} drifted from normalized intent",
                    lookup.dep_name
                );
            }
        }
        Ok(())
    }

    fn validate_inline_pattern_lookups(&self) -> Result<()> {
        let expected_inline_lookups = usize::from(self.intent.has_refactor_sources());
        if self.inline_pattern_lookups.len() != expected_inline_lookups {
            bail!(
                "blocked-artifact-contract: inline-pattern lookup count drifted from refactorSources"
            );
        }
        for lookup in &self.inline_pattern_lookups {
            if !(lookup.is_unavailable() || lookup.is_match() || lookup.is_no_match()) {
                bail!("blocked-artifact-contract: inline-pattern lookup emitted an invalid result");
            }
        }
        Ok(())
    }
}
