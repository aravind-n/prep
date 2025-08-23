use std::{
    collections::{BTreeMap, HashMap},
    error::Error,
    path::{Path, PathBuf},
};

use petgraph::{algo::toposort, graphmap::DiGraphMap};
use serde::Deserialize;
use tracing::{error, info};

use crate::recipe::Recipe;

#[derive(Debug, Deserialize)]
pub struct CookbookConfig {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
}

pub struct Cookbook {
    pub config: CookbookConfig,
    pub recipes: BTreeMap<String, Recipe>,
    recipe_to_id: HashMap<String, u32>,
    id_to_recipe: HashMap<u32, String>,
}

type RecipeMaps = (
    BTreeMap<String, Recipe>,
    HashMap<String, u32>,
    HashMap<u32, String>,
);

impl Cookbook {
    pub fn new(cookbook_path: PathBuf) -> Result<Self, Box<dyn Error>> {
        let config = Self::build_config(&cookbook_path)?;
        let (recipes, recipe_to_id, id_to_recipe) = Self::build_recipe_maps(&cookbook_path)?;

        Ok(Self {
            config,
            recipes,
            recipe_to_id,
            id_to_recipe,
        })
    }

    fn build_config(cookbook_path: &Path) -> Result<CookbookConfig, Box<dyn Error>> {
        let config_path = cookbook_path.join("config.toml");
        let raw_config = std::fs::read_to_string(&config_path)?;
        Ok(toml::from_str(&raw_config)?)
    }

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

    pub fn plan(&self) -> Result<(), Box<dyn Error>> {
        let sorted_recipe_ids = self.get_sorted_recipe_ids()?;

        println!("Cookbook {} v{} execution plan:", self.config.name, self.config.version);
        if let Some(description) = &self.config.description {
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

    pub fn run(&self, continue_on_error: bool) -> Result<(), Box<dyn Error>> {
        let sorted_recipe_ids = self.get_sorted_recipe_ids()?;

        println!(
            "Building cookbook {} v{}",
            self.config.name, self.config.version
        );
        info!(cookbook = %self.config.name, "Starting cookbook execution");

        for recipe_id in sorted_recipe_ids {
            let recipe_name = self.id_to_recipe.get(&recipe_id).unwrap();
            let recipe = self.recipes.get(recipe_name).unwrap();

            recipe.run(continue_on_error, Some(self.config.env.clone()))?;
        }

        println!("Built cookbook {}", self.config.name);
        info!(cookbook = %self.config.name, "Cookbook executed successfully");

        Ok(())
    }
}
