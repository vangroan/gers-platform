use wasmer::{imports, Function, ImportObject, Store};

use crate::{env::GersEnv, wasm_impl};

#[rustfmt::skip]
pub fn generate_import_object(store: &Store, env: &GersEnv) -> ImportObject {
    imports! {
        "gers" => {
            "log_info"       => Function::new_native_with_env(store, env.clone(), wasm_impl::log_info),
            "get_delta_time" => Function::new_native_with_env(store, env.clone(), wasm_impl::get_delta_time),
        },
        "gers_event" => {
            
        }
    }
}
