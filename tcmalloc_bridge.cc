#include "tcmalloc_rs/tcmalloc_bridge.h"

#include <algorithm>
#include <cstdint>
#include <iomanip>
#include <limits>
#include <memory>
#include <new>
#include <optional>
#include <sstream>
#include <string>

#include "absl/time/time.h"
#include "tcmalloc/malloc_extension.h"

namespace tcmalloc_rs {
namespace {

using tcmalloc::MallocExtension;

std::int64_t DurationNanos(absl::Duration duration) {
  return absl::ToInt64Nanoseconds(duration);
}

std::int64_t TimeNanos(std::optional<absl::Time> time) {
  if (!time.has_value()) {
    return -1;
  }
  return absl::ToUnixNanos(*time);
}

int OptionalBool(std::optional<bool> value) {
  if (!value.has_value()) {
    return -1;
  }
  return *value ? 1 : 0;
}

tcmalloc::ProfileType ProfileTypeFromByte(std::uint8_t value) {
  switch (value) {
    case 0:
      return tcmalloc::ProfileType::kHeap;
    case 1:
      return tcmalloc::ProfileType::kFragmentation;
    case 2:
      return tcmalloc::ProfileType::kPeakHeap;
    case 3:
      return tcmalloc::ProfileType::kAllocations;
    case 4:
      return tcmalloc::ProfileType::kLifetimes;
    default:
      return tcmalloc::ProfileType::kHeap;
  }
}

std::uint8_t ProfileTypeToByte(tcmalloc::ProfileType type) {
  switch (type) {
    case tcmalloc::ProfileType::kHeap:
      return 0;
    case tcmalloc::ProfileType::kFragmentation:
      return 1;
    case tcmalloc::ProfileType::kPeakHeap:
      return 2;
    case tcmalloc::ProfileType::kAllocations:
      return 3;
    case tcmalloc::ProfileType::kLifetimes:
      return 4;
    case tcmalloc::ProfileType::kDoNotUse:
      return 255;
  }
  return 255;
}

int AllocationTypeToInt(tcmalloc::AllocationType type) {
  switch (type) {
    case tcmalloc::AllocationType::New:
      return 0;
    case tcmalloc::AllocationType::Malloc:
      return 1;
    case tcmalloc::AllocationType::AlignedMalloc:
      return 2;
  }
  return -1;
}

std::size_t RequestedAlignment(
    std::optional<std::align_val_t> requested_alignment) {
  if (!requested_alignment.has_value()) {
    return 0;
  }
  return static_cast<std::size_t>(*requested_alignment);
}

void AppendPointer(std::ostream& out, const void* ptr) {
  out << "0x" << std::hex << reinterpret_cast<std::uintptr_t>(ptr) << std::dec;
}

std::string SerializeProfile(tcmalloc::Profile profile) {
  std::ostringstream out;
  out << "version\t1\n";
  out << "profile\t" << static_cast<int>(ProfileTypeToByte(profile.Type()))
      << "\t" << TimeNanos(profile.StartTime()) << "\t"
      << DurationNanos(profile.Duration()) << "\n";

  profile.Iterate([&out](const tcmalloc::Profile::Sample& sample) {
    out << "sample";
    out << "\t" << sample.sum;
    out << "\t" << sample.count;
    out << "\t" << sample.requested_size;
    out << "\t" << RequestedAlignment(sample.requested_alignment);
    out << "\t" << sample.allocated_size;
    out << "\t" << (sample.requested_size_returning ? 1 : 0);
    out << "\t" << static_cast<int>(sample.access_hint);
    out << "\t" << static_cast<int>(sample.access_allocated);
    out << "\t" << (sample.is_censored ? 1 : 0);
    out << "\t" << static_cast<int>(sample.guarded_status);
    out << "\t" << AllocationTypeToInt(sample.type);
    out << "\t" << sample.depth;
    out << "\t" << sample.profile_id;
    out << "\t" << absl::ToUnixNanos(sample.allocation_time);
    out << "\t" << DurationNanos(sample.avg_lifetime);
    out << "\t" << DurationNanos(sample.stddev_lifetime);
    out << "\t" << DurationNanos(sample.min_lifetime);
    out << "\t" << DurationNanos(sample.max_lifetime);
    out << "\t"
        << OptionalBool(sample.allocator_deallocator_physical_cpu_matched);
    out << "\t" << OptionalBool(sample.allocator_deallocator_virtual_cpu_matched);
    out << "\t" << OptionalBool(sample.allocator_deallocator_l3_matched);
    out << "\t" << OptionalBool(sample.allocator_deallocator_numa_matched);
    out << "\t" << OptionalBool(sample.allocator_deallocator_thread_matched);
    out << "\t";
    AppendPointer(out, sample.span_start_address);

    const int depth = std::clamp(
        sample.depth, 0, tcmalloc::Profile::Sample::kMaxStackDepth);
    out << "\t" << depth;
    out << "\t";
    for (int index = 0; index < depth; ++index) {
      if (index != 0) {
        out << ",";
      }
      AppendPointer(out, sample.stack[index]);
    }
    out << "\n";
  });

  return out.str();
}

std::int64_t DurationGetter(absl::Duration (*getter)()) {
  return DurationNanos(getter());
}

void DurationSetter(void (*setter)(absl::Duration), std::int64_t nanos) {
  setter(absl::Nanoseconds(nanos));
}

MallocExtension::LimitKind LimitKind(bool hard) {
  return hard ? MallocExtension::LimitKind::kHard
              : MallocExtension::LimitKind::kSoft;
}

}  // namespace

AllocationProfileSession::AllocationProfileSession(
    MallocExtension::AllocationProfilingToken token)
    : token_(std::move(token)) {}

rust::String AllocationProfileSession::stop() {
  if (!active_) {
    return rust::String::lossy("version\t1\nprofile\t255\t-1\t0\n");
  }

  active_ = false;
  return rust::String::lossy(SerializeProfile(std::move(token_).Stop()));
}

rust::String stats() {
  return rust::String::lossy(MallocExtension::GetStats());
}

rust::String properties() {
  std::ostringstream out;
  for (const auto& [name, property] : MallocExtension::GetProperties()) {
    out << name << "\t" << property.value << "\n";
  }
  return rust::String::lossy(out.str());
}

std::int64_t numeric_property(rust::Str property) {
  const std::string key(property.data(), property.size());
  const auto value = MallocExtension::GetNumericProperty(key);
  if (!value.has_value() ||
      *value > static_cast<std::size_t>(std::numeric_limits<std::int64_t>::max())) {
    return -1;
  }
  return static_cast<std::int64_t>(*value);
}

rust::String profile_snapshot(std::uint8_t profile_type) {
  return rust::String::lossy(
      SerializeProfile(MallocExtension::SnapshotCurrent(
          ProfileTypeFromByte(profile_type))));
}

std::unique_ptr<AllocationProfileSession> start_allocation_profiling() {
  return std::make_unique<AllocationProfileSession>(
      MallocExtension::StartAllocationProfiling());
}

std::unique_ptr<AllocationProfileSession> start_lifetime_profiling() {
  return std::make_unique<AllocationProfileSession>(
      MallocExtension::StartLifetimeProfiling());
}

void activate_guarded_sampling() {
  MallocExtension::ActivateGuardedSampling();
}

void release_memory_to_system(std::size_t bytes) {
  MallocExtension::ReleaseMemoryToSystem(bytes);
}

std::int64_t profile_sampling_interval() {
  return MallocExtension::GetProfileSamplingInterval();
}

void set_profile_sampling_interval(std::int64_t interval) {
  MallocExtension::SetProfileSamplingInterval(interval);
}

std::int64_t guarded_sampling_interval() {
  return MallocExtension::GetGuardedSamplingInterval();
}

void set_guarded_sampling_interval(std::int64_t interval) {
  MallocExtension::SetGuardedSamplingInterval(interval);
}

void mark_thread_idle() { MallocExtension::MarkThreadIdle(); }

void mark_thread_busy() { MallocExtension::MarkThreadBusy(); }

std::size_t release_cpu_memory(std::int32_t cpu) {
  return MallocExtension::ReleaseCpuMemory(cpu);
}

bool per_cpu_caches_active() { return MallocExtension::PerCpuCachesActive(); }

std::int32_t max_per_cpu_cache_size() {
  return MallocExtension::GetMaxPerCpuCacheSize();
}

void set_max_per_cpu_cache_size(std::int32_t value) {
  MallocExtension::SetMaxPerCpuCacheSize(value);
}

std::int64_t max_total_thread_cache_bytes() {
  return MallocExtension::GetMaxTotalThreadCacheBytes();
}

void set_max_total_thread_cache_bytes(std::int64_t value) {
  MallocExtension::SetMaxTotalThreadCacheBytes(value);
}

bool background_process_actions_enabled() {
  return MallocExtension::GetBackgroundProcessActionsEnabled();
}

void set_background_process_actions_enabled(bool value) {
  MallocExtension::SetBackgroundProcessActionsEnabled(value);
}

std::int64_t background_process_sleep_interval_nanos() {
  return DurationGetter(MallocExtension::GetBackgroundProcessSleepInterval);
}

void set_background_process_sleep_interval_nanos(std::int64_t value) {
  DurationSetter(MallocExtension::SetBackgroundProcessSleepInterval, value);
}

std::int64_t skip_subrelease_short_interval_nanos() {
  return DurationGetter(MallocExtension::GetSkipSubreleaseShortInterval);
}

void set_skip_subrelease_short_interval_nanos(std::int64_t value) {
  DurationSetter(MallocExtension::SetSkipSubreleaseShortInterval, value);
}

std::int64_t skip_subrelease_long_interval_nanos() {
  return DurationGetter(MallocExtension::GetSkipSubreleaseLongInterval);
}

void set_skip_subrelease_long_interval_nanos(std::int64_t value) {
  DurationSetter(MallocExtension::SetSkipSubreleaseLongInterval, value);
}

std::size_t estimated_allocated_size(std::size_t size) {
  return MallocExtension::GetEstimatedAllocatedSize(size);
}

std::size_t memory_limit(bool hard) {
  return MallocExtension::GetMemoryLimit(LimitKind(hard));
}

void set_memory_limit(std::size_t limit, bool hard) {
  MallocExtension::SetMemoryLimit(limit, LimitKind(hard));
}

void process_background_actions() {
  MallocExtension::ProcessBackgroundActions();
}

bool needs_process_background_actions() {
  return MallocExtension::NeedsProcessBackgroundActions();
}

std::size_t background_release_rate() {
  return static_cast<std::size_t>(MallocExtension::GetBackgroundReleaseRate());
}

void set_background_release_rate(std::size_t rate) {
  MallocExtension::SetBackgroundReleaseRate(
      static_cast<MallocExtension::BytesPerSecond>(rate));
}

}  // namespace tcmalloc_rs
