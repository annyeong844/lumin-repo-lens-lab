use crate::orchestration_events::MemorySnapshot;

pub(super) fn memory_snapshot() -> MemorySnapshot {
    MemorySnapshot {
        rss_bytes: current_process_rss_bytes(),
        heap_total_bytes: 0,
        heap_used_bytes: 0,
        external_bytes: 0,
        array_buffers_bytes: 0,
    }
}

#[cfg(windows)]
fn current_process_rss_bytes() -> i64 {
    windows_working_set_bytes().unwrap_or(0)
}

#[cfg(windows)]
fn windows_working_set_bytes() -> Option<i64> {
    #[repr(C)]
    struct ProcessMemoryCounters {
        cb: u32,
        page_fault_count: u32,
        peak_working_set_size: usize,
        working_set_size: usize,
        quota_peak_paged_pool_usage: usize,
        quota_paged_pool_usage: usize,
        quota_peak_non_paged_pool_usage: usize,
        quota_non_paged_pool_usage: usize,
        pagefile_usage: usize,
        peak_pagefile_usage: usize,
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GetCurrentProcess() -> *mut std::ffi::c_void;
    }

    #[link(name = "psapi")]
    extern "system" {
        fn GetProcessMemoryInfo(
            process: *mut std::ffi::c_void,
            counters: *mut ProcessMemoryCounters,
            size: u32,
        ) -> i32;
    }

    let mut counters = ProcessMemoryCounters {
        cb: std::mem::size_of::<ProcessMemoryCounters>() as u32,
        page_fault_count: 0,
        peak_working_set_size: 0,
        working_set_size: 0,
        quota_peak_paged_pool_usage: 0,
        quota_paged_pool_usage: 0,
        quota_peak_non_paged_pool_usage: 0,
        quota_non_paged_pool_usage: 0,
        pagefile_usage: 0,
        peak_pagefile_usage: 0,
    };
    // SAFETY: `counters` is fully initialized, `cb` is its exact ABI size, and
    // the current-process pseudo handle remains valid for the duration of the call.
    let ok = unsafe { GetProcessMemoryInfo(GetCurrentProcess(), &mut counters, counters.cb) };
    (ok != 0).then_some(counters.working_set_size as i64)
}

#[cfg(unix)]
fn current_process_rss_bytes() -> i64 {
    linux_proc_status_rss_bytes().unwrap_or(0)
}

#[cfg(unix)]
fn linux_proc_status_rss_bytes() -> Option<i64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    for line in status.lines() {
        let Some(rest) = line.strip_prefix("VmRSS:") else {
            continue;
        };
        let kb = rest
            .split_whitespace()
            .next()
            .and_then(|value| value.parse::<i64>().ok())?;
        return Some(kb.saturating_mul(1024));
    }
    None
}

#[cfg(not(any(windows, unix)))]
fn current_process_rss_bytes() -> i64 {
    0
}

pub(super) fn memory_delta(before: &MemorySnapshot, after: &MemorySnapshot) -> MemorySnapshot {
    MemorySnapshot {
        rss_bytes: after.rss_bytes - before.rss_bytes,
        heap_total_bytes: after.heap_total_bytes - before.heap_total_bytes,
        heap_used_bytes: after.heap_used_bytes - before.heap_used_bytes,
        external_bytes: after.external_bytes - before.external_bytes,
        array_buffers_bytes: after.array_buffers_bytes - before.array_buffers_bytes,
    }
}
