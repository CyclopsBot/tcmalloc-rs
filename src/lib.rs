//! Rust global allocator and high-level telemetry controls for Google
//! `TCMalloc`.
//!
//! The allocator path calls `TCMalloc`'s exported C ABI directly. Higher-level
//! statistics, profiling, `GWP-ASan`, and tuning operations use a CXX bridge
//! over `tcmalloc::MallocExtension`.
//!
//! This crate is intended to be installed by a final binary. Libraries should
//! expose allocator-aware APIs without setting Rust's process-global allocator
//! for their dependents.
//!
//! # Basic Usage
//!
//! ```rust,ignore
//! #[global_allocator]
//! static ALLOC: tcmalloc::TcMalloc = tcmalloc::TcMalloc;
//! ```
//!
//! # Process-Global State
//!
//! `TCMalloc` runtime knobs are process-global. If tests or admin endpoints
//! change sampling intervals, cache limits, release rates, or memory limits,
//! serialize those changes and restore old values where possible.
//!
//! # Typed Stats
//!
//! ```rust,ignore
//! let stats = tcmalloc::malloc_stats();
//! let in_use = stats.properties.generic.current_allocated_bytes;
//! ```
//!
//! Prefer `malloc_stats().properties` for metrics and automation. Use
//! [`stats`] when you need the upstream human-readable report for diagnostics.
//!
//! # Profiling
//!
//! ```rust,ignore
//! let session = tcmalloc::start_allocation_profiling();
//! // Run workload.
//! let profile = session.stop()?;
//! for record in profile.otel_records() {
//!     // Map record.name, record.value, and record.attributes into your OTel
//!     // metrics exporter.
//! }
//! # Ok::<(), tcmalloc::TcmallocError>(())
//! ```
//!
//! Profile samples are sampled and weighted. They are suitable for telemetry and
//! comparisons, but they are not a literal list of every allocation.
//!
//! # Runtime Tuning
//!
//! Use [`TcmallocConfig`] to apply several optional knobs together. `None`
//! leaves the existing upstream value unchanged.

#![allow(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

use core::{
    alloc::{GlobalAlloc, Layout},
    cmp,
    ffi::c_void,
    ptr,
};

#[path = "profiling.rs"]
mod profiling_impl;
#[path = "stats.rs"]
mod stats_impl;
#[path = "tuning.rs"]
mod tuning_impl;

pub use profiling_impl::{
    AccessPattern, AllocationKind, GuardedStatus, OtelAttribute, OtelProfileRecord, ProfileSample,
    ProfileType, ProfilingSession, TcmallocProfile, profile_snapshot, start_allocation_profiling,
    start_lifetime_profiling,
};
pub use stats_impl::{
    AllocatorProperties, AllocatorProperty, GenericProperties, HookState, MallocStats, MemoryStat,
    NamedValue, TcmallocProperties, malloc_stats, numeric_property, properties,
};
pub use tuning_impl::{
    GuardedSamplingConfig, MemoryLimitKind, TcmallocConfig, TuningSnapshot,
    background_process_actions_enabled, background_process_sleep_interval_nanos,
    background_release_rate, configure_guarded_sampling, estimated_allocated_size,
    guarded_sampling_interval_bytes, hard_memory_limit, mark_thread_busy, mark_thread_idle,
    max_per_cpu_cache_size, max_total_thread_cache_bytes, needs_process_background_actions,
    per_cpu_caches_active, process_background_actions, profile_sampling_interval_bytes,
    release_cpu_memory, release_memory_to_system, run_background_actions_forever,
    set_background_process_actions_enabled, set_background_process_sleep_interval_nanos,
    set_background_release_rate, set_guarded_sampling_interval_bytes, set_hard_memory_limit,
    set_max_per_cpu_cache_size, set_max_total_thread_cache_bytes, set_memory_limit,
    set_profile_sampling_interval_bytes, set_skip_subrelease_long_interval_nanos,
    set_skip_subrelease_short_interval_nanos, set_soft_memory_limit,
    skip_subrelease_long_interval_nanos, skip_subrelease_short_interval_nanos, soft_memory_limit,
    tuning_snapshot,
};

