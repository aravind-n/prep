use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "mise", version, propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    Run {
        cookbook_path: Option<PathBuf>,

        #[arg(short, long)]
        recipe: Option<PathBuf>,

        #[arg(long, default_value_t = false)]
        continue_on_error: bool,
    },
}
