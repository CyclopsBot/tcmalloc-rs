use crate::{TcmallocError, checked_i32, checked_i64, ffi, non_negative_i32, non_negative_i64};

/// The default guarded sampling interval documented by tcmalloc.
pub const DEFAULT_GWP_ASAN_SAMPLING_INTERVAL_BYTES: u64 = 100_000_000;

/// `TCMalloc`'s documented lower-overhead canary recommendation for GWP-ASan.
pub const RECOMMENDED_GWP_ASAN_SAMPLING_INTERVAL_BYTES: u64 = 8 * 1024 * 1024;

/// Runtime GWP-ASan guarded sampling configuration.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GuardedSamplingConfig {
    /// Approximate bytes allocated between guarded samples.
    pub sampling_interval_bytes: Option<u64>,
    /// Whether to activate guarded sampling after applying the interval.
    ///
    /// Activation is intentionally explicit because tcmalloc may crash the
    /// process when GWP-ASan detects a real heap bug.
    pub activate: bool,
}

/// Runtime tcmalloc tuning configuration.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TcmallocConfig {
    /// Approximate bytes allocated between allocation profile samples.
    pub profile_sampling_interval_bytes: Option<u64>,
    /// Optional `GWP-ASan` guarded sampling configuration.
    pub guarded_sampling: Option<GuardedSamplingConfig>,
    /// Maximum cache size per CPU cache in bytes.
    pub max_per_cpu_cache_size: Option<u32>,
    /// Whole-process maximum thread cache size in bytes.
    pub max_total_thread_cache_bytes: Option<u64>,
    /// Enables or disables allocator background actions.
    pub background_process_actions_enabled: Option<bool>,
    /// Background action sleep interval in nanoseconds.
    pub background_process_sleep_interval_nanos: Option<u64>,
    /// Short interval for delayed subrelease demand history, in nanoseconds.
    pub skip_subrelease_short_interval_nanos: Option<u64>,
    /// Long interval for delayed subrelease demand history, in nanoseconds.
    pub skip_subrelease_long_interval_nanos: Option<u64>,
    /// Page heap release rate in bytes per second.
    pub background_release_rate_bytes_per_second: Option<usize>,
    /// Soft memory limit in bytes.
    pub soft_memory_limit: Option<usize>,
    /// Hard memory limit in bytes.
    pub hard_memory_limit: Option<usize>,
}

/// Current runtime tcmalloc tuning values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TuningSnapshot {
    /// Current allocation profile sampling interval, if known.
    pub profile_sampling_interval_bytes: Option<u64>,
    /// Current `GWP-ASan` guarded sampling interval, if known.
    pub guarded_sampling_interval_bytes: Option<u64>,
    /// Whether this process is using per-CPU caches.
    pub per_cpu_caches_active: bool,
    /// Current maximum per-CPU cache size in bytes, if known.
    pub max_per_cpu_cache_size: Option<u32>,
    /// Current whole-process thread cache limit in bytes, if known.
    pub max_total_thread_cache_bytes: Option<u64>,
    /// Whether background allocator actions are enabled.
    pub background_process_actions_enabled: bool,
    /// Background action sleep interval in nanoseconds, if known.
    pub background_process_sleep_interval_nanos: Option<u64>,
    /// Short delayed-subrelease interval in nanoseconds, if known.
    pub skip_subrelease_short_interval_nanos: Option<u64>,
    /// Long delayed-subrelease interval in nanoseconds, if known.
    pub skip_subrelease_long_interval_nanos: Option<u64>,
    /// Whether this platform wants `ProcessBackgroundActions` to run.
    pub needs_process_background_actions: bool,
    /// Page heap release rate in bytes per second.
    pub background_release_rate_bytes_per_second: usize,
    /// Current soft memory limit in bytes.
    pub soft_memory_limit: usize,
    /// Current hard memory limit in bytes.
    pub hard_memory_limit: usize,
}

/// Memory limit kind used by `MallocExtension`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryLimitKind {
    Soft,
    Hard,
}

impl GuardedSamplingConfig {
    /// Production canary preset recommended by tcmalloc's GWP-ASan docs.
    #[must_use]
    pub const fn production_canary() -> Self {
        Self {
            sampling_interval_bytes: Some(RECOMMENDED_GWP_ASAN_SAMPLING_INTERVAL_BYTES),
            activate: true,
        }
    }

    /// Default-overhead activation using tcmalloc's documented default rate.
    #[must_use]
    pub const fn default_overhead() -> Self {
        Self {
            sampling_interval_bytes: Some(DEFAULT_GWP_ASAN_SAMPLING_INTERVAL_BYTES),
            activate: true,
        }
    }

