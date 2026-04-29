use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PromptSettings {
    #[serde(default)]
    pub restoration_pass: bool,
    #[serde(default)]
    pub pre_process_pass: bool,
    #[serde(default)]
    pub face_enhancement: bool,
    
    #[serde(default = "default_0")]
    pub noise_reduction: i32,
    #[serde(default = "default_0")]
    pub sharpen: i32,
    #[serde(default = "default_0")]
    pub remove_artifacts: i32,
    
    #[serde(default = "default_creativity")]
    pub creativity: f32,
    #[serde(default)]
    pub seed: Option<u64>,
    
    #[serde(default)]
    pub original_filename: Option<String>,
}

fn default_0() -> i32 { 0 }
fn default_creativity() -> f32 { 0.35 }
