use gers_api::bump::BumpAllocator;
use gers_events::*;

#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum gers_error_t {
    Success = 0,
    GenericError = 1,
}

/// As per the `ImportObject` used when creating the module instance.
#[link(wasm_import_module = "gers")]
extern "C" {
    fn log_info(str_ptr: *const u8, str_len: u32);
    fn get_delta_time() -> f32;
}

#[no_mangle]
pub extern "C" fn __gers_update() {
    unsafe {
        let message = "Hello, Mod!";
        log_info(message.as_bytes().as_ptr(), message.len() as u32);

        let delta_time = get_delta_time();
        let message = &format!("delta_time: {}", delta_time);
        log_info(message.as_bytes().as_ptr(), message.len() as u32);
    }
}

/// Safe shim for printing a log message.
///
/// # Safety
///
/// As long as the host doesn't retain the passed pointer, the
/// borrow should keep the string pointer and data in place and
/// unmutated for the duration of the call.
pub fn log(message: &str) {
    unsafe {
        log_info(message.as_bytes().as_ptr(), message.len() as u32);
    }
}

// FIXME: This should be a raw allocation.
/// Linear continious memory space for storing event data as
/// part of the event protocol.
///
/// We're relying on the WebAssembly module itself not implementing
/// threading. All guest code invoked by the host should be
/// single threaded, so global state like this shouldn't need
/// synchronising.
static mut EVENT_DATA: Vec<u8> = Vec::new();

/// Allocate space in the module's memory for storing event data.
///
/// The WebAssembly module owns its memory, so the host must be
/// polite and request space so the module can reserve it.
///
///
/// On success returns a pointer to the start of the allocated
/// memory. On failure returns a null pointer.
///
/// # Safety
///
/// The host must be careful retaining this pointer. If this function
/// is called again, any existing pointers will be invalid, and will
/// lead to move-after-free type bugs.
#[no_mangle]
pub unsafe extern "C" fn __gers_event_alloc(size: u32) -> *mut u8 {
    // TODO: Initialise global bump allocator.
    #[allow(unused_must_use)]
    if let Ok(mut bump) = BumpAllocator::new() {
        bump.alloc_aligned(16);
    }

    EVENT_DATA.resize(size as usize, 0);

    // SAFETY: The vector is global and will outlive this
    //         pointer, unless this function is called again.
    //         The onus is on the host to track the lifetime
    //         of the pointer and the calls to this function.
    EVENT_DATA.as_mut_ptr()
}

/// Update hook for host driven events.
///
/// The `event_type` identifies the concrete type of the event data
/// as per the host's API. It can be used to downcast the data and
/// dispatch to a strongly typed handler.
#[no_mangle]
pub extern "C" fn __gers_event_update(event_type: i32, data_ptr: *const u8) -> gers_error_t {
    match event_type.into() {
        EventType::NoOp => gers_error_t::Success,
        EventType::Hello => {
            if data_ptr.is_null() {
                log("data pointer is null");
                return gers_error_t::GenericError;
            }

            unsafe {
                // FIXME: Length and range checks on data buffer.

                // Convert raw data pointer to
                let offset = data_ptr.offset_from(EVENT_DATA.as_ptr());

                // When negative the pointer is before buffer.
                if offset < 0 {
                    log("data pointer is before memory buffer");
                    return gers_error_t::GenericError;
                }
                let index = offset as usize;
                let payload_size = std::mem::size_of::<HelloEvent>();

                let (_, data, _) = EVENT_DATA[index..payload_size].align_to::<HelloEvent>();
                if data.is_empty() {
                    log("data could not be transmuted");
                    return gers_error_t::GenericError;
                }

                hello_handler(&data[0]);
            }

            gers_error_t::Success
        }
    }
}

fn hello_handler(data: &HelloEvent) {
    log(format!("received event: {:?}", data).as_str());
}
