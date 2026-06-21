mod ambiguous;
mod regex;
mod strings;
mod visible;

pub(super) fn scanner_state_ambiguous_like_js(source: &str) -> bool {
    ambiguous::scanner_state_ambiguous_like_js(source)
}

pub(super) fn risk_visible_lines(source: &str) -> Vec<String> {
    visible::risk_visible_lines(source)
}
