use crate::protocol::AstOpaqueMuteReason;

pub(super) fn derive_items(body: &str) -> Option<Vec<&str>> {
    let inner = body.strip_prefix("derive(")?.strip_suffix(')')?;
    let items = inner
        .split(',')
        .map(|item| item.rsplit("::").next().unwrap_or(item).trim())
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>();
    (!items.is_empty()).then_some(items)
}

pub(super) fn derive_mute_reason(body: &str, items: &[&str]) -> Option<AstOpaqueMuteReason> {
    if items.iter().all(|item| is_builtin_derive(item)) {
        return Some(AstOpaqueMuteReason::BuiltinDeriveMacro);
    }
    if items
        .iter()
        .all(|item| is_builtin_derive(item) || is_known_data_derive(body, item))
    {
        return Some(AstOpaqueMuteReason::KnownDataDeriveMacro);
    }
    None
}

fn is_builtin_derive(name: &str) -> bool {
    matches!(
        name,
        "Clone" | "Copy" | "Debug" | "Default" | "Eq" | "Hash" | "Ord" | "PartialEq" | "PartialOrd"
    )
}

fn is_known_data_derive(body: &str, name: &str) -> bool {
    matches!(
        name,
        "Deserialize" | "ExperimentalApi" | "JsonSchema" | "Serialize" | "TS"
    ) || (name == "Message" && body.contains("prost::Message"))
}
