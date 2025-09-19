# {}

This is a new cookbook scaffolded by `mise`.

## Getting Started

### Prerequisites

- git
- mise

### Running the cookbook

From outside the cookbook directory

```sh
mise run path/to/{}
```

Alternatively from inside the cookbook directory

```sh
mise run .
```

### Editing the cookbook

- Clone this repo with `git clone <repo_url>`
- `cd {}`

## Cookbook Structure

The cookbook consists of the following

- config.toml - Main configuration settings nfor the workbook. Includes global variables shared by all recipes
- recipes/ - All recipes used to build the cookbook
- assets/ - Assets that are read-only. Recipes will not alter these files and will deploy them as is. Recipes may write new files to assets
- scripts/ - User scripts that are used by recipes. useful when a single line command isn't enough for a task
- templates/ - Template files that require per-user substitution. Recipes will use these files to generate an asset
- README - General description about the cookbook
