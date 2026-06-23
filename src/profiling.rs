use crate::{TcmallocError, ffi};

/// `TCMalloc` profile kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileType {
    /// Approximation of currently live heap allocations.
    Heap,
    /// Fragmentation-oriented view of sampled live allocations.
    Fragmentation,
    /// Sample of objects live near the allocator's recent peak heap usage.
    PeakHeap,
    /// Allocations observed during an allocation profiling session.
    Allocations,
    /// Lifetimes of sampled objects observed during a lifetime profiling session.
    Lifetimes,
    /// Profile kind parsed from upstream data that this crate does not know yet.
    Unknown(u8),
}

/// How a sampled allocation was made.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocationKind {
    /// C++ `new`.
    New,
    /// C `malloc`-family allocation.
    Malloc,
    /// Explicitly aligned allocation.
    AlignedMalloc,
    /// Allocation kind parsed from upstream data that this crate does not know yet.
    Unknown(i32),
}

/// Access pattern tcmalloc selected for a sampled allocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessPattern {
    /// Hot allocation.
    Hot,
    /// Cold allocation.
    Cold,
    /// Access pattern parsed from upstream data that this crate does not know yet.
    Unknown(i32),
}

/// GWP-ASan guarded sampling result for one sampled allocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardedStatus {
    /// The requested allocation was larger than the guardable page range.
    LargerThanOnePage,
    /// Guarded sampling was disabled.
    Disabled,
    /// Guard selection was rate limited.
    RateLimited,
    /// Allocation was too small for guarded sampling.
    TooSmall,
    /// No guarded slots were available.
    NoAvailableSlots,
    /// The allocator failed to protect pages with `mprotect`.
    MProtectFailed,
    /// Allocation was filtered by upstream guard selection.
    Filtered,
    /// Guarded sampling was not attempted for this allocation.
    NotAttempted,
    /// Guarding was requested but was not necessarily the final state.
    Requested,
    /// Guarding was required by upstream policy.
    Required,
    /// Allocation was guarded by `GWP-ASan`.
    Guarded,
    /// Guarded status parsed from upstream data that this crate does not know yet.
    Unknown(i32),
}

/// Typed allocation sample suitable for telemetry export.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileSample {
    /// Estimated bytes represented by this sampled allocation.
    pub estimated_bytes: i64,
    /// Reported sample count for this call stack.
    pub count: i64,
    /// Allocation size requested by the caller.
    pub requested_size: u64,
    /// Requested alignment in bytes, or zero when upstream did not report one.
    pub requested_alignment: u64,
    /// Actual bytes reserved by the allocator for this sample.
    pub allocated_size: u64,
    /// Whether the allocation used a size-returning `operator new` variant.
    pub requested_size_returning: bool,
    /// Raw hot/cold hint value provided to upstream `TCMalloc`.
    pub access_hint: u8,
    /// Access pattern ultimately selected by `TCMalloc`.
    pub access_allocated: AccessPattern,
    /// Whether a lifetime observation was right-censored.
    pub is_censored: bool,
    /// `GWP-ASan` guard decision or final guarded state.
    pub guarded_status: GuardedStatus,
    /// Allocation API family used for the sampled allocation.
    pub allocation_kind: AllocationKind,
    /// Stack depth reported by upstream `TCMalloc`.
    pub depth: i32,
    /// Upstream profile id for correlating allocation and lifetime data.
    pub profile_id: u64,
    /// Allocation timestamp in Unix nanoseconds when upstream reports it.
    pub allocation_time_unix_nanos: Option<i64>,
    /// Average lifetime in nanoseconds for lifetime profile samples.
    pub avg_lifetime_nanos: i64,
    /// Standard deviation of lifetime in nanoseconds for lifetime samples.
    pub stddev_lifetime_nanos: i64,
    /// Minimum lifetime in nanoseconds for lifetime profile samples.
    pub min_lifetime_nanos: i64,
    /// Maximum lifetime in nanoseconds for lifetime profile samples.
    pub max_lifetime_nanos: i64,
    /// Whether allocation and deallocation happened on the same physical CPU.
    pub allocator_deallocator_physical_cpu_matched: Option<bool>,
    /// Whether allocation and deallocation happened on the same virtual CPU.
    pub allocator_deallocator_virtual_cpu_matched: Option<bool>,
    /// Whether allocation and deallocation happened on the same L3 cache.
    pub allocator_deallocator_l3_matched: Option<bool>,
    /// Whether allocation and deallocation happened on the same NUMA node.
    pub allocator_deallocator_numa_matched: Option<bool>,
    /// Whether allocation and deallocation happened on the same thread.
    pub allocator_deallocator_thread_matched: Option<bool>,
    /// Start address of the sampled allocation span when upstream reports it.
    pub span_start_address: usize,
    /// Raw stack instruction addresses for the sample.
    pub stack: Vec<usize>,
}

