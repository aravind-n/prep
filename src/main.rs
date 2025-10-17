mod cli;
mod cookbook;
mod logging;
mod recipe;
mod step;
mod utils;

use std::{env, fs};

use anyhow::Result;
use clap::Parser;
use tracing::{error, info};

use crate::{cli::Cli, cookbook::Cookbook, logging::LogConfig, recipe::Recipe};

/// Handles all CLI command variants expected by the program
/// and delegates execution
fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing with config from CLI
    logging::init_tracing(&LogConfig {
        format: cli.log_format,
        no_ansi: cli.no_ansi,
        with_time: !cli.no_time,
        to_stderr: !cli.log_to_stdout,
    })?;

    match cli.cmd {
        cli::Command::Init { path } => {
            utils::scaffold_cookbook_project(&path)?;
        }
        cli::Command::Plan { cookbook_path } => {
            let cookbook = Cookbook::new(&cookbook_path)?;
            cookbook.plan()?;
        }
        cli::Command::Run {
            cookbook_path,
            recipe,
            continue_on_error,
        } => {
            if let Some(recipe_path) = recipe {
                info!("Single recipe execution mode invoked");

                let recipe = Recipe::new(&recipe_path)?;
                recipe.run(continue_on_error, None).inspect_err(|_| {
                    error!(recipe = recipe.name, "Recipe failed");
                })?;
            } else {
                // Set current dir to cookbook_path
                let prev_dir = env::current_dir()?;
                let cookbook_path = fs::canonicalize(cookbook_path)?;
                env::set_current_dir(&cookbook_path)?;

                // Run cookbook
                let cookbook = Cookbook::new(&cookbook_path)?;

                cookbook.run(continue_on_error).inspect_err(|_| {
                    error!(cookbook = %cookbook.name, "Cookbook failed");
                })?;

                // Restore current_dir
                env::set_current_dir(prev_dir)?;
            }
        }
    }

    Ok(())
}
