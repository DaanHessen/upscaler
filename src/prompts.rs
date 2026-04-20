use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PromptSettings {
    #[serde(default)]
    pub keep_aspect_ratio: bool,
    #[serde(default)]
    pub keep_depth_of_field: bool,
    #[serde(default)]
    pub lighting: String, // "Original", "Studio", "Cinematic", "Vivid", "Natural"
    #[serde(default)]
    pub thinking_level: String, // "MINIMAL", "HIGH"
}

pub fn build_system_prompt(style: &str, settings: &PromptSettings) -> String {
    let mut prompt = String::new();

    // 1. PREFIX - Unified Studio Identity
    prompt.push_str("System Role: You are the UPSYL high-fidelity reconstruction and super-resolution engine.\n\n");
    prompt.push_str("Objective: Perform a 1:1 UPSYL super-resolution restore of this image to 4K resolution.\n\n");

    // 2. CORE STYLE RULES
    prompt.push_str("Rules:\n");
    if style == "PHOTOGRAPHY" {
        prompt.push_str("1. Texture Balance: Restore organic micro-textures (hair, fabric, environments) with high fidelity. Preserve natural sensor grain.\n");
        prompt.push_str("2. Absolute Biological Fidelity: strictly preserve original skin condition; do NOT apply beauty filters or smooth natural textures.\n");
        
        if settings.keep_depth_of_field {
            prompt.push_str("3. Depth of Field: Strictly maintain the original focal planes. Do not sharpen background bokeh; keep them optically soft.\n");
        } else {
            prompt.push_str("3. Focal Clarity: Enhance clarity across all planes while maintaining a natural drop-off.\n");
        }
    } else {
        prompt.push_str("1. Artistic Integrity: Maintain exact original art style. Keep primary outlines razor-sharp and perfectly anti-aliased.\n");
        prompt.push_str("2. Artifact Destruction: Completely remove all JPEG compression artifacts, color banding, and noise.\n");
        prompt.push_str("3. Color & Gradients: Preserve exact flat color values. Smooth intentional background gradients without introducing texture.\n");
    }

    // 3. SETTINGS-BASED INSTRUCTIONS
    if settings.keep_aspect_ratio {
        prompt.push_str("4. Structural Lock: Preserve the exact silhouettes, proportions, and positioning of every element. Do not crop or mutate image boundaries.\n");
    } else {
        prompt.push_str("4. Compositional Optimization: Ensure subject focus is maintained while maximizing 4K canvas utility.\n");
    }

    // 4. LIGHTING RULES
    match settings.lighting.to_uppercase().as_str() {
        "STUDIO" => prompt.push_str("5. Lighting: Enhance with professional studio high-key lighting, emphasizing form and volume.\n"),
        "CINEMATIC" => prompt.push_str("5. Lighting: Reconstruct using dramatic cinematic lighting with rich shadows and anamorphic-style contrast.\n"),
        "VIVID" => prompt.push_str("5. Lighting: Amplify color vibrancy and perceived dynamic range for a striking, high-contrast finish.\n"),
        "NATURAL" => prompt.push_str("5. Lighting: Balance with soft, natural overcast light, minimizing harsh reflections.\n"),
        _ => prompt.push_str("5. Lighting Preservation: Strictly maintain the exact structural identity and lighting of the original input. Do not introduce new light sources.\n"),
    }

    // 5. SUFFIX
    prompt.push_str("\nFinal Instruction: Do not hallucinate or invent new semantic objects. Maintain 100% fidelity to the source image content.");

    prompt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_photography_lighting_vivid() {
        let settings = PromptSettings {
            keep_aspect_ratio: true,
            keep_depth_of_field: true,
            lighting: "VIVID".to_string(),
            thinking_level: "HIGH".to_string(),
        };
        let prompt = build_system_prompt("PHOTOGRAPHY", &settings);
        
        assert!(prompt.contains("UPSYL high-fidelity reconstruction"));
        assert!(prompt.contains("Restore organic micro-textures"));
        assert!(prompt.contains("Strictly maintain the original focal planes"));
        assert!(prompt.contains("Amplify color vibrancy"));
    }

    #[test]
    fn test_illustration_original_lighting() {
        let settings = PromptSettings {
            keep_aspect_ratio: false,
            keep_depth_of_field: false,
            lighting: "Original".to_string(),
            thinking_level: "MINIMAL".to_string(),
        };
        let prompt = build_system_prompt("ILLUSTRATION", &settings);
        
        assert!(prompt.contains("Maintain exact original art style"));
        assert!(prompt.contains("Completely remove all JPEG compression artifacts"));
        assert!(prompt.contains("Compositional Optimization"));
        assert!(prompt.contains("Strictly maintain the exact structural identity"));
    }

    #[test]
    fn test_photography_shallow_focus() {
        let settings = PromptSettings {
            keep_aspect_ratio: true,
            keep_depth_of_field: false,
            lighting: "STUDIO".to_string(),
            thinking_level: "HIGH".to_string(),
        };
        let prompt = build_system_prompt("PHOTOGRAPHY", &settings);
        assert!(prompt.contains("Enhance clarity across all planes"));
        assert!(prompt.contains("professional studio high-key lighting"));
    }
}