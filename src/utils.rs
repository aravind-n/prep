//! Utilities used throughout the application.

use std::{
    fs::{self},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::{Context, Result, bail};
use tracing::{error, info};

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

/// Creates a new cookbook directory structure at the given `path`.
///
/// # Errors
///
/// Returns an error if:
/// - The target path already exists,
/// - Any filesystem operation fails (creating directories, writing files, setting permissions),
/// - `git init` fails to execute or returns a non‐zero exit status.
///
/// # Side effects
///
/// - Creates and writes files/directories on the local filesystem.
/// - Executes the `git` command if available on `PATH`.
/// - On Unix platforms, sets executable permission (`0o755`) on `scripts/example-script.sh`.
pub fn scaffold_cookbook_project(path: &Path) -> Result<()> {
    info!(path = %path.display(), "Attempting to initialize new cookbook");

    if path.exists() {
        error!(path = %path.display(), "Directory already exists");
        bail!("Directory {} already exists", path.display());
    }

    let cookbook_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_owned())
        .context("Cannot infer cookbook from path (invalid or empty last component")?;

    // Helper for directory creation with context
    let mkdir = |p: &Path| -> Result<()> {
        fs::create_dir_all(p).with_context(|| format!("Creating directory `{}`", p.display()))
    };

    // Create the main cookbook directory
    mkdir(path)?;

    // Create the required subdirectories.
    let recipes_dir = path.join("recipes");
    let scripts_dir = path.join("scripts");

    mkdir(&recipes_dir)?;
    mkdir(&scripts_dir)?;

    // Write files with clear context
    let writef = |p: PathBuf, contents: &str| -> Result<()> {
        fs::write(&p, contents).with_context(|| format!("Writing file `{}`", p.display()))
    };

    // Create the README.md file
    let readme = build_template(include_str!("../assets/README.tmpl.md"), &cookbook_name);
    writef(path.join("README.md"), &readme)?;

    // Create config.toml
    let config_toml = build_template(include_str!("../assets/config.tmpl.toml"), &cookbook_name);
    writef(path.join("config.toml"), &config_toml)?;

    // Create example.toml
    writef(
        recipes_dir.join("example.toml"),
        include_str!("../assets/example.toml"),
    )?;

    // Create scripts/example-script.sh
    let script_path = scripts_dir.join("example-script.sh");
    writef(
        script_path.clone(),
        include_str!("../assets/recipe-script.sh"),
    )?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path)
            .with_context(|| format!("reading metadata for `{}`", script_path.display()))?
            .permissions();

        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).with_context(|| {
            format!(
                "setting executable permissions on `{}`",
                script_path.display()
            )
        })?;
    }

    // Create script_example.toml
    writef(
        recipes_dir.join("scripts-example.toml"),
        include_str!("../assets/scripts-example.toml"),
    )?;

    // Run `git init` in the new directory
    info!("Running `git init`...");
    let status = Command::new("git")
        .arg("init")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .current_dir(path)
        .status()
        .with_context(|| format!("failed to spawn `git init` in `{}`", path.display()))?;

    if !status.success() {
        error!(path = %path.display(), code = ?status.code(), "Failed to run `git init`");

        bail!(
            "`git init` failed in `{}`{}",
            path.display(),
            status
                .code()
                .map(|c| format!(" (exit code {c})"))
                .unwrap_or_default()
        );
    }

    println!("Initialized Cookbook");
    info!(path = %path.display(), "Successfuly initialized cookbook");

    Ok(())
}

/// Performs simple template substitution by replacing `{}` with `var`.
///
/// # Parameters
/// - `contents`: The template string containing `{}` as a placeholder.
/// - `var`: The replacement value (typically the cookbook name).
fn build_template(contents: &str, var: &str) -> String {
    contents.replace("{}", var)
}