#[cxx::bridge(namespace = "tcmalloc_rs")]
pub(crate) mod ffi {
    unsafe extern "C++" {
        include!("tcmalloc_rs/tcmalloc_bridge.h");

        type AllocationProfileSession;

        fn stats() -> String;
        fn properties() -> String;
        fn numeric_property(property: &str) -> i64;

        fn profile_snapshot(profile_type: u8) -> String;
        fn start_allocation_profiling() -> UniquePtr<AllocationProfileSession>;
        fn start_lifetime_profiling() -> UniquePtr<AllocationProfileSession>;
        fn stop(self: Pin<&mut AllocationProfileSession>) -> String;

        fn activate_guarded_sampling();
        fn release_memory_to_system(bytes: usize);
        fn profile_sampling_interval() -> i64;
        fn set_profile_sampling_interval(interval: i64);
        fn guarded_sampling_interval() -> i64;
        fn set_guarded_sampling_interval(interval: i64);

        fn mark_thread_idle();
        fn mark_thread_busy();
        fn release_cpu_memory(cpu: i32) -> usize;
        fn per_cpu_caches_active() -> bool;
        fn max_per_cpu_cache_size() -> i32;
        fn set_max_per_cpu_cache_size(value: i32);
        fn max_total_thread_cache_bytes() -> i64;
        fn set_max_total_thread_cache_bytes(value: i64);
        fn background_process_actions_enabled() -> bool;
        fn set_background_process_actions_enabled(value: bool);
        fn background_process_sleep_interval_nanos() -> i64;
        fn set_background_process_sleep_interval_nanos(value: i64);
        fn skip_subrelease_short_interval_nanos() -> i64;
        fn set_skip_subrelease_short_interval_nanos(value: i64);
        fn skip_subrelease_long_interval_nanos() -> i64;
        fn set_skip_subrelease_long_interval_nanos(value: i64);
        fn estimated_allocated_size(size: usize) -> usize;
        fn memory_limit(hard: bool) -> usize;
        fn set_memory_limit(limit: usize, hard: bool);
        fn process_background_actions();
        fn needs_process_background_actions() -> bool;
        fn background_release_rate() -> usize;
        fn set_background_release_rate(rate: usize);
    }
}

#[allow(non_snake_case)]
mod sys {
    use core::ffi::c_void;

    unsafe extern "C" {
        pub(super) fn TCMallocInternalMalloc(size: usize) -> *mut c_void;
        pub(super) fn TCMallocInternalCalloc(n: usize, size: usize) -> *mut c_void;
        pub(super) fn TCMallocInternalMemalign(align: usize, size: usize) -> *mut c_void;
        pub(super) fn TCMallocInternalRealloc(ptr: *mut c_void, size: usize) -> *mut c_void;
        pub(super) fn TCMallocInternalFree(ptr: *mut c_void);
        pub(super) fn TCMallocInternalMallocStats();
    }
}

/// Error returned by typed tcmalloc wrappers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TcmallocError {
    /// A value cannot fit into the signed integer shape required by tcmalloc.
    ValueOutOfRange { name: &'static str, value: u64 },
    /// A profile kind was parsed from tcmalloc but cannot be requested.
    UnsupportedProfileType(u8),
    /// `TCMalloc` returned profile data that the Rust parser could not decode.
    InvalidProfileData(String),
}

impl core::fmt::Display for TcmallocError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ValueOutOfRange { name, value } => {
                write!(f, "{name} value {value} does not fit the tcmalloc API")
            }
            Self::UnsupportedProfileType(value) => {
                write!(f, "unsupported tcmalloc profile type: {value}")
            }
            Self::InvalidProfileData(reason) => {
                write!(f, "invalid tcmalloc profile data: {reason}")
            }
        }
    }
}

impl std::error::Error for TcmallocError {}

/// Global allocator backed by Google tcmalloc.
///
/// Install this with Rust's `#[global_allocator]` attribute in a binary crate.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Copy, Default)]
pub struct TcMalloc;

