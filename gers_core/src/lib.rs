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
