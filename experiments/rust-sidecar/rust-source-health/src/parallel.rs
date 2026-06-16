use crate::protocol::DEFAULT_WORKER_STACK_BYTES;
use anyhow::{bail, Result};
use rayon::{ThreadPool, ThreadPoolBuilder};

#[derive(Debug, Clone, Copy)]
pub struct RuntimeConfig {
    pub thread_count: Option<usize>,
    pub worker_stack_bytes: usize,
}

pub fn build_pool(config: RuntimeConfig) -> Result<ThreadPool> {
    if matches!(config.thread_count, Some(0)) {
        bail!("runtime.threadCount must be greater than zero when provided");
    }
    if config.worker_stack_bytes < DEFAULT_WORKER_STACK_BYTES {
        bail!(
            "runtime.workerStackBytes must be at least {}",
            DEFAULT_WORKER_STACK_BYTES
        );
    }

    let mut builder = ThreadPoolBuilder::new().stack_size(config.worker_stack_bytes);
    if let Some(thread_count) = config.thread_count {
        builder = builder.num_threads(thread_count);
    }
    Ok(builder.build()?)
}
