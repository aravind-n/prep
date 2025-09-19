//! Cookbook abstraction: groups together multiple recipes, their
//! dependencies, and shared configuration.
//!
//! A [`Cookbook`] is defined by a `config.toml` file and a `recipes/`
//! directory containing individual recipe TOML files. Recipes can
//! depend on one another, and the cookbook resolves dependencies
//! using a directed acyclic graph (DAG) and topological sorting.

use std::{
    collections::{BTreeMap, HashMap},
    error::Error,
    path::{Path, PathBuf},
};

use petgraph::{algo::toposort, graphmap::DiGraphMap};
use serde::Deserialize;
use tracing::{error, info};

use crate::recipe::Recipe;

/// Configuration metadata for a cookbook, loaded from `config.toml`.
///
/// This is a helper struct used to initialize a cookbook
#[derive(Debug, Deserialize)]
struct CookbookConfig {
    name: String,
    version: String,
    description: Option<String>,
    #[serde(default)]
    env: BTreeMap<String, String>,
}

/// A collection of recipes and shared configuration.
///
/// A `Cookbook` is responsible for:
/// - Loading configuration and recipes from disk,
/// - Building a dependency graph between recipes,
/// - Sorting recipes in dependency order,
/// - Executing or planning recipes in the correct sequence.
pub struct Cookbook {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub env: BTreeMap<String, String>,
    pub recipes: BTreeMap<String, Recipe>,
    recipe_to_id: HashMap<String, u32>,
    id_to_recipe: HashMap<u32, String>,
}

/// Internal alias for a recipe map tuple.
///
/// Used to reduce code clutter.
type RecipeMaps = (
    BTreeMap<String, Recipe>,
    HashMap<String, u32>,
    HashMap<u32, String>,
);

impl Cookbook {
    /// Loads a cookbook from the given `cookbook_path`.
    ///
    /// Expects:
    /// - `config.toml` at the root of the cookbook,
    /// - A `recipes/` directory containing recipe TOML files.
    ///
    /// # Errors
    /// Returns an error if configuration or any recipe file cannot
    /// be read or parsed.
    pub fn new(cookbook_path: PathBuf) -> Result<Self, Box<dyn Error>> {
        let config = Self::build_config(&cookbook_path)?;
        let (recipes, recipe_to_id, id_to_recipe) = Self::build_recipe_maps(&cookbook_path)?;

        Ok(Self {
            name: config.name,
            version: config.version,
            description: config.description,
            env: config.env,
            recipes,
            recipe_to_id,
            id_to_recipe,
        })
    }

    /// Reads and parses `config.toml` into a [`CookbookConfig`].
    fn build_config(cookbook_path: &Path) -> Result<CookbookConfig, Box<dyn Error>> {
        let config_path = cookbook_path.join("config.toml");
        let raw_config = std::fs::read_to_string(&config_path)?;
        Ok(toml::from_str(&raw_config)?)
    }

    /// Loads all recipe TOML files from the `recipes/` directory.
    ///
    /// Assigns each recipe a numeric ID for graph construction.
    ///
    /// # Errors
    /// Returns an error if any recipe file cannot be read or parsed.
    fn build_recipe_maps(cookbook_path: &Path) -> Result<RecipeMaps, Box<dyn Error>> {
        let recipes_dir = cookbook_path.join("recipes");
        let mut recipes = BTreeMap::new();
        let mut recipe_to_id = HashMap::new();
        let mut id_to_recipe = HashMap::new();
        let mut recipe_id: u32 = 0;

        for entry in std::fs::read_dir(recipes_dir)? {
            let path = entry?.path();
            if path.extension().unwrap_or_default() == "toml" {
                let recipe = Recipe::new(path)?;
                let recipe_name = recipe.name.clone();

                recipes.insert(recipe_name.clone(), recipe);
                recipe_to_id.insert(recipe_name.clone(), recipe_id);
                id_to_recipe.insert(recipe_id, recipe_name.clone());

                recipe_id += 1;
            }
        }

        Ok((recipes, recipe_to_id, id_to_recipe))
    }

    /// Builds a directed acyclic graph (DAG) of recipe dependencies.
    ///
    /// Each recipe is represented by a node. Edges point from a dependency
    /// to the recipe that depends on it.
    ///
    /// # Errors
    /// Returns an error if a recipe declares a dependency that does not exist.
    fn build_graph(&self) -> Result<DiGraphMap<u32, ()>, Box<dyn Error>> {
        let mut graph = DiGraphMap::new();

        for &id in self.id_to_recipe.keys() {
            graph.add_node(id);
        }

        for (recipe_name, recipe) in &self.recipes {
            if let Some(&recipe_id) = self.recipe_to_id.get(recipe_name) {
                for dependency in &recipe.depends_on {
                    if let Some(&dependency_id) = self.recipe_to_id.get(dependency) {
                        graph.add_edge(dependency_id, recipe_id, ());
                    } else {
                        error!(dependency = %dependency, recipe = %recipe_name, "Dependency not found");
                        return Err("Missing dependency".into());
                    }
                }
            }
        }

        Ok(graph)
    }

