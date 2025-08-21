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
        path: Option<PathBuf>,

        #[arg(long, default_value_t = false)]
        continue_on_error: bool,
    },
}

impl Command {
    pub(crate) fn get_recipe_path(&self) -> PathBuf {
        match &self {
            Command::Run { path, .. } => match path {
                Some(p) if p.is_dir() => p.join("recipe.toml"),
                Some(p) => p.clone(),
                None => PathBuf::from("recipe.toml"),
            },
        }
    }
}
