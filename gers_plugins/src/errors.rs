use thiserror::Error;

#[derive(Error, Debug)]
pub enum PluginError {
    #[error("failed to read plugin file: {0}")]
    LoadFile(#[from] std::io::Error),

    #[error("failed to deserialize file: {0}")]
    Deserialize(#[from] toml::de::Error),

    #[error("failed to compile WebAssembly: {0}")]
    Compile(#[from] wasmer::CompileError),

    #[error("failed to instantiate WebAssembly module: {0}")]
    Instantiate(#[from] wasmer::InstantiationError),

    #[error("module entrypoint function is incorrect type")]
    FunctionType,
}
