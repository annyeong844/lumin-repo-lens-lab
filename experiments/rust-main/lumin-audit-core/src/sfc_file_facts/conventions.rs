use super::protocol::{
    AstroClientDirectiveConvention, SfcConventionBinding, SfcConventionCommon, SfcFileConvention,
    SvelteActionDirectiveConvention, SvelteStoreSubscriptionConvention,
    VueMacroRegistrationConvention, VueOptionsRegistrationConvention,
};
use super::script::{ComponentBinding, ComponentBindings};

const CONVENTION_CONFIDENCE: &str = "framework-convention-observed";
const SVELTE_STORE_FACTORIES: &[&str] = &["writable", "readable", "derived"];

pub(super) fn action_binding<'a>(
    action_name: &str,
    bindings: &'a ComponentBindings,
) -> Option<&'a ComponentBinding> {
    bindings
        .local_actions
        .get(action_name)
        .or_else(|| bindings.exposed_names.get(action_name))
        .or_else(|| bindings.imports.get(action_name))
}

pub(super) fn store_binding<'a>(
    store_name: &str,
    bindings: &'a ComponentBindings,
) -> Option<&'a ComponentBinding> {
    let binding = bindings
        .local_stores
        .get(store_name)
        .or_else(|| bindings.exposed_names.get(store_name))
        .or_else(|| bindings.imports.get(store_name))?;
    let imported_name = binding
        .imported_name
        .as_deref()
        .unwrap_or(binding.binding_name.as_str());
    if binding.binding_source == "svelte/store" && SVELTE_STORE_FACTORIES.contains(&imported_name) {
        return None;
    }
    Some(binding)
}

pub(super) fn astro_client_directive(
    file_path: &str,
    tag_name: &str,
    normalized_tag_name: &str,
    directive_name: &str,
    binding: &ComponentBinding,
    line: usize,
    block_kind: &str,
) -> SfcFileConvention {
    SfcFileConvention::AstroClientDirective(AstroClientDirectiveConvention {
        common: common(
            "astro",
            "client-directive",
            file_path,
            "sfc-framework-astro-client-directive",
            line,
            block_kind,
        ),
        tag_name: tag_name.to_string(),
        normalized_tag_name: normalized_tag_name.to_string(),
        directive_name: directive_name.to_string(),
        binding: convention_binding(binding),
    })
}

pub(super) fn svelte_action_directive(
    file_path: &str,
    tag_name: &str,
    directive_name: &str,
    action_name: &str,
    binding: &ComponentBinding,
    line: usize,
    block_kind: &str,
) -> SfcFileConvention {
    SfcFileConvention::SvelteActionDirective(SvelteActionDirectiveConvention {
        common: common(
            "svelte",
            "action-directive",
            file_path,
            "sfc-framework-svelte-action-directive",
            line,
            block_kind,
        ),
        tag_name: tag_name.to_string(),
        directive_name: directive_name.to_string(),
        action_name: action_name.to_string(),
        binding: convention_binding(binding),
    })
}

pub(super) fn svelte_store_subscription(
    file_path: &str,
    store_name: &str,
    binding: &ComponentBinding,
    line: usize,
    block_kind: &str,
) -> SfcFileConvention {
    SfcFileConvention::SvelteStoreSubscription(SvelteStoreSubscriptionConvention {
        common: common(
            "svelte",
            "store-auto-subscription",
            file_path,
            "sfc-framework-svelte-store-subscription",
            line,
            block_kind,
        ),
        subscription_name: format!("${store_name}"),
        store_name: store_name.to_string(),
        binding: convention_binding(binding),
    })
}