    /// Returns recipe IDs sorted in dependency order.
    ///
    /// Uses [topological sorting](https://en.wikipedia.org/wiki/Topological_sorting)
    /// to ensure dependencies are executed before the recipes that depend on them.
    ///
    /// # Errors
    /// Returns an error if a circular dependency is detected.
    pub fn get_sorted_recipe_ids(&self) -> Result<Vec<u32>, Box<dyn Error>> {
        let graph = self.build_graph()?;

        Ok(toposort(&graph, None).map_err(|e| {
            let node_name = self
                .id_to_recipe
                .get(&e.node_id())
                .unwrap_or(&"Unknown".to_string())
                .clone();

            error!(node = %node_name, "Circular dependency detected");
            "Circular dependency detected"
        })?)
    }

    /// Prints a plan of the cookbook to stdout.
    ///
    /// Includes:
    /// - Cookbook name and version
    /// - Optional description
    /// - Each recipe in dependency order, with its own plan output
    pub fn plan(&self) -> Result<(), Box<dyn Error>> {
        let sorted_recipe_ids = self.get_sorted_recipe_ids()?;

        println!("{} v{} execution plan:", self.name, self.version);
        if let Some(description) = &self.description {
            println!("Description: {description}\n")
        }

        for recipe_id in sorted_recipe_ids {
            let recipe_name = self.id_to_recipe.get(&recipe_id).unwrap();
            let recipe = self.recipes.get(recipe_name).unwrap();
            recipe.plan();
            println!();
        }

        Ok(())
    }

