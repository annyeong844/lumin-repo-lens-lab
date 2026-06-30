use std::collections::btree_map;

use lumin_rust_source_health::protocol::{
    AstFunctionCloneGroups, CompactFileHealth, FileHealth, HealthResponse, ResponseMeta,
    SkippedFile, Summary,
};
use lumin_rust_source_health::CompactAnalysisResponse;

pub(crate) enum SyntaxPhaseOwned {
    Full(HealthResponse),
    Compact(CompactAnalysisResponse),
}

impl SyntaxPhaseOwned {
    pub(crate) fn as_phase(&self) -> SyntaxPhase<'_> {
        match self {
            Self::Full(response) => SyntaxPhase::Full(response),
            Self::Compact(response) => SyntaxPhase::Compact(response),
        }
    }

    pub(crate) fn full_response(&self) -> Option<&HealthResponse> {
        match self {
            Self::Full(response) => Some(response),
            Self::Compact(_) => None,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum SyntaxPhase<'a> {
    Full(&'a HealthResponse),
    Compact(&'a CompactAnalysisResponse),
}

impl<'a> SyntaxPhase<'a> {
    pub(crate) fn schema_version(self) -> u32 {
        match self {
            Self::Full(response) => response.schema_version,
            Self::Compact(response) => response.schema_version,
        }
    }

    pub(crate) fn meta(self) -> &'a ResponseMeta {
        match self {
            Self::Full(response) => &response.meta,
            Self::Compact(response) => &response.meta,
        }
    }

    pub(crate) fn summary(self) -> &'a Summary {
        match self {
            Self::Full(response) => &response.summary,
            Self::Compact(response) => &response.summary,
        }
    }

    pub(crate) fn function_clone_groups(self) -> &'a AstFunctionCloneGroups {
        match self {
            Self::Full(response) => &response.function_clone_groups,
            Self::Compact(response) => &response.function_clone_groups,
        }
    }

    pub(crate) fn skipped_files(self) -> &'a [SkippedFile] {
        match self {
            Self::Full(response) => &response.skipped_files,
            Self::Compact(response) => &response.skipped_files,
        }
    }

    pub(crate) fn files(self) -> SyntaxFileIter<'a> {
        match self {
            Self::Full(response) => SyntaxFileIter::Full(response.files.iter()),
            Self::Compact(response) => SyntaxFileIter::Compact(response.files.iter()),
        }
    }

    pub(crate) fn full_file(self, path: &str) -> Option<&'a FileHealth> {
        match self {
            Self::Full(response) => response.files.get(path),
            Self::Compact(_) => None,
        }
    }
}

pub(crate) enum SyntaxFileIter<'a> {
    Full(btree_map::Iter<'a, String, FileHealth>),
    Compact(btree_map::Iter<'a, String, CompactFileHealth>),
}

impl<'a> Iterator for SyntaxFileIter<'a> {
    type Item = (&'a str, SyntaxFile<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Full(iter) => iter
                .next()
                .map(|(path, file)| (path.as_str(), SyntaxFile::Full(file))),
            Self::Compact(iter) => iter
                .next()
                .map(|(path, file)| (path.as_str(), SyntaxFile::Compact(file))),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum SyntaxFile<'a> {
    Full(&'a FileHealth),
    Compact(&'a CompactFileHealth),
}
