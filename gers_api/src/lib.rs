pub mod bump;
pub mod hooks;

#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum gers_error_t {
    Success = 0,
    GenericError = 1,
    /// Request to an allocator has invalid arguments.
    BadAlloc,
    /// System has run out of memory.
    OutOfMemory = 11,
    /// Allocator has run out of memory space.
    NoSpace = 12,
    /// Allocation or reset was called, but the allocator has not been initialized.
    AllocUninitialized = 13,
}
