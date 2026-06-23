# tcmalloc-rs

Rust allocator, statistics, profiling, GWP-ASan, and tuning bindings for Google
TCMalloc.

This crate is Linux-only and Bazel-only. It links the GitHub-pinned `tcmalloc`
Bazel module from `MODULE.bazel` and uses `cxx.rs` for the C++
`tcmalloc::MallocExtension` bridge.

## Scope

The crate has two layers:

- `TcMalloc` implements Rust's `GlobalAlloc` through TCMalloc's exported C ABI.
- The typed telemetry and tuning APIs call `tcmalloc::MallocExtension` through a
  small CXX bridge.

The public API is higher-level than the C++ interface where that pays off:

- raw reports are still available with `stats()` and `print_stats()`
- machine-readable counters are exposed as typed structs
- profile snapshots and profiling sessions return typed samples
- profile samples can be converted into OpenTelemetry-shaped metric records
- runtime knobs can be applied individually or through `TcmallocConfig`

The crate deliberately does not depend on an OpenTelemetry crate. OTel crates
change quickly, and applications often have their own meter/provider setup.
Instead, `TcmallocProfile::otel_records()` returns a small exporter-neutral
shape that can be mapped into the caller's chosen OTel SDK.

## Install The Allocator

Install `TcMalloc` in a binary crate:

```rust
#[global_allocator]
static ALLOC: tcmalloc::TcMalloc = tcmalloc::TcMalloc;
```

Do this only once in the final binary. Libraries should not install a global
allocator for their dependents.

## Stats

Use `malloc_stats()` when code needs structured data:

```rust
let stats = tcmalloc::malloc_stats();

if let Some(bytes) = stats.properties.generic.current_allocated_bytes {
    println!("application bytes in use: {bytes}");
}
```

`malloc_stats()` combines:

- `MallocExtension::GetStats()` as `MallocStats::raw`
- `MallocExtension::GetProperties()` as `AllocatorProperties`
- parsed `MALLOC:`, `TOTAL:`, `MALLOC EXPERIMENTS:`, and `MALLOC HOOKS:`
  sections from the human report

Prefer `AllocatorProperties` for automation. The human report is intentionally
kept in `MallocStats::raw` for diagnostics, but it is less stable as a machine
contract.

## Profiling

TCMalloc exposes instantaneous snapshots and scoped allocation/lifetime
profiling sessions:

```rust
let session = tcmalloc::start_allocation_profiling();
run_workload();
let profile = session.stop()?;

for sample in &profile.samples {
    println!(
        "estimated={} requested={} allocated={} stack={}",
        sample.estimated_bytes,
        sample.requested_size,
        sample.allocated_size,
        sample.stack_hex(),
    );
}
# Ok::<(), tcmalloc::TcmallocError>(())
```

Profile samples are sampled and weighted, not a literal list of every
allocation. Keep the profile sampling interval relatively large for lower
overhead in production.

## OpenTelemetry Export

`TcmallocProfile::otel_records()` converts each profile sample into two records:

- `tcmalloc.profile.sample.estimated_bytes`, unit `By`
- `tcmalloc.profile.sample.count`, unit `{sample}`

Each record carries attributes for profile type, allocation kind, guarded
status, requested size, allocated size, and stack addresses.

## Runtime Tuning

Use `TcmallocConfig` when applying several knobs together:

```rust
tcmalloc::TcmallocConfig {
    profile_sampling_interval_bytes: Some(2 * 1024 * 1024),
    guarded_sampling: Some(tcmalloc::GuardedSamplingConfig {
        sampling_interval_bytes: Some(100_000_000),
        activate: false,
    }),
    max_per_cpu_cache_size: Some(1_500_000),
    max_total_thread_cache_bytes: Some(16 * 1024 * 1024),
    background_process_actions_enabled: Some(true),
    background_process_sleep_interval_nanos: Some(10_000_000),
    background_release_rate_bytes_per_second: Some(4 * 1024 * 1024),
    soft_memory_limit: Some(2 * 1024 * 1024 * 1024),
    ..Default::default()
}
.apply()?;
# Ok::<(), tcmalloc::TcmallocError>(())
```

Every field is optional. `None` means "leave the current allocator value alone".

## Build And Verification

Useful package-local checks:

```bash
bazelisk build //:lib
bazelisk test //:test
bazelisk test //:format
bazelisk build //:clippy
bazelisk build //:doc
bazelisk test //:doc_test
```

## License

Apache-2.0