// SAFETY: Every `GlobalAlloc` method delegates to tcmalloc's exported allocator
// ABI. Pointers returned by allocation methods are released through the same ABI,
// and over-aligned reallocations are handled by allocating, copying, and freeing.
unsafe impl GlobalAlloc for TcMalloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        allocate(layout, false)
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        allocate(layout, true)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        // SAFETY: The `GlobalAlloc` contract requires callers to pass a pointer
        // allocated by this allocator and not yet deallocated.
        unsafe { sys::TCMallocInternalFree(ptr.cast::<c_void>()) };
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if requires_explicit_alignment(layout.align()) {
            return realloc_overaligned(ptr, layout, new_size);
        }

        // SAFETY: The `GlobalAlloc` contract requires `ptr` to have been
        // allocated by this allocator with `layout`; tcmalloc accepts the new
        // allocation size in bytes and returns either null or owned memory.
        unsafe { sys::TCMallocInternalRealloc(ptr.cast::<c_void>(), new_size).cast::<u8>() }
    }
}

/// Returns a human-readable tcmalloc statistics report.
#[must_use]
pub fn stats() -> String {
    ffi::stats()
}

/// Returns a human-readable tcmalloc statistics report.
///
/// This preserves the upstream crate's API name even though it returns the
/// report instead of writing it directly.
#[must_use]
pub fn print_stats() -> String {
    stats()
}

/// Writes tcmalloc's short malloc statistics summary to stderr.
pub fn print_stats_summary() {
    // SAFETY: This tcmalloc function has no caller preconditions and only writes
    // allocator statistics to stderr.
    unsafe { sys::TCMallocInternalMallocStats() };
}

/// Enables tcmalloc guarded sampling.
///
/// Prefer [`configure_guarded_sampling`] for new code so the activation and
/// sampling interval are applied together.
pub fn activate_guarded_sampling() {
    ffi::activate_guarded_sampling();
}

/// Returns the raw allocation profiling sampling interval.
///
/// Values below zero mean tcmalloc could not report the interval. New code
/// should use [`profile_sampling_interval_bytes`] for a typed `Option<u64>`.
#[must_use]
pub fn profile_sampling_interval() -> i64 {
    ffi::profile_sampling_interval()
}

/// Sets the raw allocation profiling sampling interval.
///
/// New code should use [`set_profile_sampling_interval_bytes`] to avoid passing
/// negative values accidentally.
pub fn set_profile_sampling_interval(interval: i64) {
    ffi::set_profile_sampling_interval(interval);
}

/// Returns the raw guarded sampling interval.
///
/// Values below zero mean tcmalloc could not report the interval. New code
/// should use [`guarded_sampling_interval_bytes`] for a typed `Option<u64>`.
#[must_use]
pub fn guarded_sampling_interval() -> i64 {
    ffi::guarded_sampling_interval()
}

/// Sets the raw guarded sampling interval.
///
/// New code should use [`set_guarded_sampling_interval_bytes`] to avoid passing
/// negative values accidentally.
pub fn set_guarded_sampling_interval(interval: i64) {
    ffi::set_guarded_sampling_interval(interval);
}

pub(crate) fn checked_i64(name: &'static str, value: u64) -> Result<i64, TcmallocError> {
    i64::try_from(value).map_err(|_| TcmallocError::ValueOutOfRange { name, value })
}

pub(crate) fn checked_i32(name: &'static str, value: u32) -> Result<i32, TcmallocError> {
    i32::try_from(value).map_err(|_| TcmallocError::ValueOutOfRange {
        name,
        value: u64::from(value),
    })
}

pub(crate) fn non_negative_i64(value: i64) -> Option<u64> {
    u64::try_from(value).ok()
}

pub(crate) fn non_negative_i32(value: i32) -> Option<u32> {
    u32::try_from(value).ok()
}

