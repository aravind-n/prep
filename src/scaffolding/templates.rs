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
version = "0.1.0"
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
        r#"# {}

This is a new cookbook scaffolded by `mise`.

## Getting Started

To run this cookbook, navigate to the directory and use the following command:

```sh
mise run .
```
"#,
        cookbook_name,
    )
}
