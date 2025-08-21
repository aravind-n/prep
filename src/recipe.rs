use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Recipe {
    pub name: String,

    pub version: u32,

    #[serde(default)]
    pub env: BTreeMap<String, String>,

    pub steps: Vec<Step>,
}

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
}
