# Prep

A prep kitchen for your computer
  
**prep** is a fast, minimal tool for bootstrapping developer machines with
reproducible, version-controlled recipes.

## What can I use it for?

Setting up dev machines is slow and inconsistent. Teams want a dead-simple,
reproducible way to execute a **set of commands** that may have **dependencies**

## Installation

### Direct install

```bash
curl --proto '=https' --tlsv1.2 -fsSL https://aravind.gitlab.io/prep/install.sh | bash
```

### Build from source (requires Rust)

This installs the prep binary into your Cargo bin path.

```bash
git clone https://gitlab.com/aravind/prep.git
cd prep
cargo install --path .
```

## Features (v0.3.2)

- Scaffolds a new cookbook with
- Runs an individual recipe
- Runs a cookbook
  - Performs dependency resolution for all your recipes
- Works on macOS and Linux
- Streams stdout/stderr live to your console
- Rich logging support
- Early exit logic in steps with `exit_on_success`
- New installer script

## Usage

Initialize a cookbook:

```bash
prep init ./path/to/cookbook    # Defaults to ./
```

Run a cookbook:

```bash
prep run ./cookbook     # Defaults to ./
```

Run a single recipe:

```bash
prep run --recipe java_setup.toml
```

Options:

- ``--continue-on-error``: keep going even if a step fails

## Cookbook structure

Cookbooks are a collection of recipes that can be run together. For example,
a dev machine setup cookbook can contain recipes that set up your language
tools, IDEs, etc. At it's heart, each cookbook contains a `config.toml` file
that describes the cookbook's characteristics and a `recipes` directory
containing any recipes used to build the cookbook.

## Recipe Format

A recipe is a TOML file describing environment variables and steps. Each
recipe contains a list of steps that define the actions that a recipe will
take. You can look into example recipes by running `prep init` and inspecting
the generated recipes.

## Future Work

- Arrays for cmd in recipe.toml (maybe)
- Windows support
