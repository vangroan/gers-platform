//! gers modding framework
use std::{
    fs::File,
    io::prelude::*,
    path::{Path, PathBuf},
};
use wasmer::{Array, ChainableNamedResolver, ImportObject, NativeFunc, WasmPtr};
use wasmer_compiler_cranelift::Cranelift;
use wasmer_engine_universal::Universal;

// mod builtins;
mod errors;
mod meta;

use errors::PluginError;
use meta::PluginMeta;

/// Name of the plugin definition meta file.
const PLUGIN_FILENAME: &str = "plugin.toml";

/// Name of WebAssembly module file to load.
const PLUGIN_WASM_MODULE: &str = "main.wasm";

/// Helper to get function hooks out of module
/// when setting up a plugin.
macro_rules! get_func {
    ($exports:expr, $name:literal, $args:ty, $ret:ty) => {
        match $exports.get_function($name) {
            Ok(func) => match func.native::<$args, $ret>() {
                Ok(native_func) => Some(native_func.clone()),
                Err(err) => {
                    eprintln!("native function error: {}", err);
                    return Err(PluginError::FunctionType);
                }
            },
            Err(wasmer::ExportError::Missing(..)) => None,
            Err(wasmer::ExportError::IncompatibleType) => return Err(PluginError::FunctionType),
        }
    };
    ($exports:expr, $name:literal) => {
        match $exports.get_function($name) {
            Ok(func) => Some(func.clone()),
            Err(wasmer::ExportError::Missing(..)) => None,
            Err(wasmer::ExportError::IncompatibleType) => return Err(PluginError::FunctionType),
        }
    };
}

pub type EventInitFn = NativeFunc<(), i32>;
pub type EventResetfn = NativeFunc<(), i32>;
pub type EventAlloc2Fn = NativeFunc<u32, WasmPtr<u8, Array>>;
pub type EventAllocFn = NativeFunc<u32, WasmPtr<u8, Array>>;
pub type EventUpdateFn = NativeFunc<(i32, WasmPtr<u8, Array>), i32>;

/// Registry of instantiated plugin modules.
pub struct Plugins {
    /// Keeps a around to be cloned into
    /// new module instances.
    // logger: slog::Logger,
    plugins: Vec<Plugin>,
    store: wasmer::Store,
    imports: Option<ImportObject>,
    // TODO: Import object of engine API and builtins
}

pub struct Plugin {
    instance: wasmer::Instance,
    pub data_ptr: Option<WasmPtr<u8, Array>>,
    meta: PluginMeta,
    update_fn: Option<wasmer::Function>,
    pub event_init_fn: Option<EventInitFn>,
    pub event_reset_fn: Option<EventResetfn>,
    pub event_alloc2_fn: Option<EventAllocFn>,
    pub event_alloc_fn: Option<EventAllocFn>,
    pub event_update_fn: Option<EventUpdateFn>,
}

impl Default for Plugins {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugins {
    pub fn new() -> Self {
        let compiler = Cranelift::new();

        let store = wasmer::Store::new(&Universal::new(compiler).engine());

        Plugins {
            plugins: vec![],
            store,
            imports: None,
        }
    }

    pub fn store(&self) -> &wasmer::Store {
        &self.store
    }

    /// Push a resolver to the back of the resolver chain.
    pub fn set_imports(&mut self, imports: ImportObject) {
        self.imports = Some(imports);
    }

    /// Iterate the plugins in execution order.
    #[inline(always)]
    pub fn iter_plugins(&self) -> impl Iterator<Item = &Plugin> {
        self.plugins.iter()
    }

    #[inline(always)]
    pub fn iter_plugins_mut(&mut self) -> impl Iterator<Item = &mut Plugin> {
        self.plugins.iter_mut()
    }

