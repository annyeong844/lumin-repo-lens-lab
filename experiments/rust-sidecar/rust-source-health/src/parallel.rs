use crate::protocol::{RuntimeRequest, DEFAULT_WORKER_STACK_BYTES};
use anyhow::{bail, Result};
use rayon::{ThreadPool, ThreadPoolBuilder};

#[derive(Debug, Clone, Copy)]
pub struct RuntimeConfig {
    pub thread_count: Option<usize>,
    pub worker_stack_bytes: usize,
}

impl TryFrom<RuntimeRequest> for RuntimeConfig {
    type Error = anyhow::Error;

    fn try_from(request: RuntimeRequest) -> Result<Self> {
        if matches!(request.thread_count, Some(0)) {
            bail!("runtime.threadCount must be greater than zero when provided");
        }
        if request.worker_stack_bytes < DEFAULT_WORKER_STACK_BYTES {
            bail!(
                "runtime.workerStackBytes must be at least {}",
                DEFAULT_WORKER_STACK_BYTES
            );
        }
        Ok(Self {
            thread_count: request.thread_count,
            worker_stack_bytes: request.worker_stack_bytes,
        })
    }
}

pub fn build_pool(config: RuntimeConfig) -> Result<ThreadPool> {
    let mut builder = ThreadPoolBuilder::new().stack_size(config.worker_stack_bytes);
    if let Some(thread_count) = config.thread_count {
        builder = builder.num_threads(thread_count);
    }
    Ok(builder.build()?)
}