    /// Applies this GWP-ASan config.
    ///
    /// # Errors
    ///
    /// Returns an error if `sampling_interval_bytes` does not fit tcmalloc's
    /// signed interval API.
    pub fn apply(self) -> Result<(), TcmallocError> {
        configure_guarded_sampling(self)
    }
}

impl TcmallocConfig {
    /// Applies every configured runtime knob.
    ///
    /// # Errors
    ///
    /// Returns an error if any configured byte or nanosecond value does not fit
    /// tcmalloc's signed runtime APIs.
    pub fn apply(self) -> Result<(), TcmallocError> {
        if let Some(interval) = self.profile_sampling_interval_bytes {
            set_profile_sampling_interval_bytes(interval)?;
        }
        if let Some(guarded_sampling) = self.guarded_sampling {
            configure_guarded_sampling(guarded_sampling)?;
        }
        if let Some(value) = self.max_per_cpu_cache_size {
            set_max_per_cpu_cache_size(value)?;
        }
        if let Some(value) = self.max_total_thread_cache_bytes {
            set_max_total_thread_cache_bytes(value)?;
        }
        if let Some(value) = self.background_process_actions_enabled {
            set_background_process_actions_enabled(value);
        }
        if let Some(value) = self.background_process_sleep_interval_nanos {
            set_background_process_sleep_interval_nanos(value)?;
        }
        if let Some(value) = self.skip_subrelease_short_interval_nanos {
            set_skip_subrelease_short_interval_nanos(value)?;
        }
        if let Some(value) = self.skip_subrelease_long_interval_nanos {
            set_skip_subrelease_long_interval_nanos(value)?;
        }
        if let Some(value) = self.background_release_rate_bytes_per_second {
            set_background_release_rate(value);
        }
        if let Some(value) = self.soft_memory_limit {
            set_soft_memory_limit(value);
        }
        if let Some(value) = self.hard_memory_limit {
            set_hard_memory_limit(value);
        }
        Ok(())
    }
}

/// Applies a GWP-ASan guarded sampling configuration.
///
/// # Errors
///
/// Returns an error if `sampling_interval_bytes` does not fit tcmalloc's signed
/// interval API.
pub fn configure_guarded_sampling(config: GuardedSamplingConfig) -> Result<(), TcmallocError> {
    if let Some(interval) = config.sampling_interval_bytes {
        set_guarded_sampling_interval_bytes(interval)?;
    }
    if config.activate {
        ffi::activate_guarded_sampling();
    }
    Ok(())
}

/// Captures current runtime tuning values.
#[must_use]
pub fn tuning_snapshot() -> TuningSnapshot {
    TuningSnapshot {
        profile_sampling_interval_bytes: profile_sampling_interval_bytes(),
        guarded_sampling_interval_bytes: guarded_sampling_interval_bytes(),
        per_cpu_caches_active: per_cpu_caches_active(),
        max_per_cpu_cache_size: max_per_cpu_cache_size(),
        max_total_thread_cache_bytes: max_total_thread_cache_bytes(),
        background_process_actions_enabled: background_process_actions_enabled(),
        background_process_sleep_interval_nanos: background_process_sleep_interval_nanos(),
        skip_subrelease_short_interval_nanos: skip_subrelease_short_interval_nanos(),
        skip_subrelease_long_interval_nanos: skip_subrelease_long_interval_nanos(),
        needs_process_background_actions: needs_process_background_actions(),
        background_release_rate_bytes_per_second: background_release_rate(),
        soft_memory_limit: soft_memory_limit(),
        hard_memory_limit: hard_memory_limit(),
    }
}

/// Returns the allocation profile sampling interval in bytes.
#[must_use]
pub fn profile_sampling_interval_bytes() -> Option<u64> {
    non_negative_i64(ffi::profile_sampling_interval())
}

/// Sets the allocation profile sampling interval in bytes.
///
/// # Errors
///
/// Returns an error if `interval` does not fit tcmalloc's signed interval API.
pub fn set_profile_sampling_interval_bytes(interval: u64) -> Result<(), TcmallocError> {
    ffi::set_profile_sampling_interval(checked_i64("profile_sampling_interval_bytes", interval)?);
    Ok(())
}

/// Returns the GWP-ASan guarded sampling interval in bytes.
#[must_use]
pub fn guarded_sampling_interval_bytes() -> Option<u64> {
    non_negative_i64(ffi::guarded_sampling_interval())
}

