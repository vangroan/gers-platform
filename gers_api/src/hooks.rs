//! External functions to be called by host.
use crate::{
    bump::{BumpAllocator, BumpError},
    gers_error_t,
};

/// Linear continious memory space for storing event data as
/// part of the event protocol.
///
/// We're relying on the WebAssembly module itself not implementing
/// threading. All guest code invoked by the host should be
/// single threaded, so global state like this shouldn't need
/// synchronising.
///
/// When the WebAssembly threading proposal is implemented by
/// wasmer, this will need to be wrapped in `Mutex` or `RwLock`.
static mut EVENT_DATA: BumpAllocator = unsafe { BumpAllocator::uninit() };

/// Initialize the bump allocator for commands.
#[no_mangle]
unsafe extern "C" fn __gers_bump_init() -> gers_error_t {
    use BumpError as E;

    match EVENT_DATA.initialize() {
        Ok(_) => gers_error_t::Success,
        Err(E::BadRequest) => gers_error_t::BadAlloc,
        Err(E::OutOfMemory) => gers_error_t::OutOfMemory,
        Err(_) => gers_error_t::GenericError,
    }
}
