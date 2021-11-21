//! Bump allocator for host<->guest interop of data that doesn't fit in function arguments.
//!
//! # Resources
//!
//! - https://rust-hosted-langs.github.io/book/part-allocators.html
//! - https://os.phil-opp.com/heap-allocation/
//! - https://os.phil-opp.com/allocator-designs/
//! - https://os.phil-opp.com/kernel-heap/#alignment
use std::{alloc::Layout, ptr::NonNull};

/// Align the given memory address upwards to the given alignment.
///
/// Requires that `align` is power-of-two.
///
/// Implementation taken from: https://os.phil-opp.com/allocator-designs/#address-alignment
#[inline]
fn align_up(addr: usize, align: usize) -> usize {
    debug_assert!(align.is_power_of_two(), "alignment must be power-of-two");

    // This implementation works as follows:
    //
    // - Because `align` is power-of-two, its bit representation
    //   only has one bit set (0b1000)
    // - That means `align - 1` flips the lower bits (0b0111)
    // - By inverting it with bitwise NOT !(align - 1) the
    //   bits for `align` and up are set (0b11...1111000)
    // - By masking an address with bitwise AND the lower bits
    //   are cleared and the address is aligned *downwards*
    // - But we want to align upwards so we add `align - 1` to
    //   the address. Already aligned addresses stay the same,
    //   while unaligned addresses are aligned upwards.
    (addr + align - 1) & !(align - 1)
}

/// Allocate a new block of raw memory.
///
/// # Safety
///
/// Performs a memory allocation and returns raw pointers
/// that will not be automatically freed.
///
/// It is the responsibility of the caller to ensure that
/// the pointers are freed, and not used after free.
unsafe fn alloc(size: usize, align: usize) -> Result<NonNull<u8>, BumpError> {
    if align == 0 {
        return Err(BumpError::BadRequest);
    }

    // Architectures have hard or soft requirements that
    // memory access must be aligned to power-of-two.
    if !align.is_power_of_two() {
        return Err(BumpError::BadRequest);
    }

    // Size must not overflow maximum available memory.
    // Implementation taken from Layout::from_size_align()
    if size > usize::MAX - (align - 1) {
        return Err(BumpError::BadRequest);
    }

    let layout = Layout::from_size_align_unchecked(size, align);
    let ptr = std::alloc::alloc(layout);

    if ptr.is_null() {
        // `alloc` interface returns null on invalid
        // layout as well, but we covered the validations.
        return Err(BumpError::OutOfMemory);
    } else {
        Ok(NonNull::new_unchecked(ptr))
    }
}

/// Represents a raw memory allocation.
struct RawBlock {
    ptr: NonNull<u8>,
    size: usize,
}

impl RawBlock {
    /// Allocate a new block of raw memory.
    unsafe fn new(size: usize) -> Result<RawBlock, BumpError> {
        // When the block is aligned to its size, it's trivial to
        // use bitwise operations to find the block start and end
        // boundary for a given pointer within in.
        let ptr = alloc(size, size)?;

        Ok(RawBlock { ptr, size })
    }
}

impl Drop for RawBlock {
    fn drop(&mut self) {
        unsafe {
            // Layout invariants were validated on instantiation.
            let layout = Layout::from_size_align_unchecked(self.size, self.size);
            std::alloc::dealloc(self.ptr.as_ptr(), layout);
        }
    }
}

#[derive(Debug)]
pub enum BumpError {
    /// Request for allocation has invalid arguments.
    BadRequest,
    /// System has run out of memory.
    OutOfMemory,
    /// Bump allocator has reached the end of it space limit.
    NoSpace,
}

const BLOCK_SIZE_BITS: usize = 12; // 4KB
const BLOCK_SIZE: usize = 1 << BLOCK_SIZE_BITS;

pub struct BumpAllocator {
    /// Points to the end of the previously allocated object.
    /// Not guaranteed to be aligned.
    cursor: usize,
    block: RawBlock,
}

impl BumpAllocator {
    /// Maximum number of bytes limit that can be allocated.
    pub const MAX_SIZE: usize = BLOCK_SIZE;

    /// Create a new bump allocator with `BLOCK_SIZE`.
    pub fn new() -> Result<Self, BumpError> {
        // SAFETY: Raw allocation is owned by us
        //         and will be safely dropped.
        //         Handing out pointers to inside
        //         of the block is what will be unsafe.
        let block = unsafe { RawBlock::new(BLOCK_SIZE)? };

        Ok(Self { cursor: 0, block })
    }

    /// Allocate with default alignemnt of double machine word.
    ///
    /// # Safety
    ///
    /// Has the same requirements as [`BumpAllocator::alloc()`]
    pub unsafe fn alloc_aligned(&mut self, size: usize) -> Result<RawPtr, BumpError> {
        let line_size = std::mem::size_of::<usize>() * 2;
        self.alloc(size, line_size)
    }

    /// # Safety
    ///
    /// The memory pointed to by the resulting pointer is uninitialized.
    pub unsafe fn alloc(&mut self, size: usize, align: usize) -> Result<RawPtr, BumpError> {
        let aligned = align_up(self.block.ptr.as_ptr() as usize + self.cursor, align);

        // Cursor is offset from start of block and not memory address.
        let next = (aligned + size) - self.block.ptr.as_ptr() as usize;

        if next > self.block.size {
            return Err(BumpError::NoSpace);
        }

        // Commit
        self.cursor = next;
        let ptr = aligned as *mut u8;
        Ok(RawPtr {
            ptr: NonNull::new_unchecked(ptr),
        })
    }

    /// Clears the allocator's space, making [`BumpAllocator::MAX_SIZE`]
    /// available again.
    ///
    /// #Safety
    ///
    /// Any
    pub unsafe fn reset(&mut self) {
        self.cursor = 0;
    }
}

/// Wrapper for a raw pointer so its easier to use
/// and the internal representation can be changed
/// without introducing breakage.
pub struct RawPtr {
    ptr: NonNull<u8>,
}

impl RawPtr {
    /// Retrieve the inner raw pointer.
    ///
    /// # Safety
    ///
    /// The pointer should not outlive the [`BumpAllocator`].
    #[inline(always)]
    pub unsafe fn as_ptr(&self) -> *mut u8 {
        self.ptr.as_ptr()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_raw_allocation() {
        let size = 0x1000;

        unsafe {
            let block = RawBlock::new(size).unwrap();
            assert_eq!(block.size, size);
        }
    }

    #[test]
    fn test_bump_allocation() {
        let mut bump = BumpAllocator::new().unwrap();

        unsafe {
            let ptr = bump.alloc(16, 8).unwrap();
            assert_eq!(bump.cursor, 16);
            assert_eq!(ptr.as_ptr().offset_from(bump.block.ptr.as_ptr()), 0);

            let ptr = bump.alloc(16, 8).unwrap();
            assert_eq!(bump.cursor, 32);
            assert_eq!(ptr.as_ptr().offset_from(bump.block.ptr.as_ptr()), 16);
        }
    }
}
