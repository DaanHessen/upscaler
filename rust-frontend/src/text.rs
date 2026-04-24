pub struct UiText {
    pub brand_name: &'static str,
    pub brand_suffix: &'static str,

    // Navigation
    pub nav_home: &'static str,
    pub nav_editor: &'static str,
    pub nav_history: &'static str,
    pub nav_billing: &'static str,

    // Home Page
    pub home_hero_title: &'static str,
    pub home_hero_subtitle: &'static str,
    pub home_cta_start: &'static str,
    pub home_features_title: &'static str,

    // Editor Page
    pub editor_page_title: &'static str,
    pub editor_page_subtitle: &'static str,
    pub editor_empty_title: &'static str,
    pub editor_empty_desc: &'static str,
    pub editor_sidebar_title: &'static str,
    
    // Editor Controls
    pub label_resolution: &'static str,
    pub label_style: &'static str,
    pub label_creativity: &'static str,
    pub label_seed: &'static str,
    pub label_locks: &'static str,
    pub label_lighting: &'static str,

    pub label_credits: &'static str,

    // Actions
    pub action_sign_out: &'static str,
    pub action_go_dashboard: &'static str,
    pub action_view_gallery: &'static str,

    // Descriptions (Tooltips)
    pub desc_resolution: &'static str,
    pub desc_style: &'static str,
    pub desc_creativity: &'static str,
    pub desc_seed: &'static str,
    pub desc_locks: &'static str,
    pub desc_lighting: &'static str,

    // Footer
    pub footer_rights: &'static str,
}

pub static TXT: UiText = UiText {
    brand_name: "Novura",
    brand_suffix: "STUDIO",

    nav_home: "HOME",
    nav_editor: "EDITOR",
    nav_history: "HISTORY",
    nav_billing: "BILLING",

    home_hero_title: "INFINITE RESOLUTION",
    home_hero_subtitle: "Harness the power of Gemini 3.1 Vision for high-fidelity 4K upscaling, restyling, and detail synthesis.",
    home_cta_start: "ENTER STUDIO",
    home_features_title: "ENGINE CAPABILITIES",

    editor_page_title: "Studio Editor",
    editor_page_subtitle: "Fine-tune reconstruction parameters and export high-fidelity assets.",
    editor_empty_title: "Studio Canvas Empty",
    editor_empty_desc: "Drag and drop an image here or click to select a file.",
    editor_sidebar_title: "Configuration",

    label_resolution: "TARGET RESOLUTION",
    label_style: "RECONSTRUCTION STYLE",
    label_creativity: "CREATIVITY",
    label_seed: "SEED",
    label_locks: "ADVANCED ENGINE LOCKS",
    label_lighting: "ATMOSPHERIC LIGHTING",
    label_credits: "CREDITS",

    action_sign_out: "SIGN OUT",
    action_go_dashboard: "GO TO DASHBOARD",
    action_view_gallery: "VIEW IN GALLERY",

    desc_resolution: "Higher resolution requires more GPU processing credits.",
    desc_style: "Photography optimizes for realism, Illustration for clean line art.",
    desc_creativity: "Controls how much detail the AI synthesizes vs strictly following the original.",
    desc_seed: "Use a fixed seed to ensure reproducible results for the same parameters.",
    desc_locks: "Preserves specific image characteristics like depth of field or focal planes.",
    desc_lighting: "Adjusts the atmospheric mood and light distribution of the result.",

    footer_rights: "All rights reserved. System v2.2.0-Alpha.",
};
