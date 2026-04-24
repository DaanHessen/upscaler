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

pub fn build_tool_prompt(tool_type: &str, style: &str, quality: &str, settings: &PromptSettings) -> String {
    match tool_type {
        "RELIGHT" => build_relight_prompt(settings),
        "STYLIZE" => build_stylize_prompt(settings),
        "SKETCH" => build_sketch_prompt(settings),
        "EXPAND" => build_expand_prompt(settings),
        _ => build_system_prompt(style, quality, settings), // Default upscale
    }
}

pub fn build_relight_prompt(settings: &PromptSettings) -> String {
    let mut prompt = String::new();
    prompt.push_str("System Role: You are an expert lighting technician and geometry-preserving renderer.\n\n");
    prompt.push_str("Objective: Modify the illumination of this scene without altering any underlying geometry, identities, or objects.\n\n");
    prompt.push_str("Rules:\n");
    prompt.push_str("1. Structural Lock: Preserve the exact silhouettes, proportions, and positioning of every element. Do not crop or mutate image boundaries.\n");
    prompt.push_str("2. Identity Lock: Do not change the identity of subjects, facial features, or existing textures. Only change the light interacting with them.\n");

    match settings.lighting.to_uppercase().as_str() {
        "NEON" | "CYBERPUNK" => prompt.push_str("3. Lighting: Apply a vibrant Cyberpunk Neon lighting setup with strong magenta, cyan, and teal rim lights.\n"),
        "STUDIO" => prompt.push_str("3. Lighting: Enhance the scene with professional studio high-key lighting, emphasizing form and volume naturally.\n"),
        "CINEMATIC" => prompt.push_str("3. Lighting: Reconstruct the scene using dramatic cinematic lighting with rich, deep shadows and anamorphic-style contrast.\n"),
        "GOLDEN HOUR" => prompt.push_str("3. Lighting: Bathe the scene in warm, directional Golden Hour sunlight with long, soft shadows.\n"),
        "NATURAL" => prompt.push_str("3. Lighting: Balance the illumination with soft, natural overcast light to produce a gentle and even exposure.\n"),
        _ => prompt.push_str("3. Lighting: Apply professional studio lighting to enhance the forms in the scene.\n"),
    }
    prompt.push_str("\nFinal Instruction: Do not hallucinate new semantic objects. Modify the light, not the world.");
    prompt
}

pub fn build_stylize_prompt(settings: &PromptSettings) -> String {
    let mut prompt = String::new();
    prompt.push_str("System Role: You are a master art director and style-transfer engine.\n\n");
    
    let medium = if settings.target_medium.is_empty() { "3D Render" } else { &settings.target_medium };
    prompt.push_str(&format!("Objective: Transform the artistic medium of this image into {} while perfectly maintaining the original composition, poses, and layout.\n\n", medium));
    
    prompt.push_str("Rules:\n");
    prompt.push_str("1. Composition Lock: Maintain the exact framing, poses, and spatial relationships of all subjects.\n");
    prompt.push_str(&format!("2. Medium Transformation: Completely overwrite the original texture and shading to flawlessly simulate {}.\n", medium));
    prompt.push_str("3. Cohesion: Ensure the entire image shares a unified, consistent artistic style without any fragmented or un-stylized patches.\n");
    prompt
}

pub fn build_sketch_prompt(settings: &PromptSettings) -> String {
    let mut prompt = String::new();
    prompt.push_str("System Role: You are a concept artist and photorealistic rendering engine.\n\n");
    
    let render = if settings.render_style.is_empty() { "a highly detailed, photorealistic 4K masterpiece" } else { &settings.render_style };
    prompt.push_str(&format!("Objective: Interpret this rough sketch as a structural blueprint and render it into {}.\n\n", render));
    
    prompt.push_str("Rules:\n");
    prompt.push_str("1. Blueprint Interpretation: Use the lines and shapes of the sketch to determine the core subject and composition.\n");
    prompt.push_str("2. Texture Synthesis: Generate lifelike, high-frequency textures, realistic materials, and proper shading that the sketch implies but lacks.\n");
    prompt.push_str("3. Cohesive Illumination: Apply realistic global illumination and physically accurate shadows to give the flat sketch 3D volume and depth.\n");
    prompt
}

pub fn build_expand_prompt(_settings: &PromptSettings) -> String {
    let mut prompt = String::new();
    prompt.push_str("System Role: You are an expert outpainting and background-generation engine.\n\n");
    prompt.push_str("Objective: The provided image has been padded with blank space. Seamlessly outpaint and extend the environment into the blank margins to create a cohesive, expanded composition.\n\n");
    prompt.push_str("Rules:\n");
    prompt.push_str("1. Center Lock: Do not alter, overwrite, or enhance the original central pixels. They must remain exactly as they are.\n");
    prompt.push_str("2. Seamless Blending: Generate new content in the margins that perfectly matches the lighting, perspective, depth of field, and texture of the original image.\n");
    prompt.push_str("3. Contextual Extrapolation: Infer what should naturally exist just outside the original frame and render it convincingly without hallucinating distracting focal points.\n");
    prompt
}

pub fn build_system_prompt(style: &str, quality: &str, settings: &PromptSettings) -> String {
    let mut prompt = String::new();

    prompt.push_str("System Role: You are the Novura high-fidelity reconstruction and super-resolution engine.\n\n");
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
            target_medium: String::new(),
            render_style: String::new(),
            target_aspect_ratio: String::new(),
        };
        let prompt = build_system_prompt("PHOTOGRAPHY", "4K", &settings);
        
        assert!(prompt.contains("Novura high-fidelity reconstruction"));
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
            target_medium: String::new(),
            render_style: String::new(),
            target_aspect_ratio: String::new(),
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
            target_medium: String::new(),
            render_style: String::new(),
            target_aspect_ratio: String::new(),
        };
        let prompt = build_system_prompt("PHOTOGRAPHY", "4K", &settings);
        assert!(prompt.contains("Enhance edge clarity"));
        assert!(prompt.contains("professional studio high-key lighting"));
    }
}