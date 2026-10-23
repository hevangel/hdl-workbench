//! Bump allocator for per-tick and per-block allocations.
//!
//! This module provides a simple arena allocator that allocates memory in a
//! contiguous buffer and resets it efficiently without deallocating individual
//! objects. This is ideal for short-lived allocations that are all freed at once
//! (e.g., at the end of a simulation tick).
//!
//! # Usage
//!
//! ```
//! use xezim::compiler::arena::Arena;
//!
//! // Create arena at start of tick
//! let mut arena = Arena::with_capacity(1024 * 1024);  // 1MB
//!
//! // Allocate objects
//! let val = arena.alloc(42_u64);
//! let vec = arena.alloc_vec::<u32>(100);
//!
//! // At end of tick: reset (no deallocation)
//! drop(vec);
//! let _ = val;
//! arena.reset();
//! ```

use std::cell::Cell;
use std::marker::PhantomData;
use std::mem::{align_of, size_of};
use std::ptr::NonNull;

/// Minimum alignment for all allocations (matches mimalloc's alignment)
const MIN_ALIGN: usize = 16;

/// Arena allocator for bump-allocated objects.
///
/// All allocations are aligned to `MIN_ALIGN` bytes. The arena can be
/// reset in O(1) time, making it ideal for per-tick allocations.
pub struct Arena {
    /// Pre-allocated buffer
    buffer: Vec<u8>,
    /// Current offset into buffer
    offset: Cell<usize>,
}

impl Arena {
    /// Create a new arena with the given initial capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            offset: Cell::new(0),
        }
    }

    /// Create a new arena with default capacity (64KB).
    pub fn new() -> Self {
        Self::with_capacity(64 * 1024)
    }

    /// Reset the arena, allowing reuse of the allocated buffer.
    /// This is O(1) and does not deallocate any memory.
    #[inline]
    pub fn reset(&self) {
        self.offset.set(0);
    }

    /// Get the current allocation offset.
    #[inline]
    pub fn offset(&self) -> usize {
        self.offset.get()
    }

    /// Get the capacity of the arena.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
    }

    /// Allocate raw bytes with the given alignment.
    /// Returns None if there's not enough space.
    #[inline]
    fn alloc_raw(&self, size: usize, align: usize) -> Option<NonNull<u8>> {
        let align = align.max(MIN_ALIGN);
        let offset = self.offset.get();

        // Calculate aligned offset
        let aligned_offset = (offset + align - 1) & !(align - 1);

        // Check if we have enough space
        if aligned_offset + size > self.buffer.capacity() {
            return None;
        }

        // Update offset
        self.offset.set(aligned_offset + size);

        // Safety: We've verified the buffer has enough capacity
        unsafe {
            let ptr = self.buffer.as_ptr().add(aligned_offset);
            Some(NonNull::new_unchecked(ptr as *mut u8))
        }
    }

    /// Allocate an object of type T.
    /// Returns a mutable reference to the allocated object.
    ///
    /// # Panics
    /// Panics if there's not enough space in the arena.
    #[inline]
    pub fn alloc<T>(&self, value: T) -> &mut T {
        let size = size_of::<T>();
        let align = align_of::<T>();

        let ptr = self
            .alloc_raw(size, align)
            .expect("Arena allocation failed: out of space");

        // Write the value
        unsafe {
            let raw_ptr = ptr.as_ptr() as *mut T;
            std::ptr::write(raw_ptr, value);
            &mut *raw_ptr
        }
    }

    /// Allocate a default-initialized object of type T.
    #[inline]
    pub fn alloc_default<T: Default>(&self) -> &mut T {
        let value = T::default();
        self.alloc(value)
    }

    /// Allocate a slice of T values.
    /// Returns a mutable slice.
    #[inline]
    pub fn alloc_slice<T>(&self, len: usize) -> &mut [T] {
        let size = size_of::<T>() * len;
        let align = align_of::<T>();

        let ptr = self
            .alloc_raw(size, align)
            .expect("Arena slice allocation failed: out of space");

        // The slice is uninitialized - caller must initialize it
        unsafe { std::slice::from_raw_parts_mut(ptr.as_ptr() as *mut T, len) }
    }

    /// Allocate a slice and initialize with a value.
    #[inline]
    pub fn alloc_slice_init<T: Clone>(&self, len: usize, value: T) -> &mut [T] {
        let slice = self.alloc_slice::<T>(len);
        for item in slice.iter_mut() {
            *item = value.clone();
        }
        slice
    }

    /// Allocate a Vec and return it wrapped in an ArenaVec.
    /// This allows storing the Vec in the arena while providing a convenient API.
    #[inline]
    pub fn alloc_vec<T>(&self, _capacity: usize) -> ArenaVec<'_, T> {
        ArenaVec {
            arena: self,
            _marker: PhantomData,
        }
    }
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}