fn allocate(layout: Layout, zeroed: bool) -> *mut u8 {
    if requires_explicit_alignment(layout.align()) {
        // SAFETY: `Layout` guarantees power-of-two alignment and a byte size.
        // tcmalloc returns either null or an allocation that can be freed by
        // `TCMallocInternalFree`.
        let ptr = unsafe { sys::TCMallocInternalMemalign(layout.align(), layout.size()) };
        if zeroed && !ptr.is_null() {
            // SAFETY: The pointer is non-null and owned by this allocator. The
            // allocation is at least `layout.size()` bytes.
            unsafe { ptr::write_bytes(ptr.cast::<u8>(), 0, layout.size()) };
        }
        return ptr.cast::<u8>();
    }

    if zeroed {
        // SAFETY: `1 * layout.size()` cannot overflow. tcmalloc returns either
        // null or a zeroed allocation that can be freed by this allocator.
        unsafe { sys::TCMallocInternalCalloc(1, layout.size()).cast::<u8>() }
    } else {
        // SAFETY: tcmalloc returns either null or an allocation of at least the
        // requested size that can be freed by this allocator.
        unsafe { sys::TCMallocInternalMalloc(layout.size()).cast::<u8>() }
    }
}

fn realloc_overaligned(ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
    let Ok(new_layout) = Layout::from_size_align(new_size, layout.align()) else {
        return ptr::null_mut();
    };

    let new_ptr = allocate(new_layout, false);
    if new_ptr.is_null() {
        return new_ptr;
    }

    // SAFETY: The `GlobalAlloc` contract requires `ptr` to be valid for
    // `layout.size()` bytes. `new_ptr` is a fresh allocation valid for
    // `new_size` bytes, so the regions do not overlap.
    unsafe {
        ptr::copy_nonoverlapping(ptr, new_ptr, cmp::min(layout.size(), new_size));
        sys::TCMallocInternalFree(ptr.cast::<c_void>());
    }

    new_ptr
}

const fn requires_explicit_alignment(align: usize) -> bool {
    align > core::mem::align_of::<usize>()
}

#[cfg(test)]
mod tests {
    use core::alloc::{GlobalAlloc, Layout};
    use std::sync::{Mutex, MutexGuard, OnceLock};

    use super::{
        GuardedSamplingConfig, ProfileType, TcMalloc, configure_guarded_sampling, malloc_stats,
        profile_snapshot, properties, set_guarded_sampling_interval_bytes,
        set_profile_sampling_interval_bytes, start_allocation_profiling, stats, tuning_snapshot,
    };

    fn global_tcmalloc_lock() -> MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    #[test]
    fn allocator_round_trip() {
        let layout = Layout::from_size_align(32, core::mem::align_of::<usize>()).unwrap();

        // SAFETY: The layout is non-zero and valid.
        let ptr = unsafe { TcMalloc.alloc(layout) };
        assert!(!ptr.is_null());

        // SAFETY: `ptr` is valid for `layout.size()` bytes and is deallocated
        // exactly once with the same layout.
        unsafe {
            ptr.write_bytes(0xAB, layout.size());
            TcMalloc.dealloc(ptr, layout);
        }
    }

    #[test]
    fn calloc_returns_zeroed_memory() {
        let layout = Layout::from_size_align(16, core::mem::align_of::<usize>()).unwrap();

        // SAFETY: The layout is non-zero and valid.
        let ptr = unsafe { TcMalloc.alloc_zeroed(layout) };
        assert!(!ptr.is_null());

        // SAFETY: `ptr` is valid for `layout.size()` bytes and is deallocated
        // exactly once with the same layout.
        unsafe {
            let bytes = core::slice::from_raw_parts(ptr, layout.size());
            assert!(bytes.iter().all(|byte| *byte == 0));
            TcMalloc.dealloc(ptr, layout);
        }
    }

