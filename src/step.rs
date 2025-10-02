//! Steps for each recipe that contain information and execution instructions
//!
//! A `Step` represents one executable action in a recipe, such as running
//! a shell command. Each step can be constrained to certain operating systems
//! and may reference environment variables (including `$PWD`).

use std::{borrow::Cow, collections::BTreeMap, process::Command};

use anyhow::{Context, Result};
use serde::Deserialize;
use tracing::{error, info};

/// A step in a recipe.
///
/// Runs a shell command
///
/// # Fields
/// - `name`: Identifier for the step, used in logs.
/// - `cmd`: The shell command to execute.
/// - `os`: Optional list of supported OS names (`"macos"`, `"linux"`, `"windows"`).
///   If empty, the step runs on all operating systems.
#[derive(Debug, Deserialize)]
pub struct Step {
    pub name: String,
    pub cmd: String,
    #[serde(default)]
    pub os: Vec<String>,
    #[serde(default)]
    pub exit_on_success: bool,
}

impl Step {
    /// Checks if this step supports the given operating system.
    ///
    /// Returns `true` if:
    /// - The step’s `os` list is empty (meaning "all OSes"), or
    /// - The given `host_os` string matches one of the entries in the list.
    pub fn supports_os(&self, host_os: &str) -> bool {
        self.os.is_empty() || self.os.iter().any(|o| o == host_os)
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

    /// Safely single-quote a string for sh: ' -> '\'' sequence
    ///
    /// Returns the escaped string
    fn sh_single_quote(s: &str) -> String {
        let mut out = String::with_capacity(s.len() + 16);
        out.push('\'');
        for ch in s.chars() {
            if ch == '\'' {
                out.push_str("'\\''");
            } else {
                out.push(ch);
            }
        }
        out.push('\'');
        out
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
    pub fn run(&self, env_vars: Option<&BTreeMap<String, String>>) -> Result<()> {
        info!(step = %self.name, "Attempting to run step");

        let user_shell = match Step::expand_with_cwd("$SHELL") {
            s if s.is_empty() => "/bin/sh".into(),
            s => s,
        };

        let inner = Step::sh_single_quote(&self.cmd);
        let after_login = format!("exec /bin/sh -c {}", inner);

        let mut command = Command::new(&user_shell);

        command.arg("-l").arg("-c").arg(&after_login);

        if let Some(envs) = env_vars {
            for (k, v) in envs {
                let expanded_value = Step::expand_with_cwd(v);
                command.env(k, expanded_value);
            }
        }

        let status = command
            .status()
            .with_context(|| format!("Error encountered when spawning {}", self.cmd))?;

        if !status.success() {
            let code = status.code();
            error!(
                step = %self.name,
                cmd = %self.cmd,
                exit_code = ?code,
                "Error encountered while running step"
            );

            // Return the exit code wrapped with context
            return Err(anyhow::Error::msg(format!(
                "command `{}` for step `{}` failed{}",
                self.cmd,
                self.name,
                code.map(|c| format!(" (exit code {c})"))
                    .unwrap_or_default()
            ))
            .context("step execution did not succeed"));
        }

        info!(step = %self.name, "Successfully executed step");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn build_step(name: &str, cmd: &str, os: Vec<&str>) -> Step {
        Step {
            name: name.into(),
            cmd: cmd.into(),
            os: os.iter().map(|&s| s.into()).collect(),
            exit_on_success: false,
        }
    }

    // ---------- supports_os ----------

    #[test]
    fn supports_os_when_not_restricted() {
        let s = build_step("any", "echo ok", vec![]);

        assert!(s.supports_os("linux"));
        assert!(s.supports_os("macos"));
        assert!(s.supports_os("windows"));
    }

    #[test]
    fn supports_os_only_matches_listed_values() {
        let s = build_step("nix-only", "echo ok", vec!["linux", "macos"]);
        assert!(s.supports_os("linux"));
        assert!(s.supports_os("macos"));
        assert!(!s.supports_os("windows"));
    }

    // ---------- serde deserialization ----------

    #[test]
    fn deserialize_shell_step_from_toml() {
        let toml_src = r#"
            name = "build"
            cmd = "echo building"
            os = ["linux","macos"]
        "#;

        let step: Step = toml::from_str(toml_src).expect("valid shell step toml");

        assert_eq!(step.name, "build");
        assert_eq!(step.cmd, "echo building");
        assert_eq!(step.os, vec!["linux".to_string(), "macos".to_string()]);
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
        fn run_success_returns_ok() {
            let s = build_step("always-succeeds", "true", Vec::new());
            s.run(None).expect("should succeed");
        }

        #[test]
        fn run_failure_propagates_error() {
            let s = build_step("fail_step", "exit 7", Vec::new());
            let err = s.run(None).expect_err("should error for non-zero status");
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
            let s = build_step("pwd-env", &cmd, Vec::new());

            s.run(Some(&envs)).expect("value should match expected");
        }

        #[test]
        fn missing_env_vars_expand_to_empty_string() {
            // Define an env var whose value references a missing variable;
            // expand_with_cwd should turn it into an empty string.
            let mut envs = BTreeMap::new();
            envs.insert("EMPTYVAR".into(), "$DOES_NOT_EXIST".into());

            // Succeeds only if EMPTYVAR is empty (unset or "")
            let cmd = r#"[ -z "${EMPTYVAR:-}" ]"#;
            let s = build_step("empty-env", cmd, Vec::new());

            s.run(Some(&envs)).expect("should be empty after expand");
        }
    }
}
