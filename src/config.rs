use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};
use xdg::BaseDirectories;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Settings {
    #[serde(default)]
    pub ui: UiSettings,
    #[serde(default)]
    pub info: InfoSettings,
}

impl Settings {
    pub fn load() -> anyhow::Result<Self> {
        let dirs = BaseDirectories::with_prefix("mr");
        let f = match dirs.find_config_file("config.toml") {
            Some(f) => f,
            None => return Ok(Self::default()),
        };

        let s = fs::read_to_string(f)?;

        Ok(toml_edit::de::from_str(&s)?)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UiSettings {
    pub pager: Vec<String>,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            pager: vec!["less".into(), "-FXR".into()],
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InfoSettings {
    pub paths: Vec<SearchPath>,
}

impl Default for InfoSettings {
    fn default() -> Self {
        Self {
            paths: vec![
                SearchPath::FromEnv("INFOPATH".into()),
                SearchPath::Path("/usr/share/info".into()),
            ],
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SearchPath {
    FromEnv(String),
    Path(PathBuf),
}
