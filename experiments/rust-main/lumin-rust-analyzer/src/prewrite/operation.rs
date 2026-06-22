use std::collections::BTreeSet;

use serde::Serialize;

use super::tokens::{unique_prewrite_tokens, unique_tokens};

const READ_QUERY_VERBS: &[&str] = &[
    "fetch", "find", "get", "list", "load", "lookup", "query", "read", "resolve", "retrieve",
    "search",
];
const LOCAL_OPERATION_MUTATION_VERBS: &[&str] = &[
    "add", "create", "delete", "destroy", "dispatch", "emit", "patch", "remove", "save", "send",
    "set", "update", "upsert", "write",
];
const LOCAL_OPERATION_CONTAINER_START: &[&str] = &["build", "create", "make"];
const LOCAL_OPERATION_CONTAINER_DOMAIN: &[&str] = &["repository", "service"];

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(in crate::prewrite) enum ServiceOperationFamily {
    ReadQuery,
    MutationCreate,
    MutationDelete,
    MutationSend,
    MutationUpdate,
    MutationSave,
}

impl ServiceOperationFamily {
    pub(in crate::prewrite) fn as_str(self) -> &'static str {
        match self {
            Self::ReadQuery => "read-query",
            Self::MutationCreate => "mutation-create",
            Self::MutationDelete => "mutation-delete",
            Self::MutationSend => "mutation-send",
            Self::MutationUpdate => "mutation-update",
            Self::MutationSave => "mutation-save",
        }
    }
}

#[derive(Debug)]
pub(in crate::prewrite) struct OperationInfo {
    pub(in crate::prewrite) operation_family: Option<ServiceOperationFamily>,
    pub(in crate::prewrite) domain_tokens: Vec<String>,
}

pub(in crate::prewrite) fn service_operation_info(name: &str) -> OperationInfo {
    let tokens = unique_tokens(&[name]);
    let verb = tokens.first().map(String::as_str);
    let operation_family = verb.and_then(operation_family_for_verb);
    let domain_tokens = tokens
        .iter()
        .filter(|token| Some(token.as_str()) != verb && operation_family_for_verb(token).is_none())
        .filter_map(|token| normalize_domain_token(token))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    OperationInfo {
        operation_family,
        domain_tokens,
    }
}

pub(in crate::prewrite) fn local_operation_info(name: &str) -> Option<OperationInfo> {
    let tokens = unique_prewrite_tokens(&[name]);
    let verb = tokens.first().map(String::as_str)?;
    if !READ_QUERY_VERBS.contains(&verb) || LOCAL_OPERATION_MUTATION_VERBS.contains(&verb) {
        return None;
    }
    let domain_tokens = tokens
        .iter()
        .skip(1)
        .filter(|token| {
            !READ_QUERY_VERBS.contains(&token.as_str())
                && !LOCAL_OPERATION_MUTATION_VERBS.contains(&token.as_str())
        })
        .cloned()
        .collect::<Vec<_>>();
    if domain_tokens.is_empty() {
        return None;
    }
    Some(OperationInfo {
        operation_family: Some(ServiceOperationFamily::ReadQuery),
        domain_tokens,
    })
}

pub(in crate::prewrite) fn is_local_operation_container_name(name: &str) -> bool {
    let tokens = unique_prewrite_tokens(&[name]);
    tokens
        .first()
        .is_some_and(|token| LOCAL_OPERATION_CONTAINER_START.contains(&token.as_str()))
        && tokens
            .iter()
            .any(|token| LOCAL_OPERATION_CONTAINER_DOMAIN.contains(&token.as_str()))
}

fn operation_family_for_verb(verb: &str) -> Option<ServiceOperationFamily> {
    match verb {
        "fetch" | "find" | "get" | "list" | "load" | "lookup" | "query" | "read" | "resolve"
        | "retrieve" | "search" => Some(ServiceOperationFamily::ReadQuery),
        "add" | "create" => Some(ServiceOperationFamily::MutationCreate),
        "delete" | "destroy" | "remove" => Some(ServiceOperationFamily::MutationDelete),
        "dispatch" | "emit" | "send" => Some(ServiceOperationFamily::MutationSend),
        "patch" | "set" | "update" => Some(ServiceOperationFamily::MutationUpdate),
        "save" | "upsert" | "write" => Some(ServiceOperationFamily::MutationSave),
        _ => None,
    }
}

fn normalize_domain_token(token: &str) -> Option<String> {
    if token.is_empty() {
        return None;
    }
    if token.len() > 3 && token.ends_with("ies") {
        return Some(format!("{}y", &token[..token.len() - 3]));
    }
    if token.len() > 3 && token.ends_with('s') && !token.ends_with("ss") && !token.ends_with("us") {
        return Some(token[..token.len() - 1].to_string());
    }
    Some(token.to_string())
}
