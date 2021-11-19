//! gers executable application
use gers_plugins::Plugins;
use slog::{error, info, Drain};
use std::time::{Duration, Instant};
use winit::{
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod env;
mod error;
mod fps;
mod wasm_api;
mod wasm_impl;

use fps::{FpsCounter, FpsThrottle, FpsThrottlePolicy};

fn main() {
    // Logger
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).chan_size(4096).build().fuse();
    let root = slog::Logger::root(drain, slog::o!());
    let logger = root.new(slog::o!("lang" => "Rust"));

    let _scope_guard = slog_scope::set_global_logger(logger.clone());
    let _log_guard = slog_stdlog::init_with_level(log::Level::Info).unwrap();

    // Wasmer Environment
    let gers_env = env::GersEnv {
        logger: root.new(slog::o!("lang" => "Wasm")),
        timing: Default::default(),
        memory: Default::default(),
    };

    // Plugin Infrastructure
    let mut plugins = Plugins::new();

    // WebAssembly API
    let import_object = wasm_api::generate_import_object(plugins.store(), &gers_env);
    plugins.set_imports(import_object);

    // Walk plugin directory and load
    let mut plugin_dir = std::env::current_dir().expect("getting current working directory");
    plugin_dir.extend(&["plugins", "core"]);
    info!(logger, "Loading plugins from directory: {:?}", plugin_dir);

    if let Err(err) = plugins.load_plugin_dir(plugin_dir) {
        error!(logger, "failed loading plugin from directory: {}", err);
        return;
    }

    // Frame Timing
    let mut fps_throttle = FpsThrottle::new(144, FpsThrottlePolicy::Yield);
    let mut fps_counter = FpsCounter::new();
    let mut last_time = Instant::now();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("gers - 0 FPS 0.00ms")
        .build(&event_loop)
        .unwrap();

    use winit::event::{Event as E, WindowEvent as WE};

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            E::NewEvents(_) => {
                // Boundary where frame starts.
                let now = Instant::now();
                let mut delta_time = now - last_time;
                last_time = now;

                // Because delta time is frequently used as
                // a divisor, we want to avoid divide by zero.
                if delta_time.is_zero() {
                    delta_time = Duration::from_secs_f32(std::f32::EPSILON);
                }

                fps_counter.add(delta_time);

                // Store timings for access from WASm modules.
                let mut lock = gers_env
                    .timing
                    .write()
                    .expect("write access to timings lock");
                lock.delta_time = delta_time;
            }
            E::MainEventsCleared => {
                // Logic update here

                // Write FPS to window title
                let fps = fps_counter.fps();
                let dt = 1000.0 / fps; // milliseconds
                window.set_title(&format!("gers - {:.0} FPS {:.2}ms", fps, dt));

                // Dispatch to plugins
                for plugin in plugins.iter_plugins() {
                    if let Some(update_fn) = plugin.update_fn() {
                        if let Err(err) = update_fn.call(&[]) {
                            error::print_runtime_error(&logger, &err);
                        }
                    }
                }
            }
            E::RedrawRequested(window_id) if window_id == window.id() => {
                // TODO: Render here
            }
            E::RedrawEventsCleared => {
                // Emitted after all redraw events have been emitted,
                // before control will be taken away from the program.
                //
                // Frame cleanup can happen here.
                fps_throttle.throttle(last_time);
            }
            E::WindowEvent { event, window_id } if window_id == window.id() => match event {
                WE::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WE::KeyboardInput { .. } => {}
                WE::MouseInput { .. } => {}
                WE::Resized(..) => {}
                WE::ScaleFactorChanged { .. } => {}
                _ => {}
            },
            _ => (),
        }
    });
}
