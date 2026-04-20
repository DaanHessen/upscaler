use leptos::prelude::*;
use crate::components::icons::ShieldCheck;

#[component]
pub fn Terms() -> impl IntoView {
    view! {
        <div class="legal-container fade-in">
            <div class="page-header">
                <h1 class="text-gradient">"Terms of Service"</h1>
                <p class="muted">"Service Protocol & Infrastructure usage guidelines."</p>
            </div>

            <div class="card legal-card">
                <div class="legal-section">
                    <h3>"1. Service Definition"</h3>
                    <p>"UPSYL Studio provides neural reconstruction and super-resolution services. By accessing our infrastructure, you agree to comply with our computational allocation protocols."</p>
                </div>

                <div class="legal-section">
                    <h3>"2. Credit Protocol"</h3>
                    <p>"Usage is metered via UNTS. Units are non-transferable and subject to expiration based on your specific acquisition tier. Reconstruction jobs are billed upon initiation."</p>
                </div>

                <div class="legal-section">
                    <h3>"3. Data Governance"</h3>
                    <p>"Processed assets are stored in our secure vault for a maximum of 24 hours. After this TTL (Time To Live) period, reconstruction data is permanently purged from our primary cache."</p>
                </div>

                <div class="legal-section">
                    <h3>"4. Acceptable Content"</h3>
                    <p>"Users are prohibited from processed content that violates our safety heuristics. Automated moderation is enforced on all ingress payloads."</p>
                </div>
            </div>

            <style>
                ".legal-container { max-width: 800px; margin: 0 auto; padding-bottom: var(--s-20); }
                .legal-card { padding: var(--s-10); display: flex; flex-direction: column; gap: var(--s-10); background: hsl(var(--surface-raised) / 0.3); }
                .legal-section h3 { font-family: var(--font-heading); font-size: 1rem; font-weight: 800; color: hsl(var(--text)); margin-bottom: var(--s-3); letter-spacing: -0.02em; }
                .legal-section p { font-size: 0.875rem; color: hsl(var(--text-muted)); line-height: 1.8; }
                "
            </style>
        </div>
    }
}

#[component]
pub fn Contact() -> impl IntoView {
    let (submitted, set_submitted) = signal(false);

    view! {
        <div class="legal-container fade-in">
            <div class="page-header">
                <h1 class="text-gradient">"Contact"</h1>
                <p class="muted">"Have questions or feedback? We'd love to hear from you."</p>
            </div>

            <div class="contact-grid">
                <div class="card contact-form-card">
                    {move || if submitted.get() {
                        view! {
                            <div class="success-panel" style="padding: 4rem 2rem;">
                                <ShieldCheck size={48} custom_style="color: var(--success); margin-bottom: 1rem;".to_string() />
                                <h3>"Message Sent"</h3>
                                <p>"Thanks for reaching out! We'll get back to you shortly."</p>
                                <button class="btn btn-secondary" style="margin-top: 2rem;" on:click=move |_| set_submitted.set(false)>"Send Another"</button>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <form class="contact-form" on:submit=move |ev| { ev.prevent_default(); set_submitted.set(true); }>
                                <div class="input-group">
                                    <label>"Email Address"</label>
                                    <input type="email" placeholder="you@example.com" required />
                                </div>
                                <div class="input-group">
                                    <label>"Subject"</label>
                                    <input type="text" placeholder="How can we help?" required />
                                </div>
                                <div class="input-group">
                                    <label>"Message"</label>
                                    <textarea placeholder="Tell us more about your request..." rows="5"></textarea>
                                </div>
                                <button type="submit" class="btn btn-primary" style="margin-top: var(--s-6); width: 100%;">"Send Message"</button>
                            </form>
                        }.into_any()
                    }}
                </div>

                <div class="contact-sidebar">
                    <div class="card bio-card">
                        <h3>"The Creator"</h3>
                        <p>"UPSYL started as a passion project to bring high-fidelity AI upscaling to everyone with a clean, no-nonsense interface."</p>
                        <p style="margin-top: 0.5rem; opacity: 0.7; font-style: italic;">"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua."</p>
                    </div>
                    
                    <a href="https://www.buymeacoffee.com" target="_blank" class="card info-card support-card">
                        <crate::components::icons::Coffee size={18} custom_style="color: hsl(var(--accent));".to_string() />
                        <div>
                            <span class="label">"Support the Project"</span>
                            <span class="value">"Buy me a coffee"</span>
                        </div>
                    </a>
                </div>
            </div>
            <style>
                ".contact-grid { display: grid; grid-template-columns: 1.5fr 1fr; gap: var(--s-8); }
                .contact-form-card { padding: var(--s-10); }
                .contact-form { display: flex; flex-direction: column; gap: var(--s-6); }
                .contact-form textarea { background: hsl(var(--bg)); border: 1px solid var(--glass-border); border-radius: var(--radius-md); padding: var(--s-4); color: white; font-family: inherit; font-size: 0.875rem; resize: none; }
                .contact-form textarea:focus { border-color: hsl(var(--accent)); outline: none; box-shadow: 0 0 0 4px hsl(var(--accent) / 0.1); }
                
                .contact-sidebar { display: flex; flex-direction: column; gap: var(--s-6); }
                .bio-card { padding: var(--s-8); background: hsl(var(--surface-raised) / 0.3); border: 1px solid var(--glass-border); }
                .bio-card h3 { font-size: 0.875rem; font-weight: 850; color: hsl(var(--text)); margin-bottom: var(--s-3); text-transform: uppercase; letter-spacing: 0.05em; }
                .bio-card p { font-size: 0.8125rem; color: hsl(var(--text-muted)); line-height: 1.6; }
                
                .info-card { padding: var(--s-6); display: flex; gap: var(--s-4); align-items: center; background: hsl(var(--surface-raised) / 0.5); border: 1px solid var(--glass-border); text-decoration: none; transition: all 0.2s; }
                .info-card:hover { border-color: hsl(var(--accent) / 0.4); background: hsl(var(--surface-raised) / 0.8); transform: translateY(-2px); }
                
                .info-card .label { display: block; font-size: 0.625rem; font-weight: 800; color: hsl(var(--text-dim)); text-transform: uppercase; letter-spacing: 0.1em; margin-bottom: 2px; }
                .info-card .value { font-size: 0.8125rem; font-weight: 700; color: hsl(var(--text)); font-family: var(--font-mono); }
                
                @media (max-width: 768px) {
                    .contact-grid { grid-template-columns: 1fr; }
                }
                "
            </style>
        </div>
    }
}
