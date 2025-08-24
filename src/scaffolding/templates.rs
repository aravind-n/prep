pub(super) fn config_template(cookbook_name: &str) -> String {
    format!(
        r#"name = "{}"
version = "0.1.0"
description = "This is an example of what a cookbook is"
author = "Your Name"
license = "MIT"

[env]
# Add global environment variables here
"#,
        cookbook_name
    )
}

pub(super) fn recipe_template() -> String {
    r#"name = "example"
description = "An example recipe"
author = "Your Name"
depends_on = []

[[step]]
id = "hello-world"
description = "Prints a greeting to the console"
type = "shell"
cmd = "echo 'Hello, World!'"
os = ["macos", "linux"]
"#
    .into()
}

pub(super) fn default_readme_content(cookbook_name: &str) -> String {
    format!(
        r#"# {cookbook_name}

This is a new cookbook scaffolded by `mise`.

## Getting Started

### Prerequisites

- git
- mise
  

### Running the cookbook

From outside the cookbook directory

```sh
mise run path/to/{cookbook_name}
```

Alternatively from inside the cookbook directory

```sh
mise run .
```

### Editing the cookbook

- Clone this repo with `git clone <repo_url>`
- `cd {cookbook_name}`

## Cookbook Structure

The cookbook consists of the following

```
config.toml - Main configuration settings nfor the workbook. Includes global variables shared by all recipes
recipes/ - All recipes used to build the cookbook
assets/ - Assets that are read-only. Recipes will not alter these files and will deploy them as is. Recipes may write new files to assets
scripts/ - User scripts that are used by recipes. useful when a single line command isn't enough for a task
templates/ - Template files that require per-user substitution. Recipes will use these files to generate an asset
README - General description about the cookbook
```

"#,
    )
}
