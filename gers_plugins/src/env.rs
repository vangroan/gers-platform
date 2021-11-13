//! `wasmer` host environemnt.
use wasmer::{LazyInit, Memory, WasmerEnv};

/// Contextual environment passed into host functions.
#[derive(WasmerEnv, Clone)]
pub struct GersEnv {
    #[wasmer(export)]
    pub memory: LazyInit<Memory>,
}
