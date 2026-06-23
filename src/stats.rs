use crate::ffi;

/// Typed view of `MallocExtension::GetStats()` plus structured properties.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MallocStats {
    /// Original human-readable stats report.
    pub raw: String,
    /// Stable numeric properties returned by `MallocExtension::GetProperties()`.
    pub properties: AllocatorProperties,
    /// Parsed `MALLOC:` byte counters from the human report.
    pub malloc: Vec<MemoryStat>,
    /// Parsed `TOTAL:` byte counters from the human report.
    pub total_process: Vec<MemoryStat>,
    /// Parsed `MALLOC EXPERIMENTS:` assignments.
    pub experiments: Vec<NamedValue>,
    /// Parsed `MALLOC HOOKS:` assignments.
    pub hooks: Vec<HookState>,
    /// Logical tcmalloc page size when reported.
    pub tcmalloc_page_size: Option<u64>,
    /// Logical tcmalloc hugepage size when reported.
    pub tcmalloc_hugepage_size: Option<u64>,
}

/// One memory counter from the human stats report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryStat {
    /// Optional report sign such as `+` or `=`.
    pub sign: Option<char>,
    /// Counter value in bytes.
    pub bytes: u64,
    /// Human-readable counter label.
    pub description: String,
}

/// `NAME=value` pair parsed from report sections.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedValue {
    /// Assignment key.
    pub name: String,
    /// Assignment value.
    pub value: u64,
}

/// Hook state parsed from `MALLOC HOOKS`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookState {
    /// Hook name from the human stats report.
    pub name: String,
    /// Whether the hook is enabled.
    pub enabled: bool,
}

/// Complete property map returned by `MallocExtension::GetProperties()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllocatorProperties {
    /// Every raw property reported by upstream.
    pub raw: Vec<AllocatorProperty>,
    /// Cross-allocator properties under the `generic.*` namespace.
    pub generic: GenericProperties,
    /// `TCMalloc`-specific properties under the `tcmalloc.*` namespace.
    pub tcmalloc: TcmallocProperties,
    /// Enabled upstream experiments reported as `tcmalloc.experiment.*`.
    pub experiments: Vec<AllocatorProperty>,
}

/// One `GetProperties()` entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllocatorProperty {
    /// Property name.
    pub name: String,
    /// Property value.
    pub value: u64,
}

/// Cross-allocator properties.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GenericProperties {
    /// `generic.current_allocated_bytes`, or the newer
    /// `generic.bytes_in_use_by_app` when only that key is present.
    pub current_allocated_bytes: Option<u64>,
    /// `generic.bytes_in_use_by_app` when reported.
    pub bytes_in_use_by_app: Option<u64>,
    /// `generic.heap_size`, when reported.
    pub heap_size: Option<u64>,
    /// `generic.physical_memory_used`, when reported.
    pub physical_memory_used: Option<u64>,
    /// `generic.virtual_memory_used`, when reported.
    pub virtual_memory_used: Option<u64>,
}

/// TCMalloc-specific properties.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TcmallocProperties {
    /// Maximum whole-process thread cache size in bytes.
    pub max_total_thread_cache_bytes: Option<u64>,
    /// Current whole-process thread cache bytes.
    pub current_total_thread_cache_bytes: Option<u64>,
    /// Free bytes in per-CPU caches.
    pub cpu_free: Option<u64>,
    /// Free bytes in thread caches.
    pub thread_cache_free: Option<u64>,
    /// Bytes held by transfer caches.
    pub transfer_cache: Option<u64>,
    /// Free bytes in central caches.
    pub central_cache_free: Option<u64>,
    /// Free bytes in the page heap.
    pub page_heap_free: Option<u64>,
    /// Unmapped bytes in the page heap.
    pub page_heap_unmapped: Option<u64>,
    /// Legacy spelling for page heap free bytes, when reported.
    pub pageheap_free_bytes: Option<u64>,
    /// Legacy spelling for page heap unmapped bytes, when reported.
    pub pageheap_unmapped_bytes: Option<u64>,
    /// Bytes used by allocator metadata.
    pub metadata_bytes: Option<u64>,
    /// Number of thread caches in use.
    pub thread_cache_count: Option<u64>,
    /// Whether per-CPU caches are active, when reported.
    pub per_cpu_caches_active: Option<bool>,
}

/// Returns typed tcmalloc statistics.
#[must_use]
pub fn malloc_stats() -> MallocStats {
    let raw = ffi::stats();
    let properties = properties();
    parse_malloc_stats(raw, properties)
}

