use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Create and start a spinner with the given message.
/// Call `.finish_and_clear()` or `.finish_with_message()` when done.
///
/// When stderr is not a TTY (e.g. CI), indicatif suppresses the spinner
/// animation automatically. The message is also emitted as an `info!` log
/// event so it always appears in the log file.
pub fn start(msg: impl Into<String>) -> ProgressBar {
    let msg = msg.into();
    tracing::info!("{}", msg);
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.set_message(msg);
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}
