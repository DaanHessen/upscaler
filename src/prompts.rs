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

    prompt.push_str("System Role: You are the Novura high-fidelity reconstruction and super-resolution engine.\n\n");
    prompt.push_str(&format!("Objective: Perform a professional super-resolution enhancement of this asset to {} resolution.\n\n", quality));

    prompt.push_str("## Reconstruction Pipeline:\n");
    prompt.push_str("1. Signal Integrity: Maintain absolute geometric and structural parity. Every edge, silhouette, and volume must be a direct derivative of the source's spatial data.\n");
    prompt.push_str("2. Phenomenological Restoration: Reconstruct the surface properties (reflections, textures, micro-shadows) based exclusively on the luminosity and color shifts present in the source. Do not 'guess' what the subject might be; enhance the data precisely as it exists.\n");

    // Creativity / Temperature Logic (Fluid breakpoints)
    if temperature <= 0.1 {
        prompt.push_str("3. Zero-Hallucination Mode: You are in 'Absolute Signal Proxy' mode. Your task is to mathematically resolve the high-frequency intent of the source. You are FORBIDDEN from generating new anatomical or structural features (e.g., extra hairs, whiskers, pores, or wrinkles) that are not visibly suggested by sub-pixel clusters in the original. If a feature is not in the source, it must not be in the output.\n");
    } else if temperature < 0.9 {
        prompt.push_str("3. Enhanced Handshake: You are in 'Signal Clarification' mode. Enhance the clarity of existing textures. Derive fine detail from the local frequency of the source data. Maintain the strict identity of every surface.\n");
    } else {
        prompt.push_str("3. Artistic Resolution: You are in 'Generative Handshake' mode. You have latitude to clarify ambiguous textures with high-fidelity patterns, provided they integrate seamlessly with the source's lighting and material properties.\n");
    }

    if style == "PHOTOGRAPHY" {
        prompt.push_str("4. Optic Reconstruction: Simulate a high-end sensor. Extract realistic material properties—surface roughness, specular scattering, and micro-textures—solely from the source's color gradients. Do not apply pre-trained SUBJECT prototypes.\n");
        if settings.keep_depth_of_field {
            prompt.push_str("5. Optic Lock: Preserve the original lens characteristics and focal planes. Maintain existing background blur (bokeh) consistency.\n");
        }
    } else {
        prompt.push_str("4. Illustration Purity: Reconstruct clean, aliasing-free outlines and sophisticated gradients. Maintain the purity of solid colors and the specific signature of the original medium.\n");
    }

    // Thinking Level / Depth
    if settings.thinking_level == "HIGH" {
        prompt.push_str("\n## Deep Signal Analysis:\nAnalyze the underlying luminosity data to derive physically consistent surface details. Ensure these additions are 100% anchored to the source radiance and do not introduce new, un-suggested features.\n");
    }

    prompt.push_str("\nFinal Instruction: Deliver a literal, technically superior version of the input. You are a SIGNAL PROCESSOR, not an artist. Output a version of the input that is clear, sharp, and high-fidelity, while adhering to a 'NO NEW ANATOMY' rule. Do not interpret. Reconstruct.");

    prompt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upscale_strict() {
        let settings = PromptSettings::default();
        let prompt = build_upscale_prompt("PHOTOGRAPHY", "4K", 0.0, &settings);
        assert!(prompt.contains("Zero-Hallucination"));
        assert!(prompt.contains("Signal Integrity"));
        assert!(prompt.contains("to 4K resolution"));
    }

    #[test]
    fn test_upscale_creative() {
        let settings = PromptSettings::default();
        let prompt = build_upscale_prompt("PHOTOGRAPHY", "4K", 1.0, &settings);
        assert!(prompt.contains("Artistic"));
        assert!(prompt.contains("latitude"));
    }
}
