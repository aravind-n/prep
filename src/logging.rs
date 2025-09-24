//! Logging initialization and configuration.
//!
//! This module centralizes setup of [`tracing_subscriber`] for the application.
//! It provides:
//!
//! - A [`LogFormat`] enum describing supported output formats
//! - A [`LogConfig`] struct holding configuration options
//! - A [`init_tracing`] function that installs the global tracing subscriber
//!
//! By default, the log format is chosen automatically:
//! - **Pretty** when outputting to a terminal (human-readable)
//! - **JSON** when output is redirected (structured, machine-readable)
//!
//! The subscriber is configured with:
//! - Log level filtering via `RUST_LOG` (using [`EnvFilter`])
//! - Thread IDs and names
//! - Optional timestamps (`ChronoUtc` in RFC3339)
//! - Either stdout or stderr as the log sink
//!
//! # Panics
//!
//! Calling [`init_tracing`] after another subscriber has already been set
//! will return an error, since only one global subscriber may be active.

use std::io::IsTerminal;

use anyhow::{Result, anyhow};
use clap::ValueEnum;
use tracing_subscriber::{
    EnvFilter,
    fmt::{time::ChronoUtc, writer::BoxMakeWriter},
};

/// Supported log output formats.
///
/// This enum is exposed as a [`clap::ValueEnum`] so it can be parsed
/// directly from CLI arguments.
///
/// - `Auto`: choose automatically based on TTY detection  
/// - `Pretty`: human-readable logs  
/// - `Compact`: shorter human-readable logs  
/// - `Json`: structured logs suitable for ingestion by log processors
#[derive(Clone, Debug, ValueEnum)]
pub enum LogFormat {
    Auto,
    Pretty,
    Compact,
    Json,
}

/// Runtime configuration for logging.
///
/// This struct encapsulates all options needed to initialize
/// a tracing subscriber. It can be constructed manually or
/// parsed from CLI flags.
pub struct LogConfig {
    /// Desired log format (auto, pretty, compact, json).
    pub format: LogFormat,
    /// Disable ANSI colors (useful in CI systems or non-TTY environments).
    pub no_ansi: bool,
    /// Enable timestamps in log output.
    pub with_time: bool,
    /// Send logs to stderr (`true`) or stdout (`false`).
    pub to_stderr: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            format: LogFormat::Auto,
            no_ansi: false,
            with_time: true,
            to_stderr: true,
        }
    }
}

/// Macro to finish initialization of the tracing subscriber.
///
/// Based on the selected [`LogFormat`], this macro configures
/// the formatting layer and calls `.try_init()` on the builder.
///
/// # Errors
///
/// Returns the error from [`tracing_subscriber::fmt::SubscriberBuilder::try_init`]
/// if a global subscriber has already been installed.
macro_rules! finish_init {
    ($builder:expr, $format:expr, $config:expr) => {
        match $format {
            LogFormat::Pretty => $builder.pretty().try_init(),
            LogFormat::Compact => $builder.compact().try_init(),
            LogFormat::Json => $builder
                .json()
                .with_current_span(true)
                .with_span_list(true)
                .try_init(),
            LogFormat::Auto => unreachable!(),
        }
    };
}

/// Initialize the global tracing subscriber.
///
/// Configures the default tracing subscriber for the entire program:
/// - Respects the `RUST_LOG` environment variable via [`EnvFilter`]
/// - Chooses pretty vs JSON formatting automatically if `Auto` is set
/// - Enables thread IDs and names
/// - Optionally includes UTC timestamps
/// - Writes logs to either stdout or stderr
///
/// # Errors
///
/// Returns an error if a global subscriber has already been initialized,
/// or if subscriber initialization fails for another reason.
pub fn init_tracing(config: LogConfig) -> Result<()> {
    let is_tty = if config.to_stderr {
        std::io::stderr().is_terminal()
    } else {
        std::io::stdout().is_terminal()
    };

    let format = match config.format {
        LogFormat::Auto if is_tty => LogFormat::Pretty,
        LogFormat::Auto => LogFormat::Json,
        _ => config.format.clone(),
    };

    let make_writer = if config.to_stderr {
        BoxMakeWriter::new(std::io::stderr)
    } else {
        BoxMakeWriter::new(std::io::stdout)
    };

    let base_builder = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(make_writer)
        .with_ansi(!config.no_ansi)
        .with_thread_ids(true)
        .with_thread_names(true);

    if config.with_time {
        finish_init!(
            base_builder.with_timer(ChronoUtc::rfc_3339()),
            &format,
            &config
        )
        .map_err(|e| {
            anyhow!(e.to_string()).context("while setting up base subscriber with timer")
        })?;
    } else {
        finish_init!(base_builder.without_time(), &format, &config).map_err(|e| {
            anyhow!(e.to_string()).context("while setting up base subscriber without timer")
        })?;
    }

    Ok(())
}
