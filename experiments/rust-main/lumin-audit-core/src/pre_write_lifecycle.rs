mod advisory;
mod child;
mod js_engine;
mod js_native;
mod protocol;
mod rust_engine;

use anyhow::Result;

use child::ChildStdio;

pub use protocol::{
    AnalyzerInvocationBlock, AnalyzerInvocationRequest, JsPreWriteLifecycleRequest, PreWriteBlock,
    PreWriteFailureKind, PreWriteLifecycleRequest, PreWriteLifecycleResult,
    RustPreWriteLifecycleRequest, JS_PRE_WRITE_LIFECYCLE_REQUEST_SCHEMA_VERSION,
    PRE_WRITE_LIFECYCLE_REQUEST_SCHEMA_VERSION, PRE_WRITE_LIFECYCLE_RESULT_SCHEMA_VERSION,
};

pub fn execute_pre_write_lifecycle(
    request: PreWriteLifecycleRequest,
) -> Result<PreWriteLifecycleResult> {
    rust_engine::execute(request, ChildStdio::Capture)
}

pub fn execute_rust_pre_write_lifecycle(
    request: RustPreWriteLifecycleRequest,
) -> Result<PreWriteLifecycleResult> {
    execute_pre_write_lifecycle(request)
}

pub fn execute_pre_write_lifecycle_streaming(
    request: PreWriteLifecycleRequest,
) -> Result<PreWriteLifecycleResult> {
    rust_engine::execute(request, ChildStdio::Inherit)
}

pub fn execute_rust_pre_write_lifecycle_streaming(
    request: RustPreWriteLifecycleRequest,
) -> Result<PreWriteLifecycleResult> {
    execute_pre_write_lifecycle_streaming(request)
}

pub fn execute_js_pre_write_lifecycle(
    request: JsPreWriteLifecycleRequest,
) -> Result<PreWriteLifecycleResult> {
    js_engine::execute(request, ChildStdio::Capture)
}

pub fn execute_js_pre_write_lifecycle_streaming(
    request: JsPreWriteLifecycleRequest,
) -> Result<PreWriteLifecycleResult> {
    js_engine::execute(request, ChildStdio::Inherit)
}
