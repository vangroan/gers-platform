//! Builtin function imports.
use crate::env::GersEnv;
use wasmer::{imports, Array, Function, ImportObject, RuntimeError, Store, WasmPtr};

/// Create an import object containing built-in functions.
pub fn create_builtins(store: &Store, env: &GersEnv) -> ImportObject {
    imports! {
        "env" => {
            "print" => Function::new_native_with_env(store, env.clone(), print),
        }
    }
}

fn print(env: &GersEnv, str_ptr: WasmPtr<u8, Array>, str_len: u32) -> Result<(), RuntimeError> {
    // SAFETY: The underlying memory must not be mutated while we hold this reference.
    let maybe_str = env
        .memory
        .get_ref()
        .and_then(|mem| unsafe { str_ptr.get_utf8_str(mem, str_len) });
    if let Some(message) = maybe_str {
        println!("{}", message);
    }

    Ok(())
}
