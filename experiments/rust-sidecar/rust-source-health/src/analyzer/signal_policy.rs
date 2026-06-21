use crate::locations::LineIndex;
use crate::protocol::{Signal, SignalKind, SignalMuteReason};
use crate::signals::{mute_signal, review_signal};

use super::attrs::{has_direct_cfg_test_attr, has_direct_test_attr};
use ra_ap_syntax::{ast, AstNode, SyntaxNode};

pub(super) fn contextual_review_signal(
    kind: SignalKind,
    line_index: &LineIndex,
    node: &SyntaxNode,
) -> Signal {
    let mut signal = review_signal(kind, line_index, node.text_range());
    if let Some(reason) = test_context_mute_reason(node) {
        mute_signal(&mut signal, reason);
    }
    signal
}

pub(super) fn test_context_mute_reason(node: &SyntaxNode) -> Option<SignalMuteReason> {
    for ancestor in node.ancestors() {
        if let Some(function) = ast::Fn::cast(ancestor.clone()) {
            if has_direct_test_attr(&function) {
                return Some(SignalMuteReason::TestAttribute);
            }
            if has_direct_cfg_test_attr(&function) {
                return Some(SignalMuteReason::CfgTest);
            }
        }
        if let Some(module) = ast::Module::cast(ancestor.clone()) {
            if has_direct_cfg_test_attr(&module) {
                return Some(SignalMuteReason::CfgTest);
            }
        }
        if let Some(impl_block) = ast::Impl::cast(ancestor) {
            if has_direct_cfg_test_attr(&impl_block) {
                return Some(SignalMuteReason::CfgTest);
            }
        }
    }
    None
}

pub(super) fn collect_method_call_signal(
    node: &SyntaxNode,
    line_index: &LineIndex,
    method: &str,
    signals: &mut Vec<Signal>,
) {
    match method {
        "unwrap" => signals.push(contextual_review_signal(
            SignalKind::UnwrapCall,
            line_index,
            node,
        )),
        "expect" => signals.push(contextual_review_signal(
            SignalKind::ExpectCall,
            line_index,
            node,
        )),
        "clone" => signals.push(contextual_review_signal(
            SignalKind::CloneCall,
            line_index,
            node,
        )),
        _ => {}
    }
}

pub(super) fn collect_macro_call_signal(
    node: &SyntaxNode,
    line_index: &LineIndex,
    name: &str,
    signals: &mut Vec<Signal>,
) {
    match name {
        "panic" => signals.push(contextual_review_signal(
            SignalKind::PanicMacro,
            line_index,
            node,
        )),
        "todo" => signals.push(contextual_review_signal(
            SignalKind::TodoMacro,
            line_index,
            node,
        )),
        "unimplemented" => signals.push(contextual_review_signal(
            SignalKind::UnimplementedMacro,
            line_index,
            node,
        )),
        _ => {}
    }
}
