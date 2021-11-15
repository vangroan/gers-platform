use crate::env::GersEnv;
use wasmer::{Array, WasmPtr};

pub fn log_info(env: &GersEnv, str_ptr: WasmPtr<u8, Array>, str_len: u32) {
    let maybe = env
        .memory
        .get_ref()
        // SAFETY: Underly  ing memory may not be mutated or grown while string is borrowed.
        .and_then(|mem| unsafe { str_ptr.get_utf8_str(mem, str_len) });

    if let Some(string) = maybe {
        slog::info!(env.logger, "{}", string);
    }
}

pub fn get_delta_time(env: &GersEnv) -> f32 {
    match env.timing.read() {
        Ok(ref timing) => timing.delta_time.as_secs_f32(),
        Err(_) => std::f32::EPSILON,
    }
}
