use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PromptSettings {
    #[serde(default = "default_off")]
    pub pre_processing: String,
    #[serde(default = "default_off")]
    pub post_polish: String,
    #[serde(default)]
    pub topaz_mode: Option<String>,
    #[serde(default)]
    pub face_enhancement: bool,
    #[serde(default)]
    pub skip_topaz: bool,
    #[serde(default = "default_model")]
    pub model: String, // "Standard" or "Premium"
    #[serde(default)]
    pub refinement: bool, // For Premium model
    #[serde(default)]
    pub restoration_pass: bool, // Manual override for restoration
    
    #[serde(default = "default_creativity")]
    pub creativity: f32,
    #[serde(default)]
    pub seed: Option<u64>,
    
    #[serde(default = "default_0")]
    pub noise_reduction: i32,
    #[serde(default = "default_0")]
    pub sharpen: i32,
    #[serde(default = "default_0")]
    pub remove_artifacts: i32,
    
    #[serde(default)]
    pub original_filename: Option<String>,
}

fn default_0() -> i32 { 0 }

fn default_off() -> String { "Off".to_string() }
fn default_model() -> String { "Premium".to_string() }
fn default_creativity() -> f32 { 0.35 }
