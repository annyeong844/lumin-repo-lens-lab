use std::collections::BTreeMap;

use crate::protocol::{
    AstFunctionBodyFingerprint, AstFunctionOwner, AstFunctionSignature, AstVisibility, FileHealth,
    PathClassification,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FunctionCloneFile {
    parse_ok: bool,
    parse_error_message: Option<String>,
    generated: bool,
    function_body_fingerprints: Vec<FunctionCloneBodyFact>,
    function_signatures: Vec<FunctionCloneSignatureFact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FunctionCloneBodyFact {
    name: Box<str>,
    visibility: AstVisibility,
    #[serde(skip_serializing_if = "Option::is_none")]
    owner: Option<AstFunctionOwner>,
    exact_body_hash: Box<str>,
    normalized_exact_hash: Box<str>,
    normalized_structure_hash: Box<str>,
    body_loc: usize,
    statement_count: usize,
    param_count: usize,
    #[serde(rename = "async")]
    is_async: bool,
    #[serde(rename = "unsafe")]
    is_unsafe: bool,
    #[serde(rename = "const")]
    is_const: bool,
    call_tokens: Vec<Box<str>>,
    line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FunctionCloneSignatureFact {
    hash: Box<str>,
    name: Box<str>,
    visibility: AstVisibility,
    #[serde(skip_serializing_if = "Option::is_none")]
    owner: Option<AstFunctionOwner>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generics: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    receiver_text: Option<Box<str>>,
    params: Vec<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    return_type: Option<Box<str>>,
    line: usize,
}

impl FunctionCloneFile {
    pub(crate) fn from_health(mut health: FileHealth) -> Self {
        let parse_error_message = health
            .parse
            .errors
            .first()
            .map(|error| error.message.clone());
        Self {
            parse_ok: health.parse.ok,
            parse_error_message,
            generated: health
                .path
                .classifications
                .contains(&PathClassification::Generated),
            function_body_fingerprints: std::mem::take(&mut health.ast.function_body_fingerprints)
                .into_iter()
                .map(FunctionCloneBodyFact::from)
                .collect(),
            function_signatures: std::mem::take(&mut health.ast.function_signatures)
                .into_iter()
                .map(FunctionCloneSignatureFact::from)
                .collect(),
        }
    }

    pub(super) fn prune_grouped_lanes_for_near(&mut self) {
        for fact in &mut self.function_body_fingerprints {
            fact.exact_body_hash = "".into();
            fact.normalized_exact_hash = "".into();
            fact.normalized_structure_hash = "".into();
        }
        self.function_signatures = Vec::new();
    }
}

pub(super) trait FunctionCloneFileView {
    type BodyFact: FunctionBodyFactView;
    type SignatureFact: FunctionSignatureFactView;

    fn parse_ok(&self) -> bool;
    fn parse_error_message(&self) -> Option<&str>;
    fn generated(&self) -> bool;
    fn function_body_fingerprints(&self) -> &[Self::BodyFact];
    fn function_signatures(&self) -> &[Self::SignatureFact];
}

pub(super) trait FunctionBodyFactView {
    type CallToken: AsRef<str>;

    fn name(&self) -> &str;
    fn visibility(&self) -> AstVisibility;
    fn owner(&self) -> Option<&AstFunctionOwner>;
    fn exact_body_hash(&self) -> &str;
    fn normalized_exact_hash(&self) -> &str;
    fn normalized_structure_hash(&self) -> &str;
    fn body_loc(&self) -> usize;
    fn statement_count(&self) -> usize;
    fn param_count(&self) -> usize;
    fn is_async(&self) -> bool;
    fn is_unsafe(&self) -> bool;
    fn is_const(&self) -> bool;
    fn call_tokens(&self) -> &[Self::CallToken];
    fn line(&self) -> usize;
}

pub(super) trait FunctionSignatureFactView {
    fn hash(&self) -> &str;
    fn name(&self) -> &str;
    fn visibility(&self) -> AstVisibility;
    fn owner(&self) -> Option<&AstFunctionOwner>;
    fn generics(&self) -> Option<&str>;
    fn receiver_text(&self) -> Option<&str>;
    fn param_type_texts(&self) -> Vec<&str>;
    fn return_type(&self) -> Option<&str>;
    fn line(&self) -> usize;
}

impl From<AstFunctionBodyFingerprint> for FunctionCloneBodyFact {
    fn from(fact: AstFunctionBodyFingerprint) -> Self {
        Self {
            name: fact.name.into_boxed_str(),
            visibility: fact.visibility,
            owner: fact.owner,
            exact_body_hash: fact.exact_body_hash.into_boxed_str(),
            normalized_exact_hash: fact.normalized_exact_hash.into_boxed_str(),
            normalized_structure_hash: fact.normalized_structure_hash.into_boxed_str(),
            body_loc: fact.body_loc,
            statement_count: fact.statement_count,
            param_count: fact.param_count,
            is_async: fact.is_async,
            is_unsafe: fact.is_unsafe,
            is_const: fact.is_const,
            call_tokens: fact
                .call_tokens
                .into_iter()
                .map(String::into_boxed_str)
                .collect(),
            line: fact.location.line,
        }
    }
}

impl From<AstFunctionSignature> for FunctionCloneSignatureFact {
    fn from(fact: AstFunctionSignature) -> Self {
        Self {
            hash: fact.hash.into_boxed_str(),
            name: fact.name.into_boxed_str(),
            visibility: fact.visibility,
            owner: fact.owner,
            generics: fact.generics.map(String::into_boxed_str),
            receiver_text: fact.receiver.map(|receiver| receiver.text.into_boxed_str()),
            params: fact
                .params
                .into_iter()
                .map(|param| param.type_text.into_boxed_str())
                .collect(),
            return_type: fact.return_type.map(String::into_boxed_str),
            line: fact.location.line,
        }
    }
}

impl FunctionCloneFileView for FileHealth {
    type BodyFact = AstFunctionBodyFingerprint;
    type SignatureFact = AstFunctionSignature;

    fn parse_ok(&self) -> bool {
        self.parse.ok
    }

    fn parse_error_message(&self) -> Option<&str> {
        self.parse
            .errors
            .first()
            .map(|error| error.message.as_str())
    }

    fn generated(&self) -> bool {
        self.path
            .classifications
            .contains(&PathClassification::Generated)
    }

    fn function_body_fingerprints(&self) -> &[Self::BodyFact] {
        &self.ast.function_body_fingerprints
    }

    fn function_signatures(&self) -> &[Self::SignatureFact] {
        &self.ast.function_signatures
    }
}

impl FunctionCloneFileView for FunctionCloneFile {
    type BodyFact = FunctionCloneBodyFact;
    type SignatureFact = FunctionCloneSignatureFact;

    fn parse_ok(&self) -> bool {
        self.parse_ok
    }

    fn parse_error_message(&self) -> Option<&str> {
        self.parse_error_message.as_deref()
    }

    fn generated(&self) -> bool {
        self.generated
    }

    fn function_body_fingerprints(&self) -> &[Self::BodyFact] {
        &self.function_body_fingerprints
    }

    fn function_signatures(&self) -> &[Self::SignatureFact] {
        &self.function_signatures
    }
}

impl FunctionBodyFactView for AstFunctionBodyFingerprint {
    type CallToken = String;

    fn name(&self) -> &str {
        &self.name
    }

    fn visibility(&self) -> AstVisibility {
        self.visibility
    }

    fn owner(&self) -> Option<&AstFunctionOwner> {
        self.owner.as_ref()
    }

    fn exact_body_hash(&self) -> &str {
        &self.exact_body_hash
    }

    fn normalized_exact_hash(&self) -> &str {
        &self.normalized_exact_hash
    }

    fn normalized_structure_hash(&self) -> &str {
        &self.normalized_structure_hash
    }

    fn body_loc(&self) -> usize {
        self.body_loc
    }

    fn statement_count(&self) -> usize {
        self.statement_count
    }

    fn param_count(&self) -> usize {
        self.param_count
    }

    fn is_async(&self) -> bool {
        self.is_async
    }

    fn is_unsafe(&self) -> bool {
        self.is_unsafe
    }

    fn is_const(&self) -> bool {
        self.is_const
    }

    fn call_tokens(&self) -> &[Self::CallToken] {
        &self.call_tokens
    }

    fn line(&self) -> usize {
        self.location.line
    }
}

impl FunctionBodyFactView for FunctionCloneBodyFact {
    type CallToken = Box<str>;

    fn name(&self) -> &str {
        &self.name
    }

    fn visibility(&self) -> AstVisibility {
        self.visibility
    }

    fn owner(&self) -> Option<&AstFunctionOwner> {
        self.owner.as_ref()
    }

    fn exact_body_hash(&self) -> &str {
        &self.exact_body_hash
    }

    fn normalized_exact_hash(&self) -> &str {
        &self.normalized_exact_hash
    }

    fn normalized_structure_hash(&self) -> &str {
        &self.normalized_structure_hash
    }

    fn body_loc(&self) -> usize {
        self.body_loc
    }

    fn statement_count(&self) -> usize {
        self.statement_count
    }

    fn param_count(&self) -> usize {
        self.param_count
    }

    fn is_async(&self) -> bool {
        self.is_async
    }

    fn is_unsafe(&self) -> bool {
        self.is_unsafe
    }

    fn is_const(&self) -> bool {
        self.is_const
    }

    fn call_tokens(&self) -> &[Self::CallToken] {
        &self.call_tokens
    }

    fn line(&self) -> usize {
        self.line
    }
}

impl FunctionSignatureFactView for AstFunctionSignature {
    fn hash(&self) -> &str {
        &self.hash
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn visibility(&self) -> AstVisibility {
        self.visibility
    }

    fn owner(&self) -> Option<&AstFunctionOwner> {
        self.owner.as_ref()
    }

    fn generics(&self) -> Option<&str> {
        self.generics.as_deref()
    }

    fn receiver_text(&self) -> Option<&str> {
        self.receiver
            .as_ref()
            .map(|receiver| receiver.text.as_str())
    }

    fn param_type_texts(&self) -> Vec<&str> {
        self.params
            .iter()
            .map(|param| param.type_text.as_str())
            .collect()
    }

    fn return_type(&self) -> Option<&str> {
        self.return_type.as_deref()
    }

    fn line(&self) -> usize {
        self.location.line
    }
}

impl FunctionSignatureFactView for FunctionCloneSignatureFact {
    fn hash(&self) -> &str {
        &self.hash
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn visibility(&self) -> AstVisibility {
        self.visibility
    }

    fn owner(&self) -> Option<&AstFunctionOwner> {
        self.owner.as_ref()
    }

    fn generics(&self) -> Option<&str> {
        self.generics.as_deref()
    }

    fn receiver_text(&self) -> Option<&str> {
        self.receiver_text.as_deref()
    }

    fn param_type_texts(&self) -> Vec<&str> {
        self.params.iter().map(AsRef::as_ref).collect()
    }

    fn return_type(&self) -> Option<&str> {
        self.return_type.as_deref()
    }

    fn line(&self) -> usize {
        self.line
    }
}

#[derive(Clone, Copy)]
pub(super) struct GroupMember<'a, B: FunctionBodyFactView> {
    pub(super) file: &'a str,
    pub(super) fact: &'a B,
    pub(super) generated: bool,
}

#[derive(Clone, Copy)]
pub(super) struct SignatureMember<'a, S: FunctionSignatureFactView> {
    pub(super) file: &'a str,
    pub(super) fact: &'a S,
    pub(super) generated: bool,
}

pub(super) fn function_members<F: FunctionCloneFileView>(
    files: &BTreeMap<String, F>,
) -> Vec<GroupMember<'_, F::BodyFact>> {
    files
        .iter()
        .flat_map(|(file, health)| {
            let generated = health.generated();
            health
                .function_body_fingerprints()
                .iter()
                .map(move |fact| GroupMember {
                    file: file.as_str(),
                    fact,
                    generated,
                })
        })
        .collect()
}

pub(super) fn member_identity<B: FunctionBodyFactView>(member: &GroupMember<'_, B>) -> String {
    match member.fact.owner() {
        None => format!("{}::{}", member.file, member.fact.name()),
        Some(owner) => match &owner.trait_path {
            None => format!("{}::{}#{}", member.file, owner.target, member.fact.name()),
            Some(trait_path) => format!(
                "{}::{} as {}#{}",
                member.file,
                owner.target,
                trait_path,
                member.fact.name()
            ),
        },
    }
}

pub(super) fn signature_member_identity<S: FunctionSignatureFactView>(
    member: &SignatureMember<'_, S>,
) -> String {
    match member.fact.owner() {
        None => format!("{}::{}", member.file, member.fact.name()),
        Some(owner) => match &owner.trait_path {
            None => format!("{}::{}#{}", member.file, owner.target, member.fact.name()),
            Some(trait_path) => format!(
                "{}::{} as {}#{}",
                member.file,
                owner.target,
                trait_path,
                member.fact.name()
            ),
        },
    }
}

pub(super) fn signature_text<S: FunctionSignatureFactView>(signature: &S) -> String {
    let mut params = Vec::new();
    if let Some(receiver) = signature.receiver_text() {
        params.push(receiver.to_string());
    }
    params.extend(signature.param_type_texts().into_iter().map(str::to_string));

    let mut text = String::from("fn");
    if let Some(generics) = signature.generics() {
        text.push_str(generics);
    }
    text.push('(');
    text.push_str(&params.join(", "));
    text.push(')');
    if let Some(return_type) = signature.return_type() {
        text.push_str(" -> ");
        text.push_str(return_type);
    }
    text
}