/// Typed tcmalloc profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TcmallocProfile {
    /// Kind of profile represented by these samples.
    pub profile_type: ProfileType,
    /// Session start time in Unix nanoseconds, if upstream reports it.
    pub start_unix_nanos: Option<i64>,
    /// Profile collection duration in nanoseconds.
    pub duration_nanos: i64,
    /// Typed allocation samples.
    pub samples: Vec<ProfileSample>,
    /// Bridge serialization kept for debugging parser issues.
    pub raw: String,
}

/// OTel-shaped metric record produced from a profile sample.
///
/// The crate deliberately does not depend on an OpenTelemetry Rust crate. This
/// shape is stable enough for callers to map into their chosen exporter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OtelProfileRecord {
    /// Metric name, for example `tcmalloc.profile.sample.estimated_bytes`.
    pub name: String,
    /// Metric value.
    pub value: i64,
    /// OpenTelemetry unit string.
    pub unit: String,
    /// String attributes to attach to the exported metric point.
    pub attributes: Vec<OtelAttribute>,
}

/// String attribute for [`OtelProfileRecord`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OtelAttribute {
    /// Attribute key.
    pub key: String,
    /// Attribute value.
    pub value: String,
}

/// Active allocation or lifetime profiling session.
#[must_use = "profiling continues until the session is stopped or dropped"]
pub struct ProfilingSession {
    inner: cxx::UniquePtr<ffi::AllocationProfileSession>,
}

/// Starts an allocation profiling session.
pub fn start_allocation_profiling() -> ProfilingSession {
    ProfilingSession {
        inner: ffi::start_allocation_profiling(),
    }
}

/// Starts a lifetime profiling session.
pub fn start_lifetime_profiling() -> ProfilingSession {
    ProfilingSession {
        inner: ffi::start_lifetime_profiling(),
    }
}

/// Takes an instantaneous tcmalloc profile snapshot.
///
/// # Errors
///
/// Returns an error if the bridge returns malformed profile data.
pub fn profile_snapshot(profile_type: ProfileType) -> Result<TcmallocProfile, TcmallocError> {
    if let ProfileType::Unknown(value) = profile_type {
        return Err(TcmallocError::UnsupportedProfileType(value));
    }

    parse_profile(ffi::profile_snapshot(profile_type.as_ffi()))
}

impl ProfilingSession {
    /// Stops the profiling session and returns typed samples.
    ///
    /// # Errors
    ///
    /// Returns an error if the bridge returns malformed profile data.
    pub fn stop(mut self) -> Result<TcmallocProfile, TcmallocError> {
        parse_profile(self.inner.pin_mut().stop())
    }
}

impl TcmallocProfile {
    /// Converts samples into OTel-shaped metric records.
    #[must_use]
    pub fn otel_records(&self) -> Vec<OtelProfileRecord> {
        let mut records = Vec::with_capacity(self.samples.len() * 2);

        for sample in &self.samples {
            let attributes = sample.otel_attributes(self.profile_type);
            records.push(OtelProfileRecord {
                name: "tcmalloc.profile.sample.estimated_bytes".to_string(),
                value: sample.estimated_bytes,
                unit: "By".to_string(),
                attributes: attributes.clone(),
            });
            records.push(OtelProfileRecord {
                name: "tcmalloc.profile.sample.count".to_string(),
                value: sample.count,
                unit: "{sample}".to_string(),
                attributes,
            });
        }

        records
    }
}

