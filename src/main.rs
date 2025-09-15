mod cli;
mod cookbook;
mod recipe;
mod step;
mod utils;

use std::{env, error::Error, fs, path::PathBuf};

use clap::Parser;
use tracing::error;

use crate::{cli::Cli, cookbook::Cookbook, recipe::Recipe};

/// Handles all CLI command variants expected by the program
/// and delegates execution
fn main() -> Result<(), Box<dyn Error>> {
    utils::init_tracing();

    let cli = Cli::parse();
    let cmd = cli.cmd.unwrap_or(cli::Command::Run {
        cookbook_path: Some(PathBuf::from("./")),
        recipe: None,
        continue_on_error: false,
    });

    match cmd {
        cli::Command::Run {
            cookbook_path,
            recipe,
            continue_on_error,
        } => {
            if let Some(recipe_path) = recipe {
                let recipe = Recipe::new(recipe_path)?;
                recipe.run(continue_on_error, None).inspect_err(|_| {
                    eprintln!("Recipe {} failed", recipe.name);
                    error!(recipe = recipe.name, "Recipe failed");
                })?;
            } else if let Some(cookbook_path) = cookbook_path {
                // Set current dir to cookbook_path
                let prev_dir = env::current_dir()?;
                let cookbook_path = fs::canonicalize(cookbook_path)?;
                env::set_current_dir(&cookbook_path)?;

                // Run cookbook
                let cookbook = Cookbook::new(cookbook_path)?;

                cookbook.run(continue_on_error).inspect_err(|_| {
                    eprintln!("Cookbook {} failed", cookbook.name);
                    error!(cookbook = %cookbook.name, "Cookbook failed");
                })?;

                // Restore current_dir
                env::set_current_dir(prev_dir)?;
            }
        }
        cli::Command::Init { path } => {
            utils::create_new_cookbook(&path)?;
        }
        cli::Command::Plan { cookbook_path } => {
            let cookbook = Cookbook::new(cookbook_path)?;
            cookbook.plan()?;
        }
    }

    Ok(())
}
