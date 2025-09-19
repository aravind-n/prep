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
            if !step.supports_os(host_os) {
                println!(
                    "    - \x1b[33m[SKIP]\x1b[0m Step: {} (mismatched os)",
                    step.name
                );
                continue;
            }

            println!("    - Step: {}", step.name);
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
            if !step.supports_os(host_os) {
                warn!(step = %step.name, "step skipped (mismatched os)");
                continue;
            }

            println!("\n==> step {}", step.name);

            match step.run(Some(&env_vars)) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("==> step {} failed\n", step.name);
                    if !continue_on_error {
                        return Err(e);
                    }
                }
            }
        }

        println!("\n=> Recipe completed");
        info!(recipe = %self.name, "Finished recipe");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn write_recipe_toml(dir: &tempfile::TempDir, toml_src: &str) -> PathBuf {
        let p = dir.path().join("recipe.toml");
        std::fs::write(&p, toml_src).expect("write recipe");
        p
    }

    #[test]
    fn recipe_new_loads_from_toml() {
        let td = tempdir().unwrap();
        let toml_src = r#"
            name = "example"
            description = "demo"
            depends_on = ["a","b"]
            [env]
            FOO = "bar"

            [[steps]]
            name = "noop"
            cmd = "true"
        "#;
        let path = write_recipe_toml(&td, toml_src);

        let r = Recipe::new(path).expect("should parse");
        assert_eq!(r.name, "example");
        assert_eq!(r.description.as_deref(), Some("demo"));
        assert_eq!(r.depends_on, vec!["a".to_string(), "b".to_string()]);
        assert_eq!(r.env.get("FOO").map(|s| s.as_str()), Some("bar"));
        assert_eq!(r.steps.len(), 1);
        assert_eq!(r.steps[0].name, "noop");
        assert_eq!(r.steps[0].cmd, "true");
    }

    #[test]
    fn plan_smoke_test_does_not_panic() {
        let r = Recipe {
            name: "planme".into(),
            description: Some("desc".into()),
            depends_on: vec!["one".into()],
            env: BTreeMap::new(),
            steps: vec![Step {
                name: "s1".into(),
                cmd: "true".into(),
                os: vec![utils::host_os().into()],
            }],
        };
        r.plan(); // ensure no panic
    }

    // The `run` method uses /bin/sh, so limit these to Unix.
    #[cfg(unix)]
    mod unix_run {
        use super::*;

        #[test]
        fn run_executes_steps_successfully() {
            let r = Recipe {
                name: "ok".into(),
                description: None,
                depends_on: vec![],
                env: BTreeMap::new(),
                steps: vec![
                    Step {
                        name: "s1".into(),
                        cmd: "true".into(),
                        os: vec![utils::host_os().into()],
                    },
                    Step {
                        name: "s2".into(),
                        cmd: "true".into(),
                        os: vec![utils::host_os().into()],
                    },
                ],
            };

            r.run(false, None).expect("all steps should succeed");
        }

        #[test]
        fn env_is_merged_and_visible_in_step() {
            // cookbook provides A=one, recipe overrides/extends with B=two
            let mut cookbook_env = BTreeMap::new();
            cookbook_env.insert("A".into(), "one".into());

            let mut recipe_env = BTreeMap::new();
            recipe_env.insert("B".into(), "two".into());
            // also ensure recipe can override cookbook
            recipe_env.insert("A".into(), "override".into());

            // Succeeds only if A=override and B=two in the shell
            let cmd = r#"[ "${A:-}" = "override" ] && [ "${B:-}" = "two" ]"#;

            let r = Recipe {
                name: "env".into(),
                description: None,
                depends_on: vec![],
                env: recipe_env,
                steps: vec![Step {
                    name: "check".into(),
                    cmd: cmd.into(),
                    os: vec![utils::host_os().into()],
                }],
            };

            r.run(false, Some(cookbook_env))
                .expect("env merge should be visible");
        }

        #[test]
        fn os_mismatch_is_skipped_not_error() {
            // Choose an OS name that is definitely *not* the current one.
            let current = utils::host_os();
            let other = match current {
                "linux" => "macos",
                "macos" => "windows",
                _ => "linux",
            }
            .to_string();

            let r = Recipe {
                name: "skip".into(),
                description: None,
                depends_on: vec![],
                env: BTreeMap::new(),
                steps: vec![Step {
                    name: "s1".into(),
                    os: vec![other],
                    cmd: "exit 42".into(),
                }],
            };

            // Should not run, therefore should not fail.
            r.run(false, None).expect("mismatched OS steps are skipped");
        }

        #[test]
        fn continue_on_error_true_runs_remaining_steps() {
            let r = Recipe {
                name: "cont".into(),
                description: None,
                depends_on: vec![],
                env: BTreeMap::new(),
                steps: vec![
                    Step {
                        name: "fail".into(),
                        os: vec![utils::host_os().into()],
                        cmd: "exit 2".into(),
                    },
                    Step {
                        name: "ok".into(),
                        os: vec![utils::host_os().into()],
                        cmd: "true".into(),
                    },
                ],
            };

            // Should NOT return Err because continue_on_error = true
            r.run(true, None).expect("continues after failure");
        }

        #[test]
        fn continue_on_error_false_stops_on_first_error() {
            let r = Recipe {
                name: "stop".into(),
                description: None,
                depends_on: vec![],
                env: BTreeMap::new(),
                steps: vec![
                    Step {
                        name: "fail".into(),
                        os: vec![utils::host_os().into()],
                        cmd: "exit 3".into(),
                    },
                    Step {
                        name: "should_not_run".into(),
                        os: vec![utils::host_os().into()],
                        cmd: "true".into(),
                    },
                ],
            };

            let err = r.run(false, None).expect_err("should stop on first error");
            assert!(!format!("{err}").is_empty());
        }
    }
}
