# High-Fidelity Generative Upscaling: Prompt Engineering Research (V2.1)

## 1. Executive Summary
The transition to generative upscaling (Pruna AI / Flux-based pipelines) requires a shift from "Resolution Prompts" to "Material Preservation Prompts." The current artifacts (darkening, plastic skin, loss of micro-details) are symptoms of a prompt that gives the AI too much creative freedom over lighting and smoothing.

## 2. The Darkening Issue: Causes and Mitigation
**Root Cause:** Generative models often interpret "detail enhancement" as "contrast enhancement." Without explicit lighting instructions, the AI adds "cinematic" shadows which darken the mid-tones and crush blacks.

### Mitigation Strategies:
*   **Lighting Anchors:** Use `high-key lighting`, `balanced exposure`, and `soft natural daylight` to force the AI to keep the image bright.
*   **Dynamic Range Protection:** Explicitly state `preserve original shadows`, `no contrast boost`, and `maintain highlight detail`.
*   **Neutral Grading:** Use `unprocessed raw photograph`, `neutral color balance`, and `flat lighting profile` to prevent the AI from applying its own creative "look."

## 3. Texture Preservation: Defeating the "Plastic" Look
**The Challenge:** AI models are trained on millions of aesthetic images that are often smoothed or airbrushed. By default, they "clean" noise, which includes natural skin pores and fine hair.

### The "Texture Locking" Vocabulary:
| Feature | Positive Keywords | Negative Keywords (To Block) |
| :--- | :--- | :--- |
| **Skin** | `visible pores`, `realistic skin micro-texture`, `natural imperfections`, `raw skin detail` | `plastic skin`, `airbrushed`, `waxiness`, `smeared skin`, `beauty filter` |
| **Hair** | `individual hair strands`, `fine hair follicles`, `sharp hair edges`, `natural hair flow` | `matted hair`, `clumped texture`, `smeared hair`, `smooth hair chunks` |
| **Eyes** | `sharp pupil detail`, `natural moisture`, `realistic iris texture`, `crisp catchlights` | `cartoon eyes`, `blurry gaze`, `glassy look` |

## 4. "De-AI-fying" the Aesthetic
To prevent the image from looking "AI-generated," we must inject "Analog Signals" that represent physical reality rather than digital perfection.

*   **Analog Keywords:** `shot on 35mm film`, `Fujifilm aesthetic`, `slight film grain`, `analog photography`, `raw sensor data`.
*   **Physicality Keywords:** `tactile surfaces`, `organic materials`, `physical textures`, `imperfections`.

## 5. Resolution-Specific Strategies
### Low-Res (256px - 512px) - "The Reconstruction Pass"
*   **Goal:** Rebuild missing structure without changing identity.
*   **Key Prompt:** `Structural restoration`, `identity-locked reconstruction`, `rebuild missing high-frequency data from image 1`.
*   **Denoising/Creativity:** Needs higher creativity (0.5 - 0.7) but must be anchored by strong `identity preservation` keywords.

### Med-Res (1024px+) - "The Enhancement Pass"
*   **Goal:** Polish existing data without adding new features.
*   **Key Prompt:** `Micro-detail refinement`, `gentle sharpening`, `upscale existing textures only`, `no new features`.
*   **Denoising/Creativity:** Keep low (0.2 - 0.4) to prevent "drifting" away from the source.

## 6. Proposed LoRA Strategy
The current "HotSwap LoRA" error indicates that we need to use valid HuggingFace paths if we switch to the `p-image-edit-lora` model.
*   **Recommended LoRA:** `davidberenstein1957/p-image-edit-photo-enhancement-lora`
*   **Why:** This LoRA is specifically trained to add "photorealistic grit" and "film-like sharpness" without the typical AI smoothness.
*   **Action:** If we adopt this, we must update the `replicate.rs` client to point to the `lora_weights` URL.

## 7. The "New Golden Prompt" Template
```text
(Positive)
Modify image 1 with ultra-high-fidelity enhancement. Strictly preserve the original soul, identity, and lighting. 
Reconstruct [caption] with visible skin pores, individual hair strands, and sharp optical clarity. 
Well-lit, high-key lighting, balanced exposure, natural raw photo aesthetic, shot on 35mm lens. 
(Negative)
plastic skin, airbrushed, waxiness, smeared details, over-sharpened, etched textures, cinematic lighting, 
dramatic shadows, color shift, cartoonish, digital art look, beauty filter, fake textures.
```

## 8. Conclusion
The next step is to test this "Golden Prompt" on the dog and Statue of Liberty samples. The focus must be on **Additive Realism** (adding pores/hair) rather than **Subtractive Cleaning** (smoothing).