/// Returns typed tcmalloc properties.
#[must_use]
pub fn properties() -> AllocatorProperties {
    parse_properties(&ffi::properties())
}

/// Returns one numeric property by name.
///
/// This accepts every key supported by `MallocExtension::GetNumericProperty`,
/// for example `generic.current_allocated_bytes` or
/// `tcmalloc.pageheap_free_bytes`.
#[must_use]
pub fn numeric_property(name: &str) -> Option<u64> {
    u64::try_from(ffi::numeric_property(name)).ok()
}

impl AllocatorProperties {
    /// Returns a raw property by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<u64> {
        self.raw
            .iter()
            .find(|property| property.name == name)
            .map(|property| property.value)
    }
}

fn parse_malloc_stats(raw: String, properties: AllocatorProperties) -> MallocStats {
    let mut malloc = Vec::new();
    let mut total_process = Vec::new();
    let mut experiments = Vec::new();
    let mut hooks = Vec::new();
    let mut tcmalloc_page_size = None;
    let mut tcmalloc_hugepage_size = None;

    for line in raw.lines() {
        if let Some(stat) = parse_memory_stat(line, "MALLOC:") {
            if stat.description == "Tcmalloc page size" {
                tcmalloc_page_size = Some(stat.bytes);
            } else if stat.description == "Tcmalloc hugepage size" {
                tcmalloc_hugepage_size = Some(stat.bytes);
            }
            malloc.push(stat);
        } else if let Some(stat) = parse_memory_stat(line, "TOTAL:") {
            total_process.push(stat);
        } else if let Some(values) = line.strip_prefix("MALLOC EXPERIMENTS:") {
            experiments.extend(parse_assignments(values));
        } else if let Some(values) = line.strip_prefix("MALLOC HOOKS:") {
            hooks.extend(
                parse_assignments(values)
                    .into_iter()
                    .map(|value| HookState {
                        name: value.name,
                        enabled: value.value != 0,
                    }),
            );
        }
    }

    MallocStats {
        raw,
        properties,
        malloc,
        total_process,
        experiments,
        hooks,
        tcmalloc_page_size,
        tcmalloc_hugepage_size,
    }
}

fn parse_properties(raw: &str) -> AllocatorProperties {
    let mut properties = Vec::new();
    for line in raw.lines() {
        let Some((name, value)) = line.split_once('\t') else {
            continue;
        };
        let Ok(value) = value.parse::<u64>() else {
            continue;
        };
        properties.push(AllocatorProperty {
            name: name.to_string(),
            value,
        });
    }

    let lookup = |name: &str| -> Option<u64> {
        properties
            .iter()
            .find(|property| property.name == name)
            .map(|property| property.value)
    };

    let current_allocated_bytes =
        lookup("generic.current_allocated_bytes").or_else(|| lookup("generic.bytes_in_use_by_app"));
    let bytes_in_use_by_app = lookup("generic.bytes_in_use_by_app");
    let heap_size = lookup("generic.heap_size");
    let physical_memory_used = lookup("generic.physical_memory_used");
    let virtual_memory_used = lookup("generic.virtual_memory_used");
    let max_total_thread_cache_bytes = lookup("tcmalloc.max_total_thread_cache_bytes");
    let current_total_thread_cache_bytes = lookup("tcmalloc.current_total_thread_cache_bytes");
    let cpu_free = lookup("tcmalloc.cpu_free");
    let thread_cache_free = lookup("tcmalloc.thread_cache_free");
    let transfer_cache = lookup("tcmalloc.transfer_cache");
    let central_cache_free = lookup("tcmalloc.central_cache_free");
    let page_heap_free = lookup("tcmalloc.page_heap_free");
    let page_heap_unmapped = lookup("tcmalloc.page_heap_unmapped");
    let pageheap_free_bytes = lookup("tcmalloc.pageheap_free_bytes");
    let pageheap_unmapped_bytes = lookup("tcmalloc.pageheap_unmapped_bytes");
    let metadata_bytes = lookup("tcmalloc.metadata_bytes");
    let thread_cache_count = lookup("tcmalloc.thread_cache_count");
    let per_cpu_caches_active = lookup("tcmalloc.per_cpu_caches_active").map(|value| value != 0);
    let experiments = properties
        .iter()
        .filter(|property| property.name.starts_with("tcmalloc.experiment."))
        .cloned()
        .collect();

    AllocatorProperties {
        raw: properties,
        generic: GenericProperties {
            current_allocated_bytes,
            bytes_in_use_by_app,
            heap_size,
            physical_memory_used,
            virtual_memory_used,
        },
        tcmalloc: TcmallocProperties {
            max_total_thread_cache_bytes,
            current_total_thread_cache_bytes,
            cpu_free,
            thread_cache_free,
            transfer_cache,
            central_cache_free,
            page_heap_free,
            page_heap_unmapped,
            pageheap_free_bytes,
            pageheap_unmapped_bytes,
            metadata_bytes,
            thread_cache_count,
            per_cpu_caches_active,
        },
        experiments,
    }
}

