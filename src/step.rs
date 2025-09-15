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
                std::env::var(key)
                    .ok()
                    .or(Some(String::new()))
                    .map(Cow::Owned)
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

#[cfg(test)]
mod tests {
    use super::*;

    // ---------- supports_os ----------

    #[test]
    fn supports_os_when_not_restricted() {
        let s = Step::Shell {
            id: "any".into(),
            os: vec![], // empty === all OSes
            cmd: "echo ok".into(),
        };
        assert!(s.supports_os("linux"));
        assert!(s.supports_os("macos"));
        assert!(s.supports_os("windows"));
    }

    #[test]
    fn supports_os_only_matches_listed_values() {
        let s = Step::Shell {
            id: "nix-only".into(),
            os: vec!["linux".into(), "macos".into()],
            cmd: "echo ok".into(),
        };
        assert!(s.supports_os("linux"));
        assert!(s.supports_os("macos"));
        assert!(!s.supports_os("windows"));
    }

    // ---------- serde deserialization ----------

    #[test]
    fn deserialize_shell_step_from_toml() {
        let toml_src = r#"
            type = "shell"
            id = "build"
            cmd = "echo building"
            os = ["linux","macos"]
        "#;

        let step: Step = toml::from_str(toml_src).expect("valid shell step toml");
        match step {
            Step::Shell { id, cmd, os } => {
                assert_eq!(id, "build");
                assert_eq!(cmd, "echo building");
                assert_eq!(os, vec!["linux".to_string(), "macos".to_string()]);
            }
        }
    }

    #[cfg(unix)]
    mod unix_exec_tests {
        use super::*;
        use std::path::PathBuf;

        /// Escape a Rust string for safe inclusion inside a double-quoted
        /// POSIX shell string literal.
        fn sh_escape_double_quoted(s: &str) -> String {
            s.replace('\\', r#"\\"#)
                .replace('"', r#"\""#)
                .replace('$', r#"\$"#)
        }

        #[test]
        fn execute_shell_success_returns_ok() {
            let envs = BTreeMap::new();
            Step::execute_shell_command("ok", "true", &envs).expect("should succeed");
        }

        #[test]
        fn execute_shell_failure_propagates_error() {
            let envs = BTreeMap::new();
            let err = Step::execute_shell_command("fail", "exit 7", &envs)
                .expect_err("should error for non-zero status");
            assert!(!format!("{err}").is_empty());
        }

        #[test]
        fn env_values_expand_pwd_and_match_expected() {
            // Arrange: MYVAR is defined via env map and contains $PWD + "/suffix"
            let mut envs = BTreeMap::new();
            envs.insert("MYVAR".into(), "$PWD/suffix".into());

            // Expected: current_dir()/suffix (string form)
            let expected = {
                let mut pb = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                pb.push("suffix");
                pb.to_string_lossy().into_owned()
            };
            let expected_escaped = sh_escape_double_quoted(&expected);

            // The command succeeds only if MYVAR equals the expected string.
            // Using `test` so success/failure is purely via exit code.
            let cmd = format!(r#"test "$MYVAR" = "{}""#, expected_escaped);

            Step::execute_shell_command("pwd-env", &cmd, &envs)
                .expect("value should match expected");
        }

        #[test]
        fn missing_env_vars_expand_to_empty_string() {
            // Define an env var whose value references a missing variable;
            // expand_with_cwd should turn it into an empty string.
            let mut envs = BTreeMap::new();
            envs.insert("EMPTYVAR".into(), "$DOES_NOT_EXIST".into());

            // Succeeds only if EMPTYVAR is empty (unset or "")
            let cmd = r#"[ -z "${EMPTYVAR:-}" ]"#;

            Step::execute_shell_command("empty-env", cmd, &envs)
                .expect("should be empty after expand");
        }
    }
}
