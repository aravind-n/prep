//! Steps for each recipe that contain information and execution instructions
//!
//! A `Step` represents one executable action in a recipe, such as running
//! a shell command. Each step can be constrained to certain operating systems
//! and may reference environment variables (including `$PWD`).

use std::{borrow::Cow, collections::BTreeMap, error::Error, process::Command};

use serde::Deserialize;
use tracing::error;

/// A step in a recipe.
///
/// Currently, the only supported step type is [`Shell`], which executes
/// a command in a POSIX shell.
///
/// The enum is `#[serde(tag = "type")]` so that deserialization chooses
/// the variant based on the `"type"` field in serialized input.
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Step {
    /// Run a shell command.
    ///
    /// # Fields
    /// - `id`: Identifier for the step, used in logs.
    /// - `os`: Optional list of supported OS names (`"macos"`, `"linux"`, `"windows"`).
    ///   If empty, the step runs on all operating systems.
    /// - `cmd`: The shell command to execute.
    #[serde(rename = "shell")]
    Shell {
        id: String,
        #[serde(default)]
        os: Vec<String>,
        cmd: String,
    },
}

impl Step {
    /// Checks if this step supports the given operating system.
    ///
    /// Returns `true` if:
    /// - The step’s `os` list is empty (meaning "all OSes"), or
    /// - The given `host_os` string matches one of the entries in the list.
    pub fn supports_os(&self, host_os: &str) -> bool {
        match self {
            Step::Shell { os, .. } => os.is_empty() || os.iter().any(|o| o == host_os),
        }
    }

    /// Expands environment variables in the given string,
    /// with special handling for `$PWD`.
    ///
    /// - `$PWD` is replaced with the current working directory.
    /// - Other environment variables are substituted from the process environment.
    /// - Missing variables are expanded to an empty string.
    fn expand_with_cwd(input: &str) -> String {
        shellexpand::env_with_context_no_errors(input, |key| {
            if key == "PWD" {
                std::env::current_dir()
                    .ok()
                    .map(|p| Cow::Owned(p.to_string_lossy().into_owned()))
            } else {
                std::env::var(key).ok().map(Cow::Owned)
            }
        })
        .into_owned()
    }

    /// Executes a shell command for a step. Currently only supports unix systems
    ///
    /// The command is executed with `/bin/sh -c <cmd>`, and any additional
    /// environment variables are injected from `env_vars`. Each variable value
    /// is expanded using [`expand_with_cwd`].
    ///
    /// # Parameters
    /// - `id`: Identifier of the step.
    /// - `cmd`: The command string to execute.
    /// - `env_vars`: Key/value pairs of environment variables to set.
    ///
    /// # Returns
    /// - `Ok(())` if the command executes successfully with a zero exit code.
    /// - `Err(..)` if the process fails to spawn or returns a non-zero status.
    ///
    /// # Errors
    /// Logs errors with [`tracing::error`] and returns a boxed error.
    pub fn execute_shell_command(
        id: &str,
        cmd: &str,
        env_vars: &BTreeMap<String, String>,
    ) -> Result<(), Box<dyn Error>> {
        let mut command = Command::new("/bin/sh");
        command.arg("-c").arg(cmd);

        for (k, v) in env_vars {
            let expanded_value = Step::expand_with_cwd(v);
            command.env(k, expanded_value);
        }

        let status = command.status().inspect_err(|e| {
            error!(error = %e, step_id = %id, "Error encountered when spawning {cmd}");
        })?;

        if !status.success() {
            error!(step_id = %id, cmd = %cmd, "Error encountered when running {cmd}");
            return Err("Failed to execute cmd".into());
        }

        Ok(())
    }
}
