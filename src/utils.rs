//! Utilities used throughout the application.

use tracing_subscriber::EnvFilter;

/// Returns the name of the host operating system.
///
/// This function evaluates at **compile time** using `cfg!` macros.
/// It returns one of the following static strings:
///
/// - `"macos"` if compiled for macOS
/// - `"windows"` if compiled for Windows
/// - `"linux"` otherwise (covers Linux and other Unix-like OSes)
pub(crate) fn host_os() -> &'static str {
    if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(windows) {
        "windows"
    } else {
        "linux"
    }
}

/// Initializes global tracing configuration.
///
/// This sets up a [`tracing_subscriber`] that:
/// - Formats logs as **structured JSON**
/// - Applies an [`EnvFilter`] from the `RUST_LOG` environment variable
/// - Attaches information about:
///   - The current span
///   - Span hierarchy
///   - Thread IDs
///   - Thread names
///
/// Call this function once at the start of your program before
/// emitting any tracing events.
///
/// # Panics
///
/// This function will panic if a global default subscriber
/// has already been set.
pub fn init_tracing() {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(EnvFilter::from_default_env())
        .with_current_span(true)
        .with_span_list(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .init();
}
