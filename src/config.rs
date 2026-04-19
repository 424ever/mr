use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Settings {
    pub ui: UiSettings,
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
