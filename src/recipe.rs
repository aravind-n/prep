//! Defines a [`Recipe`], which describes a sequence of steps
//! (e.g., shell commands) to execute, along with metadata like
//! dependencies, environment variables, and description.
//!
//! Recipes are typically loaded from TOML files and executed in order.

use std::{collections::BTreeMap, error::Error, path::PathBuf};

use serde::Deserialize;
use tracing::{info, warn};

use crate::{step::Step, utils};

/// A recipe describing a sequence of steps and associated metadata.
///
/// Recipe can be executed via [`run`](Recipe::run) or displayed as
/// a plan via [`plan`](Recipe::plan).
#[derive(Debug, Deserialize)]
pub struct Recipe {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    pub steps: Vec<Step>,
}

impl Recipe {
    /// Loads a recipe from a TOML file at the given `recipe_path`.
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or the TOML cannot
    /// be deserialized into a [`Recipe`].
    pub fn new(recipe_path: PathBuf) -> Result<Self, Box<dyn Error>> {
        let raw_recipe = std::fs::read_to_string(&recipe_path)?;
        let recipe: Recipe = toml::from_str(&raw_recipe)?;

        Ok(recipe)
    }

    /// Prints a plan of the recipe to stdout, without executing any steps.
    pub fn plan(&self) {
        let host_os = utils::host_os();
        let mut dependencies_str = self.depends_on.join(", ");
        if !dependencies_str.is_empty() {
            dependencies_str = format!(" {dependencies_str} ");
        }

        println!("-> Plan for recipe: {}", self.name);
        println!("   Depends on: [{}]", dependencies_str);

        if let Some(description) = &self.description {
            println!("   Description: {}", description);
        }

        for step in &self.steps {
            match step {
                Step::Shell { id, .. } => {
                    if !step.supports_os(host_os) {
                        println!("    - \x1b[33m[SKIP]\x1b[0m Step: {id} (mismatched os)");
                        continue;
                    }

                    println!("    - Step: {id}");
                }
            }
        }
    }

    /// Runs the recipe, executing its steps sequentially.
    ///
    /// Each step’s shell command is executed with environment variables
    /// merged from:
    /// - Global cookbook environment (if provided), and
    /// - This recipe’s own `env` map.
    ///
    /// Steps that don’t support the current OS (see [`Step::supports_os`])
    /// are skipped with a warning.
    ///
    /// # Parameters
    /// - `continue_on_error`: If `true`, execution continues after a failed step.
    ///   Otherwise, execution stops on the first error.
    /// - `cookbook_env_vars`: Optional environment variables inherited from
    ///   the cookbook.
    ///
    /// # Errors
    /// Returns the error from the first failed step if `continue_on_error` is `false`.
    /// Also returns errors for underlying I/O or process failures.
    pub fn run(
        &self,
        continue_on_error: bool,
        cookbook_env_vars: Option<BTreeMap<String, String>>,
    ) -> Result<(), Box<dyn Error>> {
        let host_os = utils::host_os();

        let mut env_vars;

        if let Some(cookbook_envs) = cookbook_env_vars {
            env_vars = cookbook_envs.clone();
            env_vars.extend(self.env.iter().map(|(k, v)| (k.clone(), v.clone())));
        } else {
            env_vars = self.env.clone();
        }

        info!(recipe = %self.name, "Starting recipe");
        println!("\n=> Running recipe {}", self.name);

        for step in &self.steps {
            match step {
                Step::Shell { id, cmd, .. } => {
                    if !step.supports_os(host_os) {
                        warn!(step_id = %id, "step skipped (mismatched os)");
                        continue;
                    }

                    println!("\n==> step {id}");

                    match Step::execute_shell_command(id, cmd, &env_vars) {
                        Ok(_) => (),
                        Err(e) => {
                            eprintln!("==> step {id} failed\n");
                            if !continue_on_error {
                                return Err(e);
                            }
                        }
                    }
                }
            }
        }

        println!("\n=> Recipe completed");
        info!(recipe = %self.name, "Finished recipe");

        Ok(())
    }
}
