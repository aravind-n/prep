use std::{
    error::Error,
    fs,
    path::Path,
    process::{Command, Stdio},
};

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

    // Create the README.md file
    let readme_contents = build_template(include_str!("../assets/README.tmpl.md"), &cookbook_name);
    let readme_path = path.join("README.md");
    fs::write(readme_path, readme_contents)?;

    // Create config.toml
    let config_contents =
        build_template(include_str!("../assets/config.tmpl.toml"), &cookbook_name);
    let config_path = path.join("config.toml");
    fs::write(config_path, config_contents)?;

    // Create recipe.toml
    let recipe_contents = include_str!("../assets/example.toml");
    let recipe_path = recipes_dir.join("example.toml");
    fs::write(recipe_path, recipe_contents)?;

    // Create scripts/example-script.sh
    let script_contents = include_str!("../assets/recipe-script.sh");
    let script_path = scripts_dir.join("example-script.sh");
    fs::write(&script_path, script_contents)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms)?;
    }

    // Create script_recipe.toml
    let recipe_contents = include_str!("../assets/scripts-example.toml");
    let recipe_path = recipes_dir.join("scripts-example.toml");
    fs::write(recipe_path, recipe_contents)?;

    // Run `git init` in the new directory
    info!("Running `git init`...");
    let status = Command::new("git")
        .arg("init")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .current_dir(path)
        .status()?;

    if !status.success() {
        error!("Failed to run `git init`");
        return Err("Failed to initialize git repository".into());
    }

    info!("Initialized Cookbook");
    println!("Initialized Cookbook");

    Ok(())
}

fn build_template(contents: &str, var: &str) -> String {
    contents.replace("{}", var)
}
