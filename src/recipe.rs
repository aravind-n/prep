use std::{collections::BTreeMap, error::Error, path::PathBuf, process::Command};

use serde::Deserialize;
use tracing::{error, info, warn};

use crate::utils;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Step {
    #[serde(rename = "shell")]
    Shell {
        id: String,
        #[serde(default)]
        os: Vec<String>, // ["macos"] | ["linux"]
        cmd: String,
    },
}

impl Step {
    pub fn supports_os(&self, host_os: &str) -> bool {
        match self {
            Step::Shell { os, .. } => os.is_empty() || os.iter().any(|o| o == host_os),
        }
    }

    fn expand_env(s: &str) -> String {
        shellexpand::env(s).unwrap_or_else(|_| s.into()).into()
    }

    fn execute_shell_command(
        id: &str,
        cmd: &str,
        env_vars: &BTreeMap<String, String>,
    ) -> Result<(), Box<dyn Error>> {
        let expanded_cmd = Self::expand_env(cmd);

        let mut command = Command::new("/bin/sh");
        command.arg("-c").arg(expanded_cmd);

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
}

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
    pub fn new(recipe_path: PathBuf) -> Result<Self, Box<dyn Error>> {
        let raw_recipe = std::fs::read_to_string(&recipe_path)?;
        let recipe: Recipe = toml::from_str(&raw_recipe)?;

        Ok(recipe)
    }

    pub fn plan(&self) {
        let host_os = utils::host_os();
        let mut dependencies_str = self.depends_on.join(", ");
        if !dependencies_str.is_empty() {
            dependencies_str = format!(" {dependencies_str} ");
        }

        println!("-> Plan for recipe: {}", self.name);
        println!("Depends on: [{}]", dependencies_str);

        if let Some(description) = &self.description {
            println!("Description: {}", description);
        }

        for step in &self.steps {
            match step {
                Step::Shell { id, .. } => {
                    if !step.supports_os(host_os) {
                        println!("- Step: {id} skipped (mismatched os)");
                        continue;
                    }

                    println!("- Step: {id}");
                }
            }
        }
    }

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
