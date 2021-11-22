use crate::{bump::BumpAllocator, hooks::EVENT_DATA};
use std::marker::PhantomData;

/// TODO: Take a slice of pointers into the allocator, and read multiple commands instead dof just one.
pub struct CmdReader<'a> {
    _global: &'static BumpAllocator,
    _marker: PhantomData<&'a BumpAllocator>,
}

impl<'a> CmdReader<'a> {
    pub unsafe fn new() -> Option<Self> {
        Some(Self {
            // SAFETY: This reader is inteded to run
            //         in a WebAssembly module which
            //         is single-threaded, until the
            //         threading proposal ever gets
            //         implemented.
            //         Only the host is intended to mutate
            //         the allocator, which it shouldn't
            //         do while the WASM module is executing.
            _global: &EVENT_DATA,
            _marker: Default::default(),
        })
    }

    pub fn read<T: Sized>(&self, ptr: *const u8) -> Option<&T> {
        if ptr.is_null() {
            return None;
        }

        // TODO: Range check pointer against allocator space bounds
        unsafe {
            let data: &[T] = std::slice::from_raw_parts(ptr as *const _, 1);

            if !data.is_empty() {
                Some(&data[0])
            } else {
                None
            }
        }
    }
}
