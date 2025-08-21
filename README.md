Mise
===
Mise-en-place for your computer
  

**mise** is a fast, minimal tool for bootstrapping developer machines with
reproducible, version-controlled recipes.

## Why?
Setting up dev machines is slow and inconsistent. Teams want a dead-simple,
reproducible way to execute a **set of commands** that may have **dependencies**
and can run **in parallel**  
_**NOTE**: For v0.1.0, mise runs steps sequentially for maximum simplicity and speed of delivery._

## Features (v0.1.0)
- 🚀 Runs a `recipe.toml` to set up your machine
- 📝 Simple TOML schema: `[env]` and `[[steps]]`
- 🌍 Works on macOS and Linux
- ❌ Stops on first error by default
- 📜 Streams stdout/stderr live to your console

## Installation
Build from source (requires Rust):

```bash
git clone https://github.com/yourname/mise.git
cd mise
cargo install --path .
```

This installs the mise binary into your Cargo bin path.

## Usage

Run a recipe in the current directory:
```sh
$ mise run
```

Or specify a recipe explicitly:
```sh
$ mise run ./recipes/java-dev.toml
```

Options:
- ``--continue-on-error``: keep going even if a step fails

## Recipe Format
A recipe is a TOML file describing environment variables and steps. An example can be found in ``example.toml``

## Future Work
- DAG + async execution (parallel steps)
- More step types (pkg, file, service)
- mise plan, mise validate, mise verify
- Windows support
