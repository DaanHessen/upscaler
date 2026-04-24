use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PromptSettings {
    // keep_aspect_ratio removed — structural preservation is always on
    #[serde(default)]
    pub keep_depth_of_field: bool,
    #[serde(default)]
    pub lighting: String, // "Original", "Studio", "Cinematic", "Vivid", "Natural"
    #[serde(default)]
    pub thinking_level: String, // "MINIMAL", "HIGH"
    #[serde(default)]
    pub seed: Option<u32>,
    #[serde(default)]
    pub target_medium: String, // for STYLIZE
    #[serde(default)]
    pub render_style: String, // for SKETCH
    #[serde(default)]
    pub target_aspect_ratio: String, // for EXPAND
}

pub fn build_tool_prompt(_tool_type: &str, style: &str, quality: &str, temperature: f32, settings: &PromptSettings) -> String {
    // We now focus exclusively on upscale
    build_upscale_prompt(style, quality, temperature, settings)
}

pub fn build_upscale_prompt(style: &str, quality: &str, temperature: f32, settings: &PromptSettings) -> String {
    let mut prompt = String::new();

    prompt.push_str("You are an expert image restoration and upscaling AI. Your sole objective is to enhance the resolution, sharpness, and clarity of the provided image without altering its original contents, shapes, or subjects in any way.\n\n");

    // Creativity / Temperature Logic
    if temperature <= 0.1 {
        prompt.push_str("This is a strict, perfectly faithful mathematical upscaling. Your only task is to increase the resolution, remove noise, and remove pixelation. You must maintain the exact semantic content, structural anatomy, and attributes of the original subject. Every output pixel must derive directly from the visible input data. If a texture or anatomical feature is absent from the input, it must remain completely absent from the output.\n\n");
    } else if temperature < 1.5 {
        prompt.push_str("This is a high-detail enhancement. You may clarify existing micro-details to simulate a high-quality photograph, but your additions must remain strictly constrained by the original subject's anatomy. Retain the original identity entirely without introducing any ungrounded physical features.\n\n");
    } else {
        prompt.push_str("This is a creative macro-photography upscale. You are permitted to enhance textures, lighting, and clarity to produce a highly detailed macro photograph. Interpret ambiguous areas with high-frequency details while preserving the core identity of the original subject.\n\n");
    }

    if style == "PHOTOGRAPHY" {
        prompt.push_str("Target Style: High-resolution professional photography. Ensure realistic lighting, shadows, and natural color balance.\n");
        if settings.keep_depth_of_field {
            prompt.push_str("Preserve the original depth of field and bokeh (background blur) exactly as it appears in the input.\n");
        }
    } else {
        prompt.push_str("Target Style: High-quality illustration/digital art. Preserve the original color palette, flat colors, gradients, and linework without introducing photographic artifacts or realistic textures.\n");
    }

    if settings.thinking_level == "HIGH" {
        prompt.push_str("\nPerform a deep pass on removing compression artifacts, chromatic aberration, and noise before upscaling.\n");
    }

    prompt.push_str(&format!("\nFinal output should be a clean, pristine {} image.", quality));

    prompt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upscale_strict() {
        let settings = PromptSettings::default();
        let prompt = build_upscale_prompt("PHOTOGRAPHY", "4K", 0.0, &settings);
        assert!(prompt.contains("perfectly faithful mathematical upscaling"));
        assert!(prompt.contains("absent from the input"));
    }

    #[test]
    fn test_upscale_creative() {
        let settings = PromptSettings::default();
        let prompt = build_upscale_prompt("PHOTOGRAPHY", "4K", 1.6, &settings);
        assert!(prompt.contains("creative macro-photography upscale"));
    }
}
