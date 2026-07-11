mod assembly;
mod glob;
mod input;
mod namespace;
mod path;
mod protocol;

pub use assembly::{
    build_embedded_source_use_assembly_response, build_source_use_assembly_response,
};
pub use protocol::{
    DependencyImportConsumerAddition, DirectConsumerAddition, NamespaceReExportChainEntry,
    NamespaceReExportDiagnosticAddition, NamespaceUserAddition, ResolvedInternalEdge,
    ResolvedRecordTarget, SkippedSourceUseRecord, SourceUseAssemblyCounters,
    SourceUseAssemblyReExport, SourceUseAssemblyRecordInput, SourceUseAssemblyRequest,
    SourceUseAssemblyResponse, SourceUseAssemblySummary,
    SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION, SOURCE_USE_ASSEMBLY_RESPONSE_SCHEMA_VERSION,
};

pub(crate) use assembly::build_embedded_source_use_assembly_response_with_path_table;
