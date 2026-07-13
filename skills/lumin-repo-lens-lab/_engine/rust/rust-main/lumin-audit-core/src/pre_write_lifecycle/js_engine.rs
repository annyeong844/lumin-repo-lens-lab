use anyhow::Result;

use super::child::ChildStdio;
use super::protocol::{JsPreWriteLifecycleRequest, PreWriteLifecycleResult};

pub(super) fn execute(
    request: JsPreWriteLifecycleRequest,
    stdio: ChildStdio,
) -> Result<PreWriteLifecycleResult> {
    super::js_native::execute(request, stdio)
}