impl ProfileSample {
    /// Returns the sampled stack as hexadecimal addresses separated by `;`.
    #[must_use]
    pub fn stack_hex(&self) -> String {
        self.stack
            .iter()
            .map(|address| format!("{address:#x}"))
            .collect::<Vec<_>>()
            .join(";")
    }

    fn otel_attributes(&self, profile_type: ProfileType) -> Vec<OtelAttribute> {
        vec![
            OtelAttribute::new("tcmalloc.profile.type", profile_type.as_str()),
            OtelAttribute::new("tcmalloc.allocation.kind", self.allocation_kind.as_str()),
            OtelAttribute::new("tcmalloc.guarded.status", self.guarded_status.as_str()),
            OtelAttribute::new("tcmalloc.requested_size", self.requested_size.to_string()),
            OtelAttribute::new("tcmalloc.allocated_size", self.allocated_size.to_string()),
            OtelAttribute::new("tcmalloc.stack", self.stack_hex()),
        ]
    }
}

impl OtelAttribute {
    fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

impl ProfileType {
    pub(crate) const fn as_ffi(self) -> u8 {
        match self {
            Self::Heap => 0,
            Self::Fragmentation => 1,
            Self::PeakHeap => 2,
            Self::Allocations => 3,
            Self::Lifetimes => 4,
            Self::Unknown(value) => value,
        }
    }

