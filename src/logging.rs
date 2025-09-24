use std::io::IsTerminal;

use clap::ValueEnum;
use tracing_subscriber::{
    EnvFilter,
    fmt::{time::ChronoUtc, writer::BoxMakeWriter},
};

#[derive(Clone, Debug, ValueEnum)]
pub enum LogFormat {
    Auto,
    Pretty,
    Compact,
    Json,
}

pub struct LogConfig {
    pub format: LogFormat,
    pub no_ansi: bool,
    pub with_time: bool,
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

pub fn init_tracing(config: LogConfig) {
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
        let _ = finish_init!(
            base_builder.with_timer(ChronoUtc::rfc_3339()),
            &format,
            &config
        );
    } else {
        let _ = finish_init!(base_builder.without_time(), &format, &config);
    }
}