    #[test]
    fn realloc_preserves_existing_bytes() {
        let layout = Layout::from_size_align(8, core::mem::align_of::<usize>()).unwrap();

        // SAFETY: The layout is non-zero and valid.
        let ptr = unsafe { TcMalloc.alloc(layout) };
        assert!(!ptr.is_null());

        // SAFETY: `ptr` is valid for the old layout. The returned pointer is
        // checked before use and then deallocated once with the new layout.
        unsafe {
            ptr.write(42);
            let new_ptr = TcMalloc.realloc(ptr, layout, 16);
            assert!(!new_ptr.is_null());
            assert_eq!(new_ptr.read(), 42);
            TcMalloc.dealloc(
                new_ptr,
                Layout::from_size_align(16, layout.align()).unwrap(),
            );
        }
    }

    #[test]
    fn allocator_honors_overaligned_layouts() {
        let layout = Layout::from_size_align(64, 64).unwrap();

        // SAFETY: The layout is non-zero and valid.
        let ptr = unsafe { TcMalloc.alloc(layout) };
        assert!(!ptr.is_null());
        assert_eq!(ptr.addr() % layout.align(), 0);

        // SAFETY: `ptr` was allocated with `layout` and is deallocated once.
        unsafe { TcMalloc.dealloc(ptr, layout) };
    }

    #[test]
    fn stats_api_is_callable() {
        let report = stats();
        assert!(!report.is_empty());
    }

    #[test]
    fn typed_stats_are_callable() {
        let stats = malloc_stats();
        assert!(!stats.raw.is_empty());
        assert!(!stats.properties.raw.is_empty());
    }

    #[test]
    fn properties_api_is_callable() {
        let properties = properties();
        assert!(!properties.raw.is_empty());
        assert!(
            properties.generic.current_allocated_bytes.is_some()
                || properties.generic.bytes_in_use_by_app.is_some()
        );
    }

    #[test]
    fn tuning_snapshot_is_callable() {
        let _lock = global_tcmalloc_lock();
        let snapshot = tuning_snapshot();
        assert!(snapshot.max_per_cpu_cache_size.is_some());
    }

    #[test]
    fn sampling_controls_round_trip() {
        let _lock = global_tcmalloc_lock();
        let original_profile = super::profile_sampling_interval_bytes();
        let original_guarded = super::guarded_sampling_interval_bytes();

        set_profile_sampling_interval_bytes(1_048_576).unwrap();
        set_guarded_sampling_interval_bytes(16_777_216).unwrap();

        assert_eq!(super::profile_sampling_interval_bytes(), Some(1_048_576));
        assert_eq!(super::guarded_sampling_interval_bytes(), Some(16_777_216));

        if let Some(interval) = original_profile {
            set_profile_sampling_interval_bytes(interval).unwrap();
        }
        if let Some(interval) = original_guarded {
            set_guarded_sampling_interval_bytes(interval).unwrap();
        }
    }

    #[test]
    fn guarded_sampling_config_applies_without_activation() {
        let _lock = global_tcmalloc_lock();
        let original_guarded = super::guarded_sampling_interval_bytes();

        configure_guarded_sampling(GuardedSamplingConfig {
            sampling_interval_bytes: Some(33_554_432),
            activate: false,
        })
        .unwrap();
        assert_eq!(super::guarded_sampling_interval_bytes(), Some(33_554_432));

        if let Some(interval) = original_guarded {
            set_guarded_sampling_interval_bytes(interval).unwrap();
        }
    }

    #[test]
    fn heap_profile_snapshot_is_typed() {
        let profile = profile_snapshot(ProfileType::Heap).unwrap();
        assert_eq!(profile.profile_type, ProfileType::Heap);
    }

    #[test]
    fn allocation_profile_session_stops_to_typed_profile() {
        let _lock = global_tcmalloc_lock();
        let original_profile = super::profile_sampling_interval_bytes();
        set_profile_sampling_interval_bytes(1).unwrap();

        let session = start_allocation_profiling();
        let mut allocations = Vec::with_capacity(128);
        for value in 0..128 {
            allocations.push(Box::new(value));
        }
        let profile = session.stop().unwrap();
        assert_eq!(profile.profile_type, ProfileType::Allocations);
        assert!(!profile.otel_records().is_empty() || profile.samples.is_empty());

        if let Some(interval) = original_profile {
            set_profile_sampling_interval_bytes(interval).unwrap();
        }
    }
}
