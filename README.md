Mise
===
Mise-en-place for your computer
  

**mise** is a fast, minimal tool for bootstrapping developer machines with
reproducible, version-controlled recipes.

## What can I use it for?
Setting up dev machines is slow and inconsistent. Teams want a dead-simple,
reproducible way to execute a **set of commands** that may have **dependencies**

## Features (v0.2.0)
- Runs a cookbook that you manage
- Cookbooks are a git repo that you manage
- Performs dependency resolution for all your recipes
- Works on macOS and Linux
- Stops on first error by default
- Streams stdout/stderr live to your console

## Installation
Build from source (requires Rust):

```bash
git clone https://github.com/yourname/mise.git
cd mise
cargo install --path .
```

This installs the mise binary into your Cargo bin path.

## Usage

Run a cookbook (inside cookbook):
```sh
$ mise run
```

Explicitly run a cookbook:
```sh
$ mise run ./cookbook
```

Run a single recipe:
```sh
$ mise run --recipe java_setup.toml
```

Options:
- ``--continue-on-error``: keep going even if a step fails

## Cookbook structure
See the example cookbook

## Recipe Format
A recipe is a TOML file describing environment variables and steps. An example can be found in ``example.toml``

## Future Work
- More step types (pkg, etc)
- arrays for cmd in recipe.toml (maybe)
- Windows support
