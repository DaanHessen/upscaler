use leptos::prelude::*;
use crate::components::icons::ShieldCheck;
use pulldown_cmark::{Parser, Options, html};

#[component]
fn MarkdownPage(title: &'static str, subtitle: &'static str, content: &'static str) -> impl IntoView {
    let html_content = {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        let parser = Parser::new_ext(content, options);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
        html_output
    };

    view! {
        <div class="legal-container fade-in">
            <div class="page-header">
                <h1 class="text-gradient">{title}</h1>
                <p class="muted">{subtitle}</p>
            </div>

            <div class="card legal-card">
                <div class="markdown-content" inner_html=html_content></div>
            </div>

            <style>
                ".legal-container { max-width: 900px; margin: 0 auto; padding-bottom: var(--s-20); }
                .legal-card { 
                    padding: var(--s-12); 
                    background: hsl(var(--surface-raised) / 0.3); 
                    border: 1px solid var(--glass-border);
                    box-shadow: var(--shadow-xl);
                }
                .markdown-content { 
                    font-size: 0.9375rem; 
                    color: hsl(var(--text-muted)); 
                    line-height: 1.8; 
                }
                .markdown-content h1, .markdown-content h2, .markdown-content h3 { 
                    color: hsl(var(--text)); 
                    margin-top: 2.5rem; 
                    margin-bottom: 1rem;
                    font-family: var(--font-heading);
                    font-weight: 800;
                    letter-spacing: -0.02em;
                }
                .markdown-content h1:first-child { margin-top: 0; }
                .markdown-content h2 { font-size: 1.25rem; border-bottom: 1px solid var(--glass-border); padding-bottom: 0.5rem; }
                .markdown-content h3 { font-size: 1rem; }
                .markdown-content p { margin-bottom: 1.25rem; }
                .markdown-content ul, .markdown-content ol { margin-bottom: 1.25rem; padding-left: 1.5rem; }
                .markdown-content li { margin-bottom: 0.5rem; }
                .markdown-content strong { color: hsl(var(--text)); font-weight: 700; }
                .markdown-content hr { border: 0; border-top: 1px solid var(--glass-border); margin: 3rem 0; }
                "
            </style>
        </div>
    }
}

#[component]
pub fn Terms() -> impl IntoView {
    let content = include_str!("../../../legal/terms-of-service.md");
    view! {
        <MarkdownPage 
            title="Terms of Service" 
            subtitle="The legal agreement governing your use of Upsyl."
            content=content
        />
    }
}

#[component]
pub fn Privacy() -> impl IntoView {
    let content = include_str!("../../../legal/privacy-policy.md");
    view! {
        <MarkdownPage 
            title="Privacy Policy" 
            subtitle="How we protect and manage your personal data."
            content=content
        />
    }
}

#[component]
pub fn AUP() -> impl IntoView {
    let content = include_str!("../../../legal/acceptable-use.md");
    view! {
        <MarkdownPage 
            title="Acceptable Use" 
            subtitle="Guidelines for responsible and safe service usage."
            content=content
        />
    }
}

#[component]
pub fn CookiePolicy() -> impl IntoView {
    let content = include_str!("../../../legal/cookie-policy.md");
    view! {
        <MarkdownPage 
            title="Cookie Policy" 
            subtitle="Information about how we use cookies and tracking."
            content=content
        />
    }
}

#[component]
pub fn RefundPolicy() -> impl IntoView {
    let content = include_str!("../../../legal/refund-policy.md");
    view! {
        <MarkdownPage 
            title="Refund Policy" 
            subtitle="Details on credit purchases and eligibility for refunds."
            content=content
        />
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
                            <div class="success-panel" style="padding: 4rem 2rem; text-align: center;">
                                <ShieldCheck size={48} custom_style="color: var(--success); margin-bottom: 1rem; display: block; margin-left: auto; margin-right: auto;".to_string() />
                                <h3 style="margin-bottom: 0.5rem; font-weight: 800;">"Message Sent"</h3>
                                <p style="font-size: 0.875rem; color: hsl(var(--text-muted));">"Thanks for reaching out! We'll get back to you shortly."</p>
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
                        <h3>"The Mission"</h3>
                        <p>"Upsyl is dedicated to democratizing high-fidelity AI upscaling. We believe in high-trust infrastructure and transparent data policies."</p>
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
                .contact-form-card { padding: var(--s-10); border: 1px solid var(--glass-border); }
                .contact-form { display: flex; flex-direction: column; gap: var(--s-6); }
                
                .contact-sidebar { display: flex; flex-direction: column; gap: var(--s-6); }
                .bio-card { padding: var(--s-8); background: hsl(var(--surface-raised) / 0.3); border: 1px solid var(--glass-border); }
                .bio-card h3 { font-size: 0.875rem; font-weight: 850; color: hsl(var(--text)); margin-bottom: var(--s-3); text-transform: uppercase; letter-spacing: 0.05em; }
                .bio-card p { font-size: 0.8125rem; color: hsl(var(--text-muted)); line-height: 1.6; }
                
                .info-card { padding: var(--s-6); display: flex; gap: var(--s-4); align-items: center; background: hsl(var(--surface-raised) / 0.5); border: 1px solid var(--glass-border); text-decoration: none; transition: all 0.2s; }
                .info-card:hover { border-color: hsl(var(--accent) / 0.4); background: hsl(var(--surface-raised) / 0.8); transform: translateY(-2px); }
                
                @media (max-width: 768px) {
                    .contact-grid { grid-template-columns: 1fr; }
                }
                "
            </style>
        </div>
    }
}
