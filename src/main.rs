mod cli;
mod cookbook;
mod recipe;
mod scaffolding;
mod utils;

use std::{error::Error, path::PathBuf};

use clap::Parser;
use tracing::error;

use crate::{cli::Cli, cookbook::Cookbook, recipe::Recipe};

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
                let cookbook = Cookbook::new(cookbook_path)?;

                cookbook.run(continue_on_error).inspect_err(|_| {
                    eprintln!("Cookbook {} failed", cookbook.config.name);
                    error!(cookbook = %cookbook.config.name, "Cookbook failed");
                })?;
            }
        }
        cli::Command::Init { path } => {
            scaffolding::create_new_cookbook(&path)?;
        }
    }

    Ok(())
}
