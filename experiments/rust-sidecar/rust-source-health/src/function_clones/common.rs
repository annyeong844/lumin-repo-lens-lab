use std::collections::BTreeMap;

use crate::protocol::{
    AstFunctionBodyFingerprint, AstFunctionParam, AstFunctionReceiver, AstFunctionSignature,
    FileHealth, PathClassification,
};

#[derive(Clone, Copy)]
pub(super) struct GroupMember<'a> {
    pub(super) file: &'a str,
    pub(super) fact: &'a AstFunctionBodyFingerprint,
    pub(super) generated: bool,
}

#[derive(Clone, Copy)]
pub(super) struct SignatureMember<'a> {
    pub(super) file: &'a str,
    pub(super) fact: &'a AstFunctionSignature,
    pub(super) generated: bool,
}

pub(super) fn function_members(files: &BTreeMap<String, FileHealth>) -> Vec<GroupMember<'_>> {
    files
        .iter()
        .flat_map(|(file, health)| {
            let generated = health
                .path
                .classifications
                .contains(&PathClassification::Generated);
            health
                .ast
                .function_body_fingerprints
                .iter()
                .map(move |fact| GroupMember {
                    file: file.as_str(),
                    fact,
                    generated,
                })
        })
        .collect()
}

pub(super) fn member_identity(member: &GroupMember<'_>) -> String {
    match &member.fact.owner {
        None => format!("{}::{}", member.file, member.fact.name),
        Some(owner) => match &owner.trait_path {
            None => format!("{}::{}#{}", member.file, owner.target, member.fact.name),
            Some(trait_path) => format!(
                "{}::{} as {}#{}",
                member.file, owner.target, trait_path, member.fact.name
            ),
        },
    }
}

pub(super) fn signature_member_identity(member: &SignatureMember<'_>) -> String {
    match &member.fact.owner {
        None => format!("{}::{}", member.file, member.fact.name),
        Some(owner) => match &owner.trait_path {
            None => format!("{}::{}#{}", member.file, owner.target, member.fact.name),
            Some(trait_path) => format!(
                "{}::{} as {}#{}",
                member.file, owner.target, trait_path, member.fact.name
            ),
        },
    }
}

pub(super) fn signature_text(signature: &AstFunctionSignature) -> String {
    let mut params = Vec::new();
    if let Some(receiver) = &signature.receiver {
        params.push(receiver_text(receiver));
    }
    params.extend(signature.params.iter().map(param_text));

    let mut text = String::from("fn");
    if let Some(generics) = &signature.generics {
        text.push_str(generics);
    }
    text.push('(');
    text.push_str(&params.join(", "));
    text.push(')');
    if let Some(return_type) = &signature.return_type {
        text.push_str(" -> ");
        text.push_str(return_type);
    }
    text
}

fn receiver_text(receiver: &AstFunctionReceiver) -> String {
    receiver.text.clone()
}

fn param_text(param: &AstFunctionParam) -> String {
    param.type_text.clone()
}