/// Sets the GWP-ASan guarded sampling interval in bytes.
///
/// # Errors
///
/// Returns an error if `interval` does not fit tcmalloc's signed interval API.
pub fn set_guarded_sampling_interval_bytes(interval: u64) -> Result<(), TcmallocError> {
    ffi::set_guarded_sampling_interval(checked_i64("guarded_sampling_interval_bytes", interval)?);
    Ok(())
}

/// Asks tcmalloc to release up to `bytes` bytes of free memory back to the OS.
pub fn release_memory_to_system(bytes: usize) {
    ffi::release_memory_to_system(bytes);
}

/// Marks the current thread idle so tcmalloc can release thread-local resources.
pub fn mark_thread_idle() {
    ffi::mark_thread_idle();
}

/// Marks the current thread busy after a previous [`mark_thread_idle`] call.
pub fn mark_thread_busy() {
    ffi::mark_thread_busy();
}

/// Releases memory assigned to a CPU-local cache and returns the bytes freed.
///
/// # Errors
///
/// Returns an error if `cpu` does not fit tcmalloc's signed CPU API.
pub fn release_cpu_memory(cpu: u32) -> Result<usize, TcmallocError> {
    Ok(ffi::release_cpu_memory(checked_i32("cpu", cpu)?))
}

/// Returns whether tcmalloc is using per-CPU caches.
#[must_use]
pub fn per_cpu_caches_active() -> bool {
    ffi::per_cpu_caches_active()
}

/// Returns the maximum per-CPU cache size in bytes.
#[must_use]
pub fn max_per_cpu_cache_size() -> Option<u32> {
    non_negative_i32(ffi::max_per_cpu_cache_size())
}

/// Sets the maximum per-CPU cache size in bytes.
///
/// # Errors
///
/// Returns an error if `value` does not fit tcmalloc's signed cache-size API.
pub fn set_max_per_cpu_cache_size(value: u32) -> Result<(), TcmallocError> {
    ffi::set_max_per_cpu_cache_size(checked_i32("max_per_cpu_cache_size", value)?);
    Ok(())
}

/// Returns the whole-process maximum thread cache size in bytes.
#[must_use]
pub fn max_total_thread_cache_bytes() -> Option<u64> {
    non_negative_i64(ffi::max_total_thread_cache_bytes())
}

/// Sets the whole-process maximum thread cache size in bytes.
///
/// # Errors
///
/// Returns an error if `value` does not fit tcmalloc's signed cache-size API.
pub fn set_max_total_thread_cache_bytes(value: u64) -> Result<(), TcmallocError> {
    ffi::set_max_total_thread_cache_bytes(checked_i64("max_total_thread_cache_bytes", value)?);
    Ok(())
}

/// Returns whether background allocator actions are enabled.
#[must_use]
pub fn background_process_actions_enabled() -> bool {
    ffi::background_process_actions_enabled()
}

/// Enables or disables background allocator actions.
pub fn set_background_process_actions_enabled(value: bool) {
    ffi::set_background_process_actions_enabled(value);
}

/// Returns the background process sleep interval in nanoseconds.
#[must_use]
pub fn background_process_sleep_interval_nanos() -> Option<u64> {
    non_negative_i64(ffi::background_process_sleep_interval_nanos())
}

/// Sets the background process sleep interval in nanoseconds.
///
/// # Errors
///
/// Returns an error if `value` does not fit tcmalloc's signed duration API.
pub fn set_background_process_sleep_interval_nanos(value: u64) -> Result<(), TcmallocError> {
    ffi::set_background_process_sleep_interval_nanos(checked_i64(
        "background_process_sleep_interval_nanos",
        value,
    )?);
    Ok(())
}

/// Returns the short interval used for delayed subrelease demand history.
#[must_use]
pub fn skip_subrelease_short_interval_nanos() -> Option<u64> {
    non_negative_i64(ffi::skip_subrelease_short_interval_nanos())
}

/// Sets the short interval used for delayed subrelease demand history.
///
/// # Errors
///
/// Returns an error if `value` does not fit tcmalloc's signed duration API.
pub fn set_skip_subrelease_short_interval_nanos(value: u64) -> Result<(), TcmallocError> {
    ffi::set_skip_subrelease_short_interval_nanos(checked_i64(
        "skip_subrelease_short_interval_nanos",
        value,
    )?);
    Ok(())
}

/// Returns the long interval used for delayed subrelease demand history.
#[must_use]
pub fn skip_subrelease_long_interval_nanos() -> Option<u64> {
    non_negative_i64(ffi::skip_subrelease_long_interval_nanos())
}

