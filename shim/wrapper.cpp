#include "tcmalloc/malloc_extension.h"
#include <string>

extern "C" {
// C-style wrapper function to call GetStats
const char *get_stats() {
  static std::string stats = tcmalloc::MallocExtension::GetStats();
  return stats.c_str(); // Return pointer to the internal string data
}
}
