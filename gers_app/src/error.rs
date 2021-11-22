use slog::{error, Logger};
use wasmer::RuntimeError;

/// Utility for printing a `RuntimeError`.
pub fn print_runtime_error(logger: &Logger, err: &RuntimeError) {
    let mut message = String::new();
    message.push_str(err.message().as_str());
    message.push('\n');

    let frames = err.trace();
    let frames_len = frames.len();

    for (i, frame) in frames.iter().enumerate() {
        let frame_message = format!(
            "  Frame #{}: {:?}::{:?}\n",
            frames_len - i,
            frame.module_name(),
            frame.function_name().or(Some("<func>")).unwrap()
        );

        message.push_str(frame_message.as_str());
    }

    error!(logger, "runtime error: {}", message);
}
