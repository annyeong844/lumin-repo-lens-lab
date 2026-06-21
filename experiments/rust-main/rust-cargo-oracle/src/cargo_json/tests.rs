use super::{CargoJsonReason, CargoJsonStream};
use crate::protocol::{CargoTargetKind, RustcSuggestionApplicability};
use anyhow::{Context, Result};

#[test]
fn compiler_message_uses_nested_target_when_top_level_target_is_absent() -> Result<()> {
    let mut stream = CargoJsonStream::empty();
    stream.push_json_line(
        r#"{"reason":"compiler-message","package_id":"path+file:///app#0.1.0","message":{"target":{"name":"app","kind":["bin"]},"level":"error","code":null,"spans":[]}}"#,
    )?;

    let event = stream
        .as_messages()
        .compiler_target_events()
        .next()
        .context("compiler event")?;
    let target = event.target().context("target from nested message")?;

    assert_eq!(event.reason(), Some(CargoJsonReason::CompilerMessage));
    assert_eq!(target.name(), "app");
    assert_eq!(target.kinds(), vec![CargoTargetKind::Bin]);
    Ok(())
}

#[test]
fn unknown_cargo_json_reason_is_ignored_without_rejecting_the_line() -> Result<()> {
    let mut stream = CargoJsonStream::empty();
    stream.push_json_line(r#"{"reason":"future-cargo-event","payload":{"ok":true}}"#)?;

    assert_eq!(stream.len(), 1);
    assert!(stream.as_messages().compiler_messages().next().is_none());
    assert!(stream
        .as_messages()
        .compiler_target_events()
        .next()
        .is_none());
    Ok(())
}

#[test]
fn compiler_message_keeps_valid_spans_when_one_span_has_future_shape() -> Result<()> {
    let mut stream = CargoJsonStream::empty();
    stream.push_json_line(
        r#"{"reason":"compiler-message","package_id":"path+file:///app#0.1.0","message":{"level":"error","message":"mismatched types","code":{"code":"E0308"},"spans":["future-span-shape",{"file_name":"src/lib.rs","is_primary":true,"line_start":2,"line_end":2,"column_start":5,"column_end":12,"expansion":null}]}}"#,
    )?;

    let event = stream
        .as_messages()
        .compiler_messages()
        .next()
        .context("compiler message")?;
    let diagnostic = event.rustc_diagnostic().context("rustc diagnostic")?;
    let span = diagnostic
        .spans()
        .first()
        .context("valid diagnostic span")?;

    assert_eq!(diagnostic.spans().len(), 1);
    assert_eq!(diagnostic.code_text(), Some("E0308"));
    assert_eq!(span.file_name(), Some("src/lib.rs"));
    assert_eq!(span.line_start(), Some(2));
    assert_eq!(span.column_start(), Some(5));
    Ok(())
}

#[test]
fn compiler_message_does_not_treat_future_applicability_as_machine_applicable() -> Result<()> {
    let mut stream = CargoJsonStream::empty();
    stream.push_json_line(
        r#"{"reason":"compiler-message","package_id":"path+file:///app#0.1.0","message":{"level":"warning","message":"variable does not need to be mutable","code":{"code":"unused_mut"},"spans":[{"file_name":"src/lib.rs","is_primary":true,"line_start":2,"line_end":2,"column_start":9,"column_end":13,"suggestion_applicability":"FutureDefinitelySafe","suggested_replacement":"","expansion":null}]}}"#,
    )?;

    let event = stream
        .as_messages()
        .compiler_messages()
        .next()
        .context("compiler message")?;
    let diagnostic = event.rustc_diagnostic().context("rustc diagnostic")?;
    let candidate = diagnostic
        .suggestion_candidate_spans()
        .into_iter()
        .next()
        .context("suggestion candidate span")?;

    assert_eq!(candidate.suggested_replacement(), Some(""));
    assert_ne!(
        candidate.suggestion_applicability(),
        Some(RustcSuggestionApplicability::MachineApplicable)
    );
    assert_eq!(candidate.suggestion_applicability(), None);
    Ok(())
}

#[test]
fn compiler_message_with_future_message_shape_keeps_top_level_target() -> Result<()> {
    let mut stream = CargoJsonStream::empty();
    stream.push_json_line(
        r#"{"reason":"compiler-message","package_id":"path+file:///app#0.1.0","target":{"name":"app","kind":["bin"]},"message":"future-rustc-shape"}"#,
    )?;

    assert_eq!(stream.len(), 1);

    let target_event = stream
        .as_messages()
        .compiler_target_events()
        .next()
        .context("compiler target event")?;
    let target = target_event.target().context("top-level target")?;

    assert_eq!(target.name(), "app");
    assert_eq!(target.kinds(), vec![CargoTargetKind::Bin]);

    let diagnostic_event = stream
        .as_messages()
        .compiler_messages()
        .next()
        .context("compiler message event")?;
    assert!(diagnostic_event.rustc_diagnostic().is_none());
    Ok(())
}

#[test]
fn compiler_message_preserves_future_target_kind_without_string_drift() -> Result<()> {
    let mut stream = CargoJsonStream::empty();
    stream.push_json_line(
        r#"{"reason":"compiler-message","package_id":"path+file:///app#0.1.0","target":{"name":"app","kind":["future-target-kind"]},"message":"future-rustc-shape"}"#,
    )?;

    let target_event = stream
        .as_messages()
        .compiler_target_events()
        .next()
        .context("compiler target event")?;
    let target = target_event.target().context("top-level target")?;

    assert_eq!(
        target.kinds(),
        vec![CargoTargetKind::Unknown("future-target-kind".to_string())]
    );
    Ok(())
}
