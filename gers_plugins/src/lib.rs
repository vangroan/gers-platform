//! gers modding framework
use std::{
    fs::File,
    io::prelude::*,
    path::{Path, PathBuf},
};
use wasmer_compiler_cranelift::Cranelift;
use wasmer_engine_universal::Universal;

mod builtins;
mod env;
mod errors;
mod meta;

use env::GersEnv;
use errors::PluginError;
use meta::PluginMeta;

/// Name of the plugin definition meta file.
const PLUGIN_FILENAME: &str = "plugin.toml";

/// Name of WebAssembly module file to load.
const PLUGIN_WASM_MODULE: &str = "main.wat";

/// Registry of instantiated plugin modules.
pub struct Plugins {
    /// Keeps a around to be cloned into
    /// new module instances.
    // logger: slog::Logger,
    plugins: Vec<Plugin>,
    store: wasmer::Store,
    // TODO: Env should be decoupled from plugin infrastructure
    env: GersEnv,
    // TODO: Import object of engine API and builtins
}

pub struct Plugin {
    instance: wasmer::Instance,
    meta: PluginMeta,
    update_fn: Option<wasmer::Function>,
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

        // TODO: Env should be decoupled from plugin registry
        let env = GersEnv {
            memory: wasmer::LazyInit::default(),
        };

        Plugins {
            // logger,
            plugins: vec![],
            store,
            env,
        }
    }

    /// Iterate the plugins in execution order.
    #[inline(always)]
    pub fn iter_plugins(&self) -> impl Iterator<Item = &Plugin> {
        self.plugins.iter()
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
        let update_fn = match instance.exports.get_function("__gers_update") {
            Ok(func) => Some(func.clone()),
            Err(wasmer::ExportError::Missing(..)) => None,
            Err(wasmer::ExportError::IncompatibleType) => return Err(PluginError::FunctionType),
        };

        self.plugins.push(Plugin {
            instance,
            meta: plugin_meta,
            update_fn,
        });

        Ok(())
    }

    /// Load a WebAssembly module file and instantiate it into an instance.
    fn load_wasm(&self, module_path: impl AsRef<Path>) -> Result<wasmer::Instance, PluginError> {
        let mut file = File::open(module_path)?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;

        let module = wasmer::Module::new(&self.store, buf.as_str())?;

        // TODO: Build import object according to dependencies in meta file
        let import_object = builtins::create_builtins(&self.store, &self.env);
        let instance = wasmer::Instance::new(&module, &import_object)?;

        Ok(instance)
    }
}

impl Plugin {
    pub fn instance(&self) -> &wasmer::Instance {
        &self.instance
    }

    pub fn meta(&self) -> &PluginMeta {
        &self.meta
    }

    pub fn update_fn(&self) -> Option<&wasmer::Function> {
        self.update_fn.as_ref()
    }
}

#[cfg(test)]
mod test_plugin {
    use super::*;
    use wasmer::WasmerEnv;

    #[derive(WasmerEnv, Clone)]
    struct Env {}

    #[test]
    fn test_load_plugin() {
        let mut plugins = Plugins::new();
        plugins.load_plugin_dir("test").unwrap();
        assert_eq!(plugins.plugins.len(), 1);
        let plugin = &plugins.plugins[0];
        assert_eq!(plugin.meta.name.as_str(), "test-plugin");
        assert_eq!(plugin.meta.version.as_str(), "1.0.0");
    }
}
