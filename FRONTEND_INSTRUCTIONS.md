# Agent Instructions: Professional Rust Frontend Implementation

You are tasked with replacing the current placeholder frontend with an **exceptionally professional, high-performance web application built in Rust**. This application must serve as the flagship interface for a Gemini-powered image upscaling SaaS.

## Core Objective
Transform the `frontend/` directory into a modern, production-grade application that feels premium and "carefully crafted." The design philosophy is **"Less is More"**—clean lines, intuitive flows, and no visual clutter.

---

## Technical Architecture
- **Language**: Rust (Target: WASM).
- **Framework Recommendation**: [Leptos](https://leptos.dev/) or [Dioxus](https://dioxuslabs.com/) for a high-performance "Rust-all-the-way" experience.
- **Backend Integration**: The existing Axum backend is located in `src/`. You must integrate with its JSON/Multipart API.
- **Authentication**: Supabase Auth (JWT-based). The backend already validates these tokens.

---

## Visual Design System
- **Theme**: Premium Dark Mode. Avoid flat pure black; use deep charcoal (`#0a0a0b`) and slate levels for depth.
- **Accents**: Use a single, vibrant accent color (e.g., Electric Violet or Emerald Green) for primary actions.
- **Typography**: Modern sans-serif (e.g., Inter, Outfit, or Roboto).
- **Aesthetics**: Glassmorphism, subtle micro-animations (Framer Motion style transitions), and high-quality iconography (Lucide/Heroicons).

---

## Feature Roadmap & UX Flow

### 1. The Global Header
- App Logo (Gemini Upscaler).
- User Profile / Logout (Supabase integrated).
- **Credit Balance Counter**: Displays current credits (via `GET /balance`). Animates when credits are spent.

### 2. The Landing/Auth State
- If not logged in: Beautiful hero section with a "Sign in with Google/Email" card. Use Supabase Auth UI patterns.

### 3. The "Smart" Upload Flow (The Core UX)
- **Dropzone**: A sleek, animated drag-and-drop area.
- **Instant Moderation**: Upon file selection, immediately hit `POST /moderate`.
    - Show a subtle "Analyzing image..." pulse.
    - Display result: "Detected Style: **Illustration**" or "Detected Style: **Photography**".
    - Allow the user to manually toggle/correct this via a toggle switch or pill buttons.
- **Configuration Picker**:
    - Quality (1K, 2K, 4K).
    - Temperature (0.0 to 2.0 slider).
    - Dynamic Cost Calculator: Show the credit cost (e.g., "Cost: 2 Credits") updating in real-time.

### 4. The Processing State
- After hitting "Upscale" (`POST /upscale`):
    - Transition to a processing view.
    - Poll `GET /upscales/:job_id`.
    - Show a progressive enhancement UI or a sophisticated "enhancing" animation.

### 5. Result & History
- **Result Details**: Large side-by-side comparison (Before/After) if possible, or a beautiful high-res preview with a "Download" button.
- **History Gallery**: A masonry grid (`GET /history`) of past upscales. Use lazy loading for images.

### 6. Billing/Store
- A simple, clean "Add Credits" section.
- Tiered pricing cards (e.g., 50 Credits, 200 Credits).
- Integration with `POST /checkout` -> Redirect to Stripe.

---

## API Reference for Implementation

| Endpoint | Method | Input | Description |
| :--- | :--- | :--- | :--- |
| `/moderate` | `POST` | Multipart (`image`) | Returns `{ nsfw: bool, detected_style: "ILLUSTRATION" | "PHOTOGRAPHY" }` |
| `/upscale` | `POST` | Multipart (`image`, `quality`, `temperature`, `style`?) | Returns `{ success: bool, job_id, final_style }`. `style` is optional override. |
| `/upscales/:id` | `GET` | N/A | Returns `{ status, image_url?, error? }`. |
| `/balance` | `GET` | N/A | Returns `{ credits: i32 }`. |
| `/history` | `GET` | N/A | Returns list of past upscale records. |
| `/checkout` | `POST` | `{ tier: string }` | Returns `{ url: string }` for Stripe redirect. |

---

## Final Instruction
**Do not settle for a basic MVP.** The user should feel like they are using a high-end creative tool. Prioritize smooth transitions between states and a perfectly responsive layout. Overwrite all files in `frontend/` as necessary.