/// A vector-like type that allocates its elements from an arena.
///
/// This is useful for building up collections during a tick that will
/// be discarded at the end.
pub struct ArenaVec<'a, T> {
    arena: &'a Arena,
    _marker: PhantomData<T>,
}

impl<'a, T> ArenaVec<'a, T> {
    /// Push a value onto the vector.
    #[inline]
    pub fn push(&self, value: T) -> &mut T {
        self.arena.alloc(value)
    }

    /// Allocate a slice of values.
    #[inline]
    pub fn alloc_slice(&self, len: usize) -> &mut [T] {
        self.arena.alloc_slice(len)
    }
}

/// Guard that resets an arena when dropped.
///
/// Use this with RAII pattern to ensure arena is reset even on panic.
pub struct ArenaGuard<'a> {
    arena: &'a Arena,
    original_offset: usize,
}

impl<'a> ArenaGuard<'a> {
    pub fn new(arena: &'a Arena) -> Self {
        let original_offset = arena.offset();
        Self {
            arena,
            original_offset,
        }
    }
}

impl<'a> Drop for ArenaGuard<'a> {
    fn drop(&mut self) {
        self.arena.offset.set(self.original_offset);
    }
}

/// RAII-based arena scope.
///
/// Creates an arena guard that resets the arena when dropped.
/// This is the recommended way to use arenas to ensure they're always reset.
#[macro_export]
macro_rules! arena_scope {
    ($arena:expr_2021) => {
        $crate::compiler::arena::ArenaGuard::new(&$arena)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_basic() {
        let arena = Arena::with_capacity(1024);

        let v1 = arena.alloc(42u32);
        assert_eq!(*v1, 42);

        let v2 = arena.alloc(100u64);
        assert_eq!(*v2, 100);

        *v1 = 99;
        assert_eq!(*v1, 99);
    }

    #[test]
    fn test_arena_reset() {
        let arena = Arena::with_capacity(1024);

        let _v1 = arena.alloc(42u32);
        assert_eq!(arena.offset(), 4); // u32 is 4 bytes

        arena.reset();
        assert_eq!(arena.offset(), 0);

        let v2 = arena.alloc(99u32);
        assert_eq!(*v2, 99);
    }

    #[test]
    fn test_arena_slice() {
        let arena = Arena::with_capacity(1024);

        let slice = arena.alloc_slice::<u32>(10);
        assert_eq!(slice.len(), 10);

        for (i, v) in slice.iter_mut().enumerate() {
            *v = i as u32;
        }

        assert_eq!(slice[5], 5);
    }

    #[test]
    fn test_arena_guard() {
        let arena = Arena::with_capacity(1024);

        {
            let _guard = ArenaGuard::new(&arena);
            let v = arena.alloc(42u32);
            assert_eq!(*v, 42);
            assert!(arena.offset() > 0);
        }

        // Guard has been dropped, arena should be reset
        assert_eq!(arena.offset(), 0);
    }
}
