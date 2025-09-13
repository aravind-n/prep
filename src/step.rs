use std::{borrow::Cow, collections::BTreeMap, error::Error, process::Command};

use serde::Deserialize;
use tracing::error;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Step {
    #[serde(rename = "shell")]
    Shell {
        id: String,
        #[serde(default)]
        os: Vec<String>, // ["macos"] | ["linux"]
        cmd: String,
    },
}

impl Step {
    pub fn supports_os(&self, host_os: &str) -> bool {
        match self {
            Step::Shell { os, .. } => os.is_empty() || os.iter().any(|o| o == host_os),
        }
    }

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
