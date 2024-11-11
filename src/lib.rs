#![no_std]

//! ## Motivation
//!
//! This is a wrapper for the [tcmalloc](https://google.github.io/tcmalloc/) allocator.

//! ## Basic usage
//!
//! ```rust,ignore
//! #[global_allocator]
//! static ALLOC: tcmalloc::TcMalloc = tcmalloc::TcMalloc;
//! ```

use core::{alloc::GlobalAlloc, ffi::c_void};

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
    /// Print global tcmalloc statistics.
    pub fn TCMallocInternalMallocStats();

}

/// Print statistics of the memory allocator.
///
/// This function will print statistics about memory usage to `stderr`.
pub fn stats_print() {
    unsafe {
        TCMallocInternalMallocStats();
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
        TCMallocInternalMalloc(layout.size()) as *mut u8
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: core::alloc::Layout) -> *mut u8 {
        TCMallocInternalCalloc(1, layout.size()) as *mut u8
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: core::alloc::Layout) {
        TCMallocInternalFree(ptr as *mut c_void)
    }

    #[inline]
    unsafe fn realloc(
        &self,
        ptr: *mut u8,
        _layout: core::alloc::Layout,
        new_size: usize,
    ) -> *mut u8 {
        TCMallocInternalRealloc(ptr as *mut c_void, new_size) as *mut u8
    }
}

#[test]
fn ok_free_malloc() {
    let ptr = unsafe { TCMallocInternalMalloc(8) } as *mut u8;
    unsafe { TCMallocInternalFree(ptr as *mut c_void) };
}

#[test]
fn ok_free_calloc() {
    let ptr = unsafe { TCMallocInternalCalloc(1, 8) } as *mut u8;
    unsafe { TCMallocInternalFree(ptr as *mut c_void) };
}

#[test]
fn ok_free_realloc() {
    let ptr = unsafe { TCMallocInternalMalloc(8) } as *mut u8;
    let ptr = unsafe { TCMallocInternalRealloc(ptr as *mut c_void, 16) } as *mut u8;
    unsafe { TCMallocInternalFree(ptr as *mut c_void) };
}

#[test]
fn print_stats() {
    // Here we can simply call the stats_print function.
    // Note that the output is directed to stderr, so we won't capture it in the test.
    // Instead, we will just ensure it doesn't panic.

    // Call the stats_print function
    stats_print();

    // If you have any way of checking specific conditions based on the state of your program,
    // you can add those assertions here. Since this just prints to stderr, we can't check output directly.
    // For now, we assume that calling the function is sufficient.
    // Optionally, you can assert that there were no allocation errors by doing allocations before.
    unsafe {
        let _ptr = TCMallocInternalMalloc(16);
        assert!(!_ptr.is_null(), "Failed to allocate memory.");
        TCMallocInternalFree(_ptr);
    }
}
