use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Deserialize)]
pub struct ScanRequest {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    pub root: String,
    pub files: Vec<String>,
    #[serde(rename = "policyVersion")]
    pub policy_version: PolicyVersion,
}

#[derive(Debug, Serialize)]
pub struct ScanResponse {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(rename = "policyVersion")]
    pub policy_version: PolicyVersion,
    pub files: Vec<FileScanResult>,
    pub timing: Timing,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PolicyVersion {
    ModuleEdgeScannerV1,
    Unknown(String),
}

impl PolicyVersion {
    pub fn current() -> Self {
        Self::ModuleEdgeScannerV1
    }

    pub fn is_supported(&self) -> bool {
        matches!(self, Self::ModuleEdgeScannerV1)
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::ModuleEdgeScannerV1 => "module-edge-scanner-v1",
            Self::Unknown(value) => value,
        }
    }
}

impl Serialize for PolicyVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for PolicyVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(match value.as_str() {
            "module-edge-scanner-v1" => Self::ModuleEdgeScannerV1,
            _ => Self::Unknown(value),
        })
    }
}

#[derive(Debug, Serialize, PartialEq, Eq, Clone)]
pub struct ModuleEdge {
    pub source: String,
    pub line: usize,
    #[serde(rename = "typeOnly")]
    pub type_only: bool,
    #[serde(rename = "reExport")]
    pub re_export: bool,
    pub dynamic: bool,
}

#[derive(Debug, Serialize)]
pub struct FileScanResult {
    pub file: String,
    pub ok: bool,
    pub loc: usize,
    pub edges: Vec<ModuleEdge>,
    pub risk: Vec<RiskKind>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RiskKind {
    DecoratorOrReflect,
    DynamicImportOptions,
    ImportMetaGlob,
    NonLiteralDynamicImport,
    RequireCall,
    RequireContext,
    ScannerStateAmbiguous,
    TemplateDynamicImport,
    TsAmbientModule,
    TsExportAssignment,
    TsImportEquals,
    UnsupportedSyntax,
}

impl RiskKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DecoratorOrReflect => "decorator-or-reflect",
            Self::DynamicImportOptions => "dynamic-import-options",
            Self::ImportMetaGlob => "import-meta-glob",
            Self::NonLiteralDynamicImport => "non-literal-dynamic-import",
            Self::RequireCall => "require-call",
            Self::RequireContext => "require-context",
            Self::ScannerStateAmbiguous => "scanner-state-ambiguous",
            Self::TemplateDynamicImport => "template-dynamic-import",
            Self::TsAmbientModule => "ts-ambient-module",
            Self::TsExportAssignment => "ts-export-assignment",
            Self::TsImportEquals => "ts-import-equals",
            Self::UnsupportedSyntax => "unsupported-syntax",
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Timing {
    pub files: usize,
    #[serde(rename = "elapsedMs")]
    pub elapsed_ms: u128,
}