    /// Executes all recipes in the cookbook in dependency order.
    ///
    /// Global environment variables from [`CookbookConfig::env`] are
    /// passed to each recipe.
    ///
    /// # Parameters
    /// - `continue_on_error`: If `true`, continue executing even if
    ///   a recipe fails. Otherwise, stop on first failure.
    ///
    /// # Errors
    /// Returns an error if a recipe fails and `continue_on_error` is `false`,
    /// or if any underlying recipe execution errors occur.
    pub fn run(&self, continue_on_error: bool) -> Result<(), Box<dyn Error>> {
        let sorted_recipe_ids = self.get_sorted_recipe_ids()?;

        println!("Building cookbook {} v{}", self.name, self.version);
        info!(cookbook = %self.name, "Starting cookbook execution");

        for recipe_id in sorted_recipe_ids {
            let recipe_name = self.id_to_recipe.get(&recipe_id).unwrap();
            let recipe = self.recipes.get(recipe_name).unwrap();

            recipe.run(continue_on_error, Some(self.env.clone()))?;
        }

        println!("Built cookbook {}", self.name);
        info!(cookbook = %self.name, "Cookbook executed successfully");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    // Escape a Rust string for TOML basic string context.
    fn toml_basic_escape(s: &str) -> String {
        s.replace('\\', "\\\\").replace('"', "\\\"")
    }

    /// Create a temp cookbook on disk with a config and N recipes.
    ///
    /// - `name`/`version` go into config.toml
    /// - `extra_env` goes under [env] in config.toml (e.g., OUTPUT path)
    /// - `recipes` is (file_stem, depends_on, cmd)
    fn write_cookbook(
        name: &str,
        version: &str,
        extra_env: &[(&str, &str)],
        recipes: &[(&str, &[&str], &str)],
    ) -> tempfile::TempDir {
        let td = tempdir().expect("tempdir");
        let root = td.path();

        // config.toml
        let mut config = format!(
            r#"
name = "{name}"
version = "{version}"
description = "test cookbook"
[env]
"#
        );
        for (k, v) in extra_env {
            config.push_str(&format!(r#"{k} = "{}""#, toml_basic_escape(v)));
            config.push('\n');
        }
        fs::write(root.join("config.toml"), config).unwrap();

        // recipes/
        let recipes_dir = root.join("recipes");
        fs::create_dir_all(&recipes_dir).unwrap();

        for (file_stem, deps, cmd) in recipes {
            let mut toml_src = String::new();
            toml_src.push_str(&format!(r#"name = "{file_stem}""#));
            toml_src.push('\n');

            if !deps.is_empty() {
                toml_src.push_str("depends_on = [");
                for (i, d) in deps.iter().enumerate() {
                    if i > 0 {
                        toml_src.push_str(", ");
                    }
                    toml_src.push('"');
                    toml_src.push_str(d);
                    toml_src.push('"');
                }
                toml_src.push_str("]\n");
            }

            toml_src.push_str(
                r#"
[[steps]]
"#,
            );
            toml_src.push_str(&format!(r#"name = "step-{file_stem}""#));
            toml_src.push('\n');

            // 🔧 Escape the command for TOML basic string
            let cmd_escaped = toml_basic_escape(cmd);
            toml_src.push_str(&format!(r#"cmd = "{}""#, cmd_escaped));
            toml_src.push('\n');

            fs::write(recipes_dir.join(format!("{file_stem}.toml")), toml_src).unwrap();
        }

        td
    }

    #[test]
    fn new_loads_config_and_recipes() {
        let td = write_cookbook(
            "demo",
            "0.1.0",
            &[("FOO", "BAR")],
            &[("a", &[], "true"), ("b", &["a"], "true")],
        );

        let cb = Cookbook::new(td.path().to_path_buf()).expect("load cookbook");
        assert_eq!(cb.name, "demo");
        assert_eq!(cb.version, "0.1.0");
        assert_eq!(cb.description.as_deref(), Some("test cookbook"));
        assert_eq!(cb.env.get("FOO").map(|s| s.as_str()), Some("BAR"));
        assert_eq!(cb.recipes.len(), 2);
        assert!(cb.recipes.contains_key("a"));
        assert!(cb.recipes.contains_key("b"));
    }

    #[test]
    fn sorted_ids_respect_dependencies_linear_chain() {
        // a -> b -> c
        let td = write_cookbook(
            "sort",
            "1.0",
            &[],
            &[
                ("a", &[], "true"),
                ("b", &["a"], "true"),
                ("c", &["b"], "true"),
            ],
        );
        let cb = Cookbook::new(td.path().to_path_buf()).unwrap();

        let order = cb.get_sorted_recipe_ids().expect("toposort ok");

        // Map ids back to names (tests are in-module, so we can read private fields)
        let names: Vec<String> = order
            .into_iter()
            .map(|id| cb.id_to_recipe.get(&id).unwrap().clone())
            .collect();

        assert!(names.windows(2).any(|w| w == ["a", "b"]), "b after a");
        assert!(names.windows(2).any(|w| w == ["b", "c"]), "c after b");
        // Ensure 'a' appears before 'c' as well
        let pos = |s: &str| names.iter().position(|n| n == s).unwrap();
        assert!(pos("a") < pos("b"));
        assert!(pos("b") < pos("c"));
    }

    #[test]
    fn missing_dependency_errors() {
        let td = write_cookbook(
            "missing-dep",
            "1.0",
            &[],
            &[("a", &["does-not-exist"], "true")],
        );
        let cb = Cookbook::new(td.path().to_path_buf()).unwrap();
        let err = cb
            .get_sorted_recipe_ids()
            .expect_err("should error on missing dep");
        assert!(!format!("{err}").is_empty());
    }

    #[test]
    fn circular_dependency_detected() {
        // a -> b, b -> a
        let td = write_cookbook(
            "cycle",
            "1.0",
            &[],
            &[("a", &["b"], "true"), ("b", &["a"], "true")],
        );
        let cb = Cookbook::new(td.path().to_path_buf()).unwrap();
        let err = cb
            .get_sorted_recipe_ids()
            .expect_err("should error on cycle");
        assert!(!format!("{err}").is_empty());
    }

    #[test]
    fn plan_smoke_test() {
        let td = write_cookbook(
            "plan",
            "1.0",
            &[],
            &[("a", &[], "true"), ("b", &["a"], "true")],
        );
        let cb = Cookbook::new(td.path().to_path_buf()).unwrap();
        cb.plan().expect("plan should succeed");
    }

    // The run path shells out with /bin/sh; make this Unix-only.
    #[cfg(unix)]
    #[test]
    fn run_executes_in_dependency_order_and_passes_env() {
        // We'll set OUTPUT in config.env so steps can append their letter there.
        let td = tempdir().unwrap();
        let out_file = td.path().join("out.txt");
        let out_str = out_file.to_string_lossy().into_owned();

        // Build cookbook where:
        // a: append "A"
        // b (depends on a): append "B"
        // c (depends on b): append "C"
        let cookdir = write_cookbook(
            "runit",
            "1.0",
            &[("OUTPUT", &out_str)],
            &[
                ("a", &[], r#"printf "A" >> "$OUTPUT""#),
                ("b", &["a"], r#"printf "B" >> "$OUTPUT""#),
                ("c", &["b"], r#"printf "C" >> "$OUTPUT""#),
            ],
        );

        let cb = Cookbook::new(cookdir.path().to_path_buf()).unwrap();
        cb.run(false).expect("run should succeed");

        let contents = fs::read_to_string(out_file).expect("read out");
        assert_eq!(contents, "ABC");
    }
}
