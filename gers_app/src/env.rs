use slog::Logger;
use std::{
    sync::{Arc, RwLock},
    time::Duration,
};
use wasmer::{LazyInit, Memory, WasmerEnv};

#[derive(WasmerEnv, Clone)]
pub struct GersEnv {
    pub logger: Logger,
    pub timing: Arc<RwLock<Timing>>,

    #[wasmer(export)]
    pub memory: LazyInit<Memory>,
}

/// Event loop timing information.
pub struct Timing {
    /// Variable delta time since last event loop iteration.
    pub delta_time: Duration,
}

impl Default for Timing {
    fn default() -> Self {
        Timing {
            delta_time: Duration::from_secs_f32(std::f32::EPSILON),
        }
    }
}