    const fn from_ffi(value: u8) -> Self {
        match value {
            0 => Self::Heap,
            1 => Self::Fragmentation,
            2 => Self::PeakHeap,
            3 => Self::Allocations,
            4 => Self::Lifetimes,
            value => Self::Unknown(value),
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Heap => "heap",
            Self::Fragmentation => "fragmentation",
            Self::PeakHeap => "peak_heap",
            Self::Allocations => "allocations",
            Self::Lifetimes => "lifetimes",
            Self::Unknown(_) => "unknown",
        }
    }
}

impl AllocationKind {
    const fn from_ffi(value: i32) -> Self {
        match value {
            0 => Self::New,
            1 => Self::Malloc,
            2 => Self::AlignedMalloc,
            value => Self::Unknown(value),
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Malloc => "malloc",
            Self::AlignedMalloc => "aligned_malloc",
            Self::Unknown(_) => "unknown",
        }
    }
}

impl AccessPattern {
    const fn from_ffi(value: i32) -> Self {
        match value {
            0 => Self::Hot,
            1 => Self::Cold,
            value => Self::Unknown(value),
        }
    }
}

impl GuardedStatus {
    const fn from_ffi(value: i32) -> Self {
        match value {
            -1 => Self::LargerThanOnePage,
            -2 => Self::Disabled,
            -3 => Self::RateLimited,
            -4 => Self::TooSmall,
            -5 => Self::NoAvailableSlots,
            -6 => Self::MProtectFailed,
            -7 => Self::Filtered,
            0 => Self::NotAttempted,
            1 => Self::Requested,
            2 => Self::Required,
            10 => Self::Guarded,
            value => Self::Unknown(value),
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::LargerThanOnePage => "larger_than_one_page",
            Self::Disabled => "disabled",
            Self::RateLimited => "rate_limited",
            Self::TooSmall => "too_small",
            Self::NoAvailableSlots => "no_available_slots",
            Self::MProtectFailed => "mprotect_failed",
            Self::Filtered => "filtered",
            Self::NotAttempted => "not_attempted",
            Self::Requested => "requested",
            Self::Required => "required",
            Self::Guarded => "guarded",
            Self::Unknown(_) => "unknown",
        }
    }
}

fn parse_profile(raw: String) -> Result<TcmallocProfile, TcmallocError> {
    let mut profile_type = None;
    let mut start_unix_nanos = None;
    let mut duration_nanos = 0;
    let mut samples = Vec::new();

    for (line_index, line) in raw.lines().enumerate() {
        let fields = line.split('\t').collect::<Vec<_>>();
        match fields.first().copied() {
            Some("profile") if fields.len() == 4 => {
                let parsed_type = parse_u8(fields[1], line_index)?;
                let start = parse_i64(fields[2], line_index)?;
                profile_type = Some(ProfileType::from_ffi(parsed_type));
                start_unix_nanos = (start >= 0).then_some(start);
                duration_nanos = parse_i64(fields[3], line_index)?;
            }
            Some("sample") => samples.push(parse_sample(&fields, line_index)?),
            Some("version" | "") | None => {}
            _ => {
                return Err(TcmallocError::InvalidProfileData(format!(
                    "unexpected record on line {}",
                    line_index + 1
                )));
            }
        }
    }

    let Some(profile_type) = profile_type else {
        return Err(TcmallocError::InvalidProfileData(
            "profile header missing".to_string(),
        ));
    };

    Ok(TcmallocProfile {
        profile_type,
        start_unix_nanos,
        duration_nanos,
        samples,
        raw,
    })
}

fn parse_sample(fields: &[&str], line_index: usize) -> Result<ProfileSample, TcmallocError> {
    if fields.len() != 27 {
        return Err(TcmallocError::InvalidProfileData(format!(
            "sample on line {} has {} fields",
            line_index + 1,
            fields.len()
        )));
    }

    let allocation_time = parse_i64(fields[14], line_index)?;
    let stack_depth = parse_usize(fields[25], line_index)?;
    let stack = parse_stack(fields[26], line_index)?;
    if stack.len() != stack_depth {
        return Err(TcmallocError::InvalidProfileData(format!(
            "sample on line {} has stack depth {} but {} addresses",
            line_index + 1,
            stack_depth,
            stack.len()
        )));
    }

    Ok(ProfileSample {
        estimated_bytes: parse_i64(fields[1], line_index)?,
        count: parse_i64(fields[2], line_index)?,
        requested_size: parse_u64(fields[3], line_index)?,
        requested_alignment: parse_u64(fields[4], line_index)?,
        allocated_size: parse_u64(fields[5], line_index)?,
        requested_size_returning: parse_bool(fields[6], line_index)?,
        access_hint: parse_u8(fields[7], line_index)?,
        access_allocated: AccessPattern::from_ffi(parse_i32(fields[8], line_index)?),
        is_censored: parse_bool(fields[9], line_index)?,
        guarded_status: GuardedStatus::from_ffi(parse_i32(fields[10], line_index)?),
        allocation_kind: AllocationKind::from_ffi(parse_i32(fields[11], line_index)?),
        depth: parse_i32(fields[12], line_index)?,
        profile_id: parse_u64(fields[13], line_index)?,
        allocation_time_unix_nanos: (allocation_time >= 0).then_some(allocation_time),
        avg_lifetime_nanos: parse_i64(fields[15], line_index)?,
        stddev_lifetime_nanos: parse_i64(fields[16], line_index)?,
        min_lifetime_nanos: parse_i64(fields[17], line_index)?,
        max_lifetime_nanos: parse_i64(fields[18], line_index)?,
        allocator_deallocator_physical_cpu_matched: parse_optional_bool(fields[19], line_index)?,
        allocator_deallocator_virtual_cpu_matched: parse_optional_bool(fields[20], line_index)?,
        allocator_deallocator_l3_matched: parse_optional_bool(fields[21], line_index)?,
        allocator_deallocator_numa_matched: parse_optional_bool(fields[22], line_index)?,
        allocator_deallocator_thread_matched: parse_optional_bool(fields[23], line_index)?,
        span_start_address: parse_usize(fields[24], line_index)?,
        stack,
    })
}

fn parse_stack(value: &str, line_index: usize) -> Result<Vec<usize>, TcmallocError> {
    if value.is_empty() {
        return Ok(Vec::new());
    }

    value
        .split(',')
        .map(|address| parse_usize(address, line_index))
        .collect()
}

fn parse_bool(value: &str, line_index: usize) -> Result<bool, TcmallocError> {
    match value {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => Err(parse_error(value, line_index)),
    }
}

fn parse_optional_bool(value: &str, line_index: usize) -> Result<Option<bool>, TcmallocError> {
    match value {
        "-1" => Ok(None),
        "0" => Ok(Some(false)),
        "1" => Ok(Some(true)),
        _ => Err(parse_error(value, line_index)),
    }
}

fn parse_u8(value: &str, line_index: usize) -> Result<u8, TcmallocError> {
    value.parse().map_err(|_| parse_error(value, line_index))
}

fn parse_i32(value: &str, line_index: usize) -> Result<i32, TcmallocError> {
    value.parse().map_err(|_| parse_error(value, line_index))
}

fn parse_i64(value: &str, line_index: usize) -> Result<i64, TcmallocError> {
    value.parse().map_err(|_| parse_error(value, line_index))
}

fn parse_u64(value: &str, line_index: usize) -> Result<u64, TcmallocError> {
    value.parse().map_err(|_| parse_error(value, line_index))
}

fn parse_usize(value: &str, line_index: usize) -> Result<usize, TcmallocError> {
    let Some(hex) = value.strip_prefix("0x") else {
        return value.parse().map_err(|_| parse_error(value, line_index));
    };
    usize::from_str_radix(hex, 16).map_err(|_| parse_error(value, line_index))
}

fn parse_error(value: &str, line_index: usize) -> TcmallocError {
    TcmallocError::InvalidProfileData(format!(
        "could not parse {value:?} on line {}",
        line_index + 1
    ))
}

#[cfg(test)]
mod tests {
    use crate::TcmallocError;

