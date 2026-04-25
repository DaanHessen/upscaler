use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PromptSettings {
    // keep_aspect_ratio removed — structural preservation is always on
    #[serde(default = "default_off", alias = "refinement_pass")]
    pub pre_processing: String,
    #[serde(default = "default_off")]
    pub post_polish: String,
    #[serde(default)]
    pub topaz_mode: Option<String>,
}

fn default_off() -> String { "Off".to_string() }
