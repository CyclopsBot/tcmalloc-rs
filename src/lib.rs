#![deny(clippy::pedantic)]

//! ## Motivation
//!
//! This is a wrapper for the [tcmalloc](https://google.github.io/tcmalloc/) allocator.

//! ## Basic usage
//!
//! ```rust,ignore
//! #[global_allocator]
//! static ALLOC: tcmalloc::TcMalloc = tcmalloc::TcMalloc;
//! ```

use core::{
    alloc::GlobalAlloc,
    ffi::{c_char, c_void, CStr},
};

extern "C" {
    /// Allocate a block of memory of at least `size` bytes, aligned to the
    /// given alignment.
    pub fn TCMallocInternalMalloc(size: usize) -> *mut c_void;
    /// Allocate a block of memory of at least `size` bytes, aligned to the
    /// given alignment, and zeroed.
    pub fn TCMallocInternalCalloc(n: usize, size: usize) -> *mut c_void;
    /// Reallocate a block of memory to a new size.
    pub fn TCMallocInternalRealloc(p: *mut c_void, newsize: usize) -> *mut c_void;
    /// Free previously allocated memory.
    pub fn TCMallocInternalFree(p: *mut c_void);
    /// Enable GWP-ASan for improved memory safety.
    pub fn MallocExtension_Internal_ActivateGuardedSampling();
    /// Print global tcmalloc statistics summary.
    pub fn TCMallocInternalMallocStats();
    /// Print global tcmalloc statistics.
    pub fn get_stats() -> *const c_char;
}

/// Print statistics of the memory allocator.
///
/// This function will print statistics about memory usage to `stderr`.
#[must_use]
pub fn print_stats() -> String {
    let stats = unsafe { CStr::from_ptr(get_stats()) };
    String::from_utf8_lossy(stats.to_bytes()).to_string()
}

pub fn print_stats_summary() {
    unsafe {
        TCMallocInternalMallocStats();
    }
}

/// Activates GWP-Asan
///
/// [More info](https://github.com/google/tcmalloc/blob/master/docs/gwp-asan.md)
pub fn activate_guarded_sampling() {
    unsafe {
        MallocExtension_Internal_ActivateGuardedSampling();
    }
}

/// ## Usage
///
/// Inside of the `main.rs` for any binary:
///
/// ```rust,ignore
/// #[global_allocator]
/// static ALLOC: tcmalloc::TcMalloc = tcmalloc::TcMalloc;
/// ```
pub struct TcMalloc;

unsafe impl GlobalAlloc for TcMalloc {
    #[inline]
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        TCMallocInternalMalloc(layout.size()).cast::<u8>()
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: core::alloc::Layout) -> *mut u8 {
        TCMallocInternalCalloc(1, layout.size()).cast::<u8>()
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: core::alloc::Layout) {
        TCMallocInternalFree(ptr.cast::<c_void>());
    }

    #[inline]
    unsafe fn realloc(
        &self,
        ptr: *mut u8,
        _layout: core::alloc::Layout,
        new_size: usize,
    ) -> *mut u8 {
        TCMallocInternalRealloc(ptr.cast::<c_void>(), new_size).cast::<u8>()
    }
}

#[test]
fn free_malloc() {
    let ptr = unsafe { TCMallocInternalMalloc(8) } as *mut u8;
    unsafe { TCMallocInternalFree(ptr as *mut c_void) };
}

#[test]
fn free_calloc() {
    let ptr = unsafe { TCMallocInternalCalloc(1, 8) } as *mut u8;
    unsafe { TCMallocInternalFree(ptr as *mut c_void) };
}

#[test]
fn calloc_zeroed() {
    let ptr = unsafe { TCMallocInternalCalloc(1, 8) } as *mut u8;

    // Check if the memory is zeroed
    unsafe {
        for i in 0..8 {
            assert_eq!(*ptr.add(i), 0); // All bytes should be zero
        }
        // F`ree the memory
        TCMallocInternalFree(ptr as *mut c_void);
    }
}

#[test]
fn realloc() {
    let ptr = unsafe { TCMallocInternalMalloc(8) } as *mut u8;
    // Verify realloc preserves data
    unsafe {
        *ptr = 42;
        // Reallocate the block
        let new_ptr = TCMallocInternalRealloc(ptr as *mut c_void, 16) as *mut u8;
        // Check if the original data is preserved
        assert_eq!(*new_ptr, 42);
        // Free the memory
        TCMallocInternalFree(new_ptr as *mut c_void);
    }
}

#[test]
fn free_realloc() {
    let ptr = unsafe { TCMallocInternalMalloc(8) } as *mut u8;
    let ptr = unsafe { TCMallocInternalRealloc(ptr as *mut c_void, 16) } as *mut u8;
    unsafe { TCMallocInternalFree(ptr as *mut c_void) };
}
