# {}

This is a new cookbook scaffolded by [prep](https://gitlab.com/aravind/prep).

## Getting Started

### Prerequisites

- git
- prep

### Running the cookbook

From outside the cookbook directory

```sh
prep run path/to/{}
```

Alternatively from inside the cookbook directory

```sh
prep run .
```

### Editing the cookbook

- Clone this repo with `git clone <repo_url>`
- `cd {}`

## Cookbook Structure

The cookbook consists of the following

- config.toml - Main configuration settings for the cookbook. Includes global variables shared by all recipes
- recipes/ - All recipes used to build the cookbook
- scripts/ - Optional user scripts that are used by recipes. Useful when a single line command isn't enough for a task
- README - General description about the cookbook
