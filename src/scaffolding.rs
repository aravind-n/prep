mod templates;

use std::{error::Error, fs, path::Path, process::Command};

use tracing::{error, info};

pub fn create_new_cookbook(path: &Path) -> Result<(), Box<dyn Error>> {
    if path.exists() {
        error!("Directory {} already exists", path.display());
        return Err("Directory already exists".into());
    }

    let cookbook_name = path.file_name().unwrap().to_str().unwrap().to_string();
    info!("Initializing new cookbook at '{}'", cookbook_name);

    // Create the main cookbook directory
    fs::create_dir_all(path)?;

    // Create the required subdirectories.
    let recipes_dir = path.join("recipes");
    let scripts_dir = path.join("scripts");
    let templates_dir = path.join("templates");
    let assets_dir = path.join("assets");

    fs::create_dir_all(&recipes_dir)?;
    fs::create_dir_all(&scripts_dir)?;
    fs::create_dir_all(&templates_dir)?;
    fs::create_dir_all(&assets_dir)?;

    // Create config.toml
    let config_contents = templates::config_template(&cookbook_name);
    let config_path = path.join("config.toml");
    fs::write(config_path, config_contents)?;

    // Create recipe.toml
    let recipe_contents = templates::recipe_template();
    let recipe_path = recipes_dir.join("example.toml");
    fs::write(recipe_path, recipe_contents)?;

    // Create the README.md file
    let readme_contents = templates::default_readme_content(&cookbook_name);
    let readme_path = path.join("README.md");
    fs::write(readme_path, readme_contents)?;

    // Run `git init` in the new directory
    info!("Running `git init`...");
    let status = Command::new("git").arg("init").current_dir(path).status()?;

    if !status.success() {
        error!("Failed to run `git init`");
        return Err("Failed to initialize git repository".into());
    }

    info!("Initialized Cookbook");
    println!("Initialized Cookbook");

    Ok(())
}
