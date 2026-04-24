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
}

pub fn build_system_prompt(style: &str, quality: &str, settings: &PromptSettings) -> String {
    let mut prompt = String::new();

    prompt.push_str("System Role: You are the UPSYL high-fidelity reconstruction and super-resolution engine.\n\n");
    prompt.push_str(&format!("Objective: Perform a precise 1:1 super-resolution enhancement of this image to {} resolution.\n\n", quality));

    prompt.push_str("Rules:\n");
    if style == "PHOTOGRAPHY" {
        prompt.push_str("1. Texture Balance: Synthesize photorealistic organic micro-textures (hair, pores, fabric, foliage) with maximum fidelity. Retain natural sensor grain seamlessly.\n");
        prompt.push_str("2. Absolute Biological Fidelity: Render human subjects with flawless realism, keeping all natural skin features, pores, and distinct physiological traits exactly as they appear.\n");
        
        if settings.keep_depth_of_field {
            prompt.push_str("3. Depth of Field: Reconstruct the exact original focal planes. Ensure background elements remain optically soft and beautifully blurred out of focus.\n");
        } else {
            prompt.push_str("3. Focal Clarity: Enhance edge clarity and fine details across all depth planes while retaining a believable photographic drop-off.\n");
        }
    } else {
        prompt.push_str("1. Artistic Integrity: Recreate the precise original art style. Render primary outlines perfectly sharp, crisp, and flawlessly anti-aliased.\n");
        prompt.push_str("2. Surface Purity: Generate clean, immaculate surfaces. Smooth out all compression artifacts, color banding, and digital noise into pristine color fields.\n");
        prompt.push_str("3. Color & Gradients: Replicate the original flat color values perfectly. Render background gradients as completely smooth transitions devoid of unintended texture.\n");
    }

    prompt.push_str("4. Structural Lock: Lock the exact silhouettes, relative proportions, and spatial positioning of every element. Ensure the entire composition matches the source frame flawlessly.\n");

    match settings.lighting.to_uppercase().as_str() {
        "STUDIO" => prompt.push_str("5. Lighting: Enhance the scene with professional studio high-key lighting, emphasizing form and volume naturally.\n"),
        "CINEMATIC" => prompt.push_str("5. Lighting: Reconstruct the scene using dramatic cinematic lighting with rich, deep shadows and anamorphic-style contrast.\n"),
        "VIVID" => prompt.push_str("5. Lighting: Amplify color vibrancy and perceived dynamic range to produce a strikingly rich, high-contrast finish.\n"),
        "NATURAL" => prompt.push_str("5. Lighting: Balance the illumination with soft, natural overcast light to produce a gentle and even exposure.\n"),
        _ => prompt.push_str("5. Lighting Preservation: Replicate the exact native lighting and illumination sources present in the original input.\n"),
    }

    prompt.push_str("\nFinal Instruction: Produce a literal, hyper-accurate representation of the input image. Ensure all generated details structurally align with the original source content.");

    prompt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_photography_lighting_vivid() {
        let settings = PromptSettings {
            keep_depth_of_field: true,
            lighting: "VIVID".to_string(),
            thinking_level: "HIGH".to_string(),
            seed: None,
        };
        let prompt = build_system_prompt("PHOTOGRAPHY", "4K", &settings);
        
        assert!(prompt.contains("UPSYL high-fidelity reconstruction"));
        assert!(prompt.contains("organic micro-textures"));
        assert!(prompt.contains("exact original focal planes"));
        assert!(prompt.contains("Amplify color vibrancy"));
        assert!(prompt.contains("to 4K resolution"));
    }

    #[test]
    fn test_illustration_original_lighting() {
        let settings = PromptSettings {
            keep_depth_of_field: false,
            lighting: "Original".to_string(),
            thinking_level: "MINIMAL".to_string(),
            seed: None,
        };
        let prompt = build_system_prompt("ILLUSTRATION", "2K", &settings);
        
        assert!(prompt.contains("Recreate the precise original art style"));
        assert!(prompt.contains("clean, immaculate surfaces"));
        assert!(prompt.contains("Structural Lock"));
        assert!(prompt.contains("exact native lighting"));
        assert!(prompt.contains("to 2K resolution"));
    }

    #[test]
    fn test_photography_shallow_focus() {
        let settings = PromptSettings {
            keep_depth_of_field: false,
            lighting: "STUDIO".to_string(),
            thinking_level: "HIGH".to_string(),
            seed: None,
        };
        let prompt = build_system_prompt("PHOTOGRAPHY", "4K", &settings);
        assert!(prompt.contains("Enhance edge clarity"));
        assert!(prompt.contains("professional studio high-key lighting"));
    }
}