    /// Load a plugin contained in a directory.
    pub fn load_plugin_dir(&mut self, dir_path: impl AsRef<Path>) -> Result<(), PluginError> {
        let mut meta_path = PathBuf::new();
        meta_path.push(dir_path.as_ref());
        meta_path.push(PLUGIN_FILENAME);

        let mut file = File::open(meta_path)?;

        let mut buf = String::new();
        file.read_to_string(&mut buf)?;

        let plugin_meta: PluginMeta = toml::from_str(buf.as_str())?;

        let mut wasm_path = PathBuf::new();
        wasm_path.push(dir_path);
        wasm_path.push(PLUGIN_WASM_MODULE);

        let instance = self.load_wasm(wasm_path)?;

        // TODO: Decouple calls from plugin module into event framework
        // Frame Update entry point
        // let update_fn = match instance.exports.get_function("__gers_update") {
        //     Ok(func) => Some(func.clone()),
        //     Err(wasmer::ExportError::Missing(..)) => None,
        //     Err(wasmer::ExportError::IncompatibleType) => return Err(PluginError::FunctionType),
        // };
        let update_fn = get_func!(instance.exports, "__gers_update");
        let event_init_fn = get_func!(instance.exports, "__gers_bump_init", (), i32);
        let event_reset_fn = get_func!(instance.exports, "__gers_bump_reset", (), i32);
        let event_alloc2_fn = get_func!(instance.exports, "__gers_bump_alloc", u32, WasmPtr<u8, Array>);
        let event_alloc_fn = get_func!(instance.exports, "__gers_event_alloc", u32, WasmPtr<u8, Array>);
        let event_update_fn = get_func!(instance.exports, "__gers_event_update", (i32, WasmPtr<u8, Array>), i32);

        self.plugins.push(Plugin {
            instance,
            data_ptr: None,
            meta: plugin_meta,
            update_fn,
            event_init_fn,
            event_reset_fn,
            event_alloc2_fn,
            event_alloc_fn,
            event_update_fn,
        });

        Ok(())
    }

    /// Load a WebAssembly module file and instantiate it into an instance.
    fn load_wasm(&self, module_path: impl AsRef<Path>) -> Result<wasmer::Instance, PluginError> {
        let mut file = File::open(module_path)?;
        let mut buf: Vec<u8> = vec![];
        file.read_to_end(&mut buf)?;

        let module = wasmer::Module::new(&self.store, buf)?;

        // TODO: Build import object according to dependencies in meta file
        let dependencies = wasmer::imports! {};

        // Host can provide built-in imports.
        let builtins = match self.imports {
            Some(ref builtins) => builtins.clone(),
            None => wasmer::imports! {},
        };

        // Module dependencies are resolved first.
        let chain = dependencies.chain_back(builtins);

        let instance = wasmer::Instance::new(&module, &chain)?;

        Ok(instance)
    }
}

impl Plugin {
    pub fn instance(&self) -> &wasmer::Instance {
        &self.instance
    }

    pub fn memory(&self) -> Result<&wasmer::Memory, wasmer::ExportError> {
        self.instance.exports.get_memory("memory")
    }

    pub fn meta(&self) -> &PluginMeta {
        &self.meta
    }

    pub fn update_fn(&self) -> Option<&wasmer::Function> {
        self.update_fn.as_ref()
    }

    pub fn event_alloc_fn(&self) -> Option<&EventAllocFn> {
        self.event_alloc_fn.as_ref()
    }

    pub fn event_update_fn(&self) -> Option<&EventUpdateFn> {
        self.event_update_fn.as_ref()
    }
}

#[cfg(test)]
mod test_plugin {
    use super::*;
    use wasmer::WasmerEnv;

    #[derive(WasmerEnv, Clone)]
    struct Env {}

    // FIXME: Loader should support both WAT and WASM
    // #[test]
    #[allow(dead_code)]
    fn test_load_plugin() {
        let mut plugins = Plugins::new();
        plugins.load_plugin_dir("test").unwrap();
        assert_eq!(plugins.plugins.len(), 1);
        let plugin = &plugins.plugins[0];
        assert_eq!(plugin.meta.name.as_str(), "test-plugin");
        assert_eq!(plugin.meta.version.as_str(), "1.0.0");
    }
}
