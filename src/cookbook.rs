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

        println!(
            "{} v{} execution plan:",
            self.name, self.version
        );
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

        println!(
            "Building cookbook {} v{}",
            self.name, self.version
        );
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
