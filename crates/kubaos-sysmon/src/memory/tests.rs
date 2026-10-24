use super::parse_meminfo;

const GIB: u64 = 1024 * 1024 * 1024;

const MEMINFO: &str = "\
MemTotal:       33554432 kB
MemFree:         8388608 kB
MemAvailable:   20971520 kB
Buffers:         1048576 kB
Cached:          9437184 kB
SwapCached:            0 kB
Shmem:           1048576 kB
SReclaimable:    1048576 kB
SwapTotal:       8388608 kB
SwapFree:        6291456 kB
";

#[test]
fn ram_splits_into_used_cache_and_available() {
    let memory = parse_meminfo(MEMINFO).expect("memory");

    assert_eq!(memory.total, 32 * GIB);
    assert_eq!(memory.available, 20 * GIB);
    assert_eq!(memory.used, 12 * GIB);
    // Buffers, page cache and reclaimable slab, less shared memory, which can't be dropped.
    assert_eq!(memory.cache, 10 * GIB);
    assert!((memory.percent - 37.5).abs() < 0.01);
}

#[test]
fn swap_used_is_what_is_not_free() {
    let memory = parse_meminfo(MEMINFO).expect("memory");

    assert_eq!(memory.swap_total, 8 * GIB);
    assert_eq!(memory.swap_used, 2 * GIB);
}

#[test]
fn the_cache_never_counts_more_than_is_available() {
    let memory = parse_meminfo("MemTotal: 1000 kB\nMemAvailable: 100 kB\nCached: 900 kB\n").expect("memory");

    assert_eq!(memory.cache, 100 * 1024);
}

#[test]
fn a_file_without_a_total_is_no_memory() {
    assert!(parse_meminfo("MemFree: 12 kB\n").is_none());
}