fn parse_memory_stat(line: &str, prefix: &str) -> Option<MemoryStat> {
    let mut rest = line.strip_prefix(prefix)?.trim_start();
    if rest.starts_with('-') {
        return None;
    }

    let sign = match rest.as_bytes().first().copied() {
        Some(b'+' | b'=') => {
            let sign = rest.as_bytes()[0] as char;
            rest = rest[1..].trim_start();
            Some(sign)
        }
        _ => None,
    };

    let digits = rest
        .char_indices()
        .take_while(|(_, ch)| ch.is_ascii_digit())
        .map(|(idx, ch)| idx + ch.len_utf8())
        .last()?;
    if digits == 0 {
        return None;
    }

    let bytes = rest[..digits].parse().ok()?;
    let mut description = rest[digits..].trim_start();
    if let Some(after_unit) = description
        .strip_prefix('(')
        .and_then(|value| value.split_once(')'))
    {
        description = after_unit.1.trim_start();
    }

    if description.is_empty() {
        return None;
    }

    Some(MemoryStat {
        sign,
        bytes,
        description: description.to_string(),
    })
}

fn parse_assignments(values: &str) -> Vec<NamedValue> {
    values
        .split_whitespace()
        .filter_map(|assignment| {
            let (name, value) = assignment.split_once('=')?;
            let value = value.parse().ok()?;
            Some(NamedValue {
                name: name.to_string(),
                value,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{parse_malloc_stats, parse_memory_stat, parse_properties};

    #[test]
    fn parses_memory_stat_with_unit() {
        let stat = parse_memory_stat(
            "MALLOC: + 827129856 ( 788.8 MiB) Bytes in page heap freelist",
            "MALLOC:",
        )
        .unwrap();
        assert_eq!(stat.sign, Some('+'));
        assert_eq!(stat.bytes, 827_129_856);
        assert_eq!(stat.description, "Bytes in page heap freelist");
    }

    #[test]
    fn parses_memory_stat_without_unit() {
        let stat = parse_memory_stat("MALLOC: 32768 Tcmalloc page size", "MALLOC:").unwrap();
        assert_eq!(stat.sign, None);
        assert_eq!(stat.bytes, 32_768);
        assert_eq!(stat.description, "Tcmalloc page size");
    }

    #[test]
    fn parses_properties_into_typed_groups() {
        let properties = parse_properties(
            "generic.current_allocated_bytes\t123\n\
             generic.physical_memory_used\t456\n\
             tcmalloc.per_cpu_caches_active\t1\n\
             tcmalloc.cpu_free\t64\n\
             tcmalloc.experiment.TEST\t1\n",
        );

        assert_eq!(properties.generic.current_allocated_bytes, Some(123));
        assert_eq!(properties.generic.physical_memory_used, Some(456));
        assert_eq!(properties.tcmalloc.per_cpu_caches_active, Some(true));
        assert_eq!(properties.tcmalloc.cpu_free, Some(64));
        assert_eq!(properties.experiments.len(), 1);
    }

    #[test]
    fn parses_malloc_stats_report_sections() {
        let properties = parse_properties("generic.current_allocated_bytes\t1\n");
        let stats = parse_malloc_stats(
            "MALLOC: 1024 ( 1.0 KiB) Bytes in use by application\n\
             MALLOC: 32768 Tcmalloc page size\n\
             TOTAL: 4096 ( 4.0 KiB) Bytes resident (physical memory used)\n\
             MALLOC EXPERIMENTS: TCMALLOC_TEMERAIRE=1\n\
             MALLOC HOOKS: NEW=0 DELETE=1\n"
                .to_string(),
            properties,
        );

        assert_eq!(stats.malloc.len(), 2);
        assert_eq!(stats.total_process.len(), 1);
        assert_eq!(stats.tcmalloc_page_size, Some(32_768));
        assert_eq!(stats.experiments[0].name, "TCMALLOC_TEMERAIRE");
        assert!(!stats.hooks[0].enabled);
        assert!(stats.hooks[1].enabled);
    }
}
