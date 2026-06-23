#[cfg(codspeed)]
extern crate divan as codspeed_divan_compat;

use divan::black_box;
use tcmalloc::{ProfileType, malloc_stats, profile_snapshot, properties, tuning_snapshot};

fn main() {
    divan::main();
}

#[divan::bench(sample_count = 20, max_time = 1)]
fn typed_stats_snapshot() {
    black_box(malloc_stats());
}

#[divan::bench(sample_count = 20, max_time = 1)]
fn property_snapshot() {
    black_box(properties());
}

#[divan::bench(sample_count = 20, max_time = 1)]
fn tuning_snapshot_read() {
    black_box(tuning_snapshot());
}

#[divan::bench(sample_count = 10, max_time = 1)]
fn heap_profile_snapshot() {
    black_box(profile_snapshot(black_box(ProfileType::Heap)).expect("heap profile"));
}