/// Sets the long interval used for delayed subrelease demand history.
///
/// # Errors
///
/// Returns an error if `value` does not fit tcmalloc's signed duration API.
pub fn set_skip_subrelease_long_interval_nanos(value: u64) -> Result<(), TcmallocError> {
    ffi::set_skip_subrelease_long_interval_nanos(checked_i64(
        "skip_subrelease_long_interval_nanos",
        value,
    )?);
    Ok(())
}

/// Returns the tcmalloc-estimated allocated size for a request.
#[must_use]
pub fn estimated_allocated_size(size: usize) -> usize {
    ffi::estimated_allocated_size(size)
}

/// Returns the soft memory limit in bytes.
#[must_use]
pub fn soft_memory_limit() -> usize {
    memory_limit(MemoryLimitKind::Soft)
}

/// Sets the soft memory limit in bytes.
pub fn set_soft_memory_limit(limit: usize) {
    set_memory_limit(limit, MemoryLimitKind::Soft);
}

/// Returns the hard memory limit in bytes.
#[must_use]
pub fn hard_memory_limit() -> usize {
    memory_limit(MemoryLimitKind::Hard)
}

/// Sets the hard memory limit in bytes.
pub fn set_hard_memory_limit(limit: usize) {
    set_memory_limit(limit, MemoryLimitKind::Hard);
}

/// Returns the configured memory limit in bytes.
#[must_use]
pub fn memory_limit(kind: MemoryLimitKind) -> usize {
    ffi::memory_limit(kind.is_hard())
}

/// Sets the configured memory limit in bytes.
pub fn set_memory_limit(limit: usize, kind: MemoryLimitKind) {
    ffi::set_memory_limit(limit, kind.is_hard());
}

/// Runs tcmalloc background actions.
///
/// When linked against tcmalloc this does not return; use
/// [`run_background_actions_forever`] when the type-level non-returning
/// behavior is useful.
pub fn process_background_actions() {
    ffi::process_background_actions();
}

/// Runs tcmalloc background actions forever.
pub fn run_background_actions_forever() -> ! {
    ffi::process_background_actions();
    loop {
        std::thread::park();
    }
}

/// Returns whether this platform needs/supports background actions.
#[must_use]
pub fn needs_process_background_actions() -> bool {
    ffi::needs_process_background_actions()
}

/// Returns the background release rate in bytes per second.
#[must_use]
pub fn background_release_rate() -> usize {
    ffi::background_release_rate()
}

/// Sets the background release rate in bytes per second.
pub fn set_background_release_rate(rate: usize) {
    ffi::set_background_release_rate(rate);
}

impl MemoryLimitKind {
    const fn is_hard(self) -> bool {
        matches!(self, Self::Hard)
    }
}

#[cfg(test)]
mod tests {
    use crate::TcmallocError;

    use super::{
        DEFAULT_GWP_ASAN_SAMPLING_INTERVAL_BYTES, GuardedSamplingConfig, MemoryLimitKind,
        RECOMMENDED_GWP_ASAN_SAMPLING_INTERVAL_BYTES, TcmallocConfig,
    };

    #[test]
    fn guarded_sampling_presets_are_documented_values() {
        assert_eq!(
            GuardedSamplingConfig::default_overhead().sampling_interval_bytes,
            Some(DEFAULT_GWP_ASAN_SAMPLING_INTERVAL_BYTES)
        );
        assert_eq!(
            GuardedSamplingConfig::production_canary().sampling_interval_bytes,
            Some(RECOMMENDED_GWP_ASAN_SAMPLING_INTERVAL_BYTES)
        );
    }

    #[test]
    fn config_default_is_noop() {
        TcmallocConfig::default().apply().unwrap();
    }

    #[test]
    fn memory_limit_kind_maps_to_expected_variant() {
        assert!(!MemoryLimitKind::Soft.is_hard());
        assert!(MemoryLimitKind::Hard.is_hard());
    }

    #[test]
    fn checked_i64_rejects_too_large_values() {
        assert!(matches!(
            crate::checked_i64("too_large", i64::MAX as u64 + 1),
            Err(TcmallocError::ValueOutOfRange { .. })
        ));
    }

    #[test]
    fn checked_i32_rejects_too_large_values() {
        assert!(matches!(
            crate::checked_i32("too_large", i32::MAX as u32 + 1),
            Err(TcmallocError::ValueOutOfRange { .. })
        ));
    }
}