    use super::{GuardedStatus, ProfileType, parse_profile, profile_snapshot};

    #[test]
    fn parses_profile_without_samples() {
        let profile = parse_profile("version\t1\nprofile\t0\t-1\t0\n".to_string()).unwrap();
        assert_eq!(profile.profile_type, ProfileType::Heap);
        assert_eq!(profile.start_unix_nanos, None);
        assert!(profile.samples.is_empty());
    }

    #[test]
    fn parses_profile_samples_and_otel_records() {
        let raw = "version\t1\n\
                   profile\t3\t10\t20\n\
                   sample\t64\t2\t32\t8\t32\t0\t255\t0\t0\t10\t1\t2\t99\t100\t1\t2\t3\t4\t-1\t1\t0\t-1\t1\t0x1000\t2\t0x1,0x2\n";
        let profile = parse_profile(raw.to_string()).unwrap();

        assert_eq!(profile.profile_type, ProfileType::Allocations);
        assert_eq!(profile.samples.len(), 1);
        assert_eq!(profile.samples[0].guarded_status, GuardedStatus::Guarded);
        assert_eq!(profile.samples[0].stack, vec![1, 2]);

        let records = profile.otel_records();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "tcmalloc.profile.sample.estimated_bytes");
        assert!(
            records[0]
                .attributes
                .iter()
                .any(|attr| attr.key == "tcmalloc.stack" && attr.value == "0x1;0x2")
        );
    }

    #[test]
    fn rejects_mismatched_stack_depth() {
        let raw = "version\t1\n\
                   profile\t3\t10\t20\n\
                   sample\t64\t2\t32\t8\t32\t0\t255\t0\t0\t10\t1\t2\t99\t100\t1\t2\t3\t4\t-1\t1\t0\t-1\t1\t0x1000\t3\t0x1,0x2\n";

        let error = parse_profile(raw.to_string()).unwrap_err();
        assert!(error.to_string().contains("stack depth 3"));
    }

    #[test]
    fn rejects_unknown_profile_snapshot_request() {
        let error = profile_snapshot(ProfileType::Unknown(200)).unwrap_err();
        assert!(matches!(error, TcmallocError::UnsupportedProfileType(200)));
    }
}
