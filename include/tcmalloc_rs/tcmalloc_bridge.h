#pragma once

#include <cstddef>
#include <cstdint>
#include <memory>

#include "rust/cxx.h"
#include "tcmalloc/malloc_extension.h"

namespace tcmalloc_rs {

class AllocationProfileSession final {
 public:
  explicit AllocationProfileSession(
      tcmalloc::MallocExtension::AllocationProfilingToken token);
  AllocationProfileSession(AllocationProfileSession&&) = delete;
  AllocationProfileSession(const AllocationProfileSession&) = delete;
  AllocationProfileSession& operator=(AllocationProfileSession&&) = delete;
  AllocationProfileSession& operator=(const AllocationProfileSession&) =
      delete;

  rust::String stop();

 private:
  tcmalloc::MallocExtension::AllocationProfilingToken token_;
  bool active_ = true;
};

rust::String stats();
rust::String properties();
std::int64_t numeric_property(rust::Str property);

rust::String profile_snapshot(std::uint8_t profile_type);
std::unique_ptr<AllocationProfileSession> start_allocation_profiling();
std::unique_ptr<AllocationProfileSession> start_lifetime_profiling();

void activate_guarded_sampling();
void release_memory_to_system(std::size_t bytes);
std::int64_t profile_sampling_interval();
void set_profile_sampling_interval(std::int64_t interval);
std::int64_t guarded_sampling_interval();
void set_guarded_sampling_interval(std::int64_t interval);

void mark_thread_idle();
void mark_thread_busy();
std::size_t release_cpu_memory(std::int32_t cpu);
bool per_cpu_caches_active();
std::int32_t max_per_cpu_cache_size();
void set_max_per_cpu_cache_size(std::int32_t value);
std::int64_t max_total_thread_cache_bytes();
void set_max_total_thread_cache_bytes(std::int64_t value);
bool background_process_actions_enabled();
void set_background_process_actions_enabled(bool value);
std::int64_t background_process_sleep_interval_nanos();
void set_background_process_sleep_interval_nanos(std::int64_t value);
std::int64_t skip_subrelease_short_interval_nanos();
void set_skip_subrelease_short_interval_nanos(std::int64_t value);
std::int64_t skip_subrelease_long_interval_nanos();
void set_skip_subrelease_long_interval_nanos(std::int64_t value);
std::size_t estimated_allocated_size(std::size_t size);
std::size_t memory_limit(bool hard);
void set_memory_limit(std::size_t limit, bool hard);
void process_background_actions();
bool needs_process_background_actions();
std::size_t background_release_rate();
void set_background_release_rate(std::size_t rate);

}  // namespace tcmalloc_rs