pub(super) fn vue_macro_registration(
    file_path: &str,
    component_name: &str,
    binding: &ComponentBinding,
    line: usize,
    block_kind: &str,
) -> SfcFileConvention {
    SfcFileConvention::VueMacroRegistration(VueMacroRegistrationConvention {
        common: common(
            "vue",
            "macro-registration",
            file_path,
            "sfc-framework-vue-macro-registration",
            line,
            block_kind,
        ),
        macro_name: "defineOptions",
        component_name: component_name.to_string(),
        normalized_tag_names: normalized_component_names(component_name),
        binding: convention_binding(binding),
    })
}

pub(super) fn vue_options_registration(
    file_path: &str,
    component_name: &str,
    binding: &ComponentBinding,
    line: usize,
    block_kind: &str,
) -> SfcFileConvention {
    SfcFileConvention::VueOptionsRegistration(VueOptionsRegistrationConvention {
        common: common(
            "vue",
            "options-registration",
            file_path,
            "sfc-framework-vue-options-registration",
            line,
            block_kind,
        ),
        option_name: "components",
        component_name: component_name.to_string(),
        normalized_tag_names: normalized_component_names(component_name),
        binding: convention_binding(binding),
    })
}

fn common(
    framework: &'static str,
    convention_kind: &'static str,
    file_path: &str,
    reason: &'static str,
    line: usize,
    block_kind: &str,
) -> SfcConventionCommon {
    SfcConventionCommon {
        framework,
        convention_kind,
        consumer_file: file_path.to_string(),
        source: reason,
        confidence: CONVENTION_CONFIDENCE,
        eligible_for_fan_in: false,
        eligible_for_safe_fix: false,
        status: "muted",
        reason,
        line,
        sfc_block_kind: block_kind.to_string(),
    }
}

fn convention_binding(binding: &ComponentBinding) -> SfcConventionBinding {
    SfcConventionBinding {
        binding_name: binding.binding_name.clone(),
        binding_source: binding.binding_source.clone(),
        from_spec: binding.binding_source.clone(),
        binding_kind: binding.binding_kind.clone(),
        imported_name: binding.imported_name.clone(),
    }
}

fn normalized_component_names(component_name: &str) -> Vec<String> {
    let mut names = vec![component_name.to_string()];
    if let Some(pascal) = pascal_from_kebab(component_name) {
        if !names.contains(&pascal) {
            names.push(pascal);
        }
    }
    if let Some(kebab) = kebab_from_pascal(component_name) {
        if !names.contains(&kebab) {
            names.push(kebab);
        }
    }
    names
}

fn pascal_from_kebab(value: &str) -> Option<String> {
    let mut parts = value.split('-');
    let first = parts.next()?;
    if first.is_empty()
        || !first.starts_with(|character: char| character.is_ascii_lowercase())
        || !first
            .chars()
            .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit())
    {
        return None;
    }
    let rest: Vec<&str> = parts.collect();
    if rest.is_empty()
        || rest.iter().any(|part| {
            part.is_empty()
                || !part
                    .chars()
                    .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit())
        })
    {
        return None;
    }
    let mut out = capitalize(first);
    for part in rest {
        out.push_str(&capitalize(part));
    }
    Some(out)
}

fn kebab_from_pascal(value: &str) -> Option<String> {
    let mut characters = value.chars();
    let first = characters.next()?;
    if !first.is_ascii_uppercase()
        || !characters
            .clone()
            .all(|character| character.is_ascii_alphanumeric())
    {
        return None;
    }
    let chars: Vec<char> = value.chars().collect();
    let mut out = String::new();
    for (index, character) in chars.iter().enumerate() {
        if character.is_ascii_uppercase()
            && index > 0
            && (chars[index - 1].is_ascii_lowercase()
                || chars[index - 1].is_ascii_digit()
                || chars.get(index + 1).is_some_and(char::is_ascii_lowercase))
        {
            out.push('-');
        }
        out.push(character.to_ascii_lowercase());
    }
    Some(out)
}

fn capitalize(value: &str) -> String {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return String::new();
    };
    let mut out = first.to_ascii_uppercase().to_string();
    out.extend(characters);
    out
}
