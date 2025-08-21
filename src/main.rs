mod cli;
mod recipe;
mod utils;

use std::{collections::BTreeMap, error::Error, fs, path::PathBuf, process::Command};

use clap::Parser;
use tracing::{error, warn};
use tracing_subscriber::EnvFilter;

use crate::{
    cli::Cli,
    recipe::{Recipe, Step},
};

/// Initializes `tracing_subscriber` configuration
///
/// This allows the package to output logs using
/// the `tracing` crate
pub fn init_tracing() {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(EnvFilter::from_default_env())
        .with_current_span(true)
        .with_span_list(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .init();
}

fn execute_shell_command(
    id: &str,
    cmd: &str,
    env_vars: &BTreeMap<String, String>,
) -> Result<(), Box<dyn Error>> {
    let mut command = Command::new("/bin/sh");
    command.arg("-c").arg(cmd);

    for (k, v) in env_vars {
        let expanded_value = shellexpand::env(v).unwrap().to_string();
        command.env(k, expanded_value);
    }

    let status = command.status().inspect_err(|e| {
        error!(error = %e, step_id = %id, "Error encountered when spawning {cmd}");
    })?;

    if !status.success() {
        error!(step_id = %id, cmd = %cmd, "Error encountered when running {cmd}");
        return Err("Failed to execute cmd".into());
    }

    Ok(())
}

fn run(path: &PathBuf, continue_on_error: bool) -> Result<(), Box<dyn Error>> {
    let raw =
        fs::read_to_string(path).inspect_err(|e| error!(error = %e, "Unable to read file"))?;

    let recipe: Recipe =
        toml::from_str(&raw).inspect_err(|e| error!(error = %e, "Unable to parse as toml"))?;

    let host_os = utils::host_os();

    println!(
        "====> Running recipe {} v{} on {host_os}\n",
        recipe.name, recipe.version
    );

    for step in recipe.steps.iter() {
        match step {
            Step::Shell { id, cmd, .. } => {
                if !step.supports_os(host_os) {
                    warn!("- skip {id} (mismatched os)");
                    continue;
                }

                let expanded_cmd = utils::expand_env(cmd);
                println!("-> step {id}");

                match execute_shell_command(id, &expanded_cmd, &recipe.env) {
                    Ok(_) => println!("- success {id}\n"),
                    Err(e) => {
                        eprintln!("- Failed {id}\n");
                        if !continue_on_error {
                            return Err(e);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    init_tracing();

    let cli = Cli::parse();
    let cmd = cli.cmd.unwrap_or(cli::Command::Run {
        path: None,
        continue_on_error: false,
    });

    let recipe_path = cmd.get_recipe_path();

    match cmd {
        cli::Command::Run {
            continue_on_error, ..
        } => {
            run(&recipe_path, continue_on_error)?;
        }
    }

    Ok(())
}
