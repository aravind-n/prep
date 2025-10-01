# Prep

A prep kitchen for your computer
  
**prep** is a fast, minimal tool for bootstrapping developer machines with
reproducible, version-controlled recipes.

## What can I use it for?

Setting up dev machines is slow and inconsistent. Teams want a dead-simple,
reproducible way to execute a **set of commands** that may have **dependencies**

## Features (v0.2.8)

- Scaffolds a new cookbook with
- Runs an individual recipe
- Runs a cookbook
  - Performs dependency resolution for all your recipes
- Works on macOS and Linux
- Streams stdout/stderr live to your console
- Rich logging support
- Early exit logic in steps with `exit_on_success`

## Installation

Build from source (requires Rust):

```bash
git clone https://gitlab.com/aravind/prep.git
cd prep
cargo install --path .
```

This installs the prep binary into your Cargo bin path.

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

See the example cookbook

## Recipe Format

A recipe is a TOML file describing environment variables and steps. An example can be found in ``example.toml``

## Future Work

- Arrays for cmd in recipe.toml (maybe)
- Windows support
- Early exit in recipe
- New name (because conflict. I like this name)
