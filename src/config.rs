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

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct UiSettings {
    pub pager: PagerSettings,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PagerSettings {
    pub cmd: Vec<String>,
    pub start_line_arg: Option<String>,
}

impl Default for PagerSettings {
    fn default() -> Self {
        Self {
            cmd: vec!["less".into(), "-FXR".into()],
            start_line_arg: Some("+{}G".into()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InfoSettings {
    paths: Vec<SearchPath>,
}

impl InfoSettings {
    pub(crate) fn paths(&self) -> &Vec<SearchPath> {
        &self.paths
    }
}

impl Default for InfoSettings {
    fn default() -> Self {
        Self {
            paths: vec![
                SearchPath::Immediate,
                SearchPath::DirFile("/usr/share/info/dir".into()),
                SearchPath::DirsFromEnv("INFOPATH".into()),
                SearchPath::Dir("/usr/share/info".into()),
            ],
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SearchPath {
    Immediate,
    DirFile(PathBuf),
    DirsFromEnv(String),
    Dir(PathBuf),
}
