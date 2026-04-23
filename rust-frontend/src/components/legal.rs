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
                ".legal-container { max-width: 1000px; margin: 0 auto; padding-bottom: var(--s-20); }
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
                    margin-top: 2.5rem; 
                    margin-bottom: 1rem;
                    font-family: var(--font-heading);
                    font-weight: 800;
                    letter-spacing: -0.04em;
                    background: linear-gradient(135deg, hsl(var(--text)) 0%, hsl(var(--text-dim)) 25%, hsl(var(--text)) 50%, hsl(var(--text-dim)) 75%, hsl(var(--text)) 100%);
                    background-size: 200% auto;
                    -webkit-background-clip: text;
                    -webkit-text-fill-color: transparent;
                    padding-bottom: 0.1em;
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
                <div class="header-main">
                    <h1 class="stagger-1 text-gradient">"Support Request"</h1>
                    <p class="muted stagger-2">"Priority assistance for Upsyl Studio usage, billing, and technical integrations."</p>
                </div>
            </div>

            <div class="card shadow-lg stagger-3 contact-form-card" style="padding: var(--s-10) var(--s-12); max-width: 700px; margin-top: var(--s-8);">
                {move || if submitted.get() {
                    view! {
                        <div class="success-panel" style="padding: 4rem 2rem; text-align: center;">
                            <ShieldCheck size={48} custom_style="color: var(--success); margin-bottom: 2rem; display: block; margin-left: auto; margin-right: auto;".to_string() />
                            <h3 class="text-gradient" style="margin-bottom: 0.5rem; font-size: 1.5rem; letter-spacing: -0.02em;">"Message Received"</h3>
                            <p style="color: hsl(var(--text-muted)); font-size: 0.9375rem; margin-bottom: 2rem; line-height: 1.6;">"Our engineering and support team has been notified. We will contact you at your provided email address shortly."</p>
                            <button class="btn btn-secondary" on:click=move |_| set_submitted.set(false)>"Send Another Request"</button>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <form class="contact-form" on:submit=move |ev| { ev.prevent_default(); set_submitted.set(true); }>
                            <div class="input-group">
                                <label>"Email Verification"</label>
                                <input type="email" placeholder="studio-admin@example.com" required style="width: 100%;" />
                            </div>
                            <div class="input-group" style="margin-top: var(--s-6);">
                                <label>"Subject Context"</label>
                                <input type="text" placeholder="Billing inquiry, API integration, or bug report" required style="width: 100%;" />
                            </div>
                            <div class="input-group" style="margin-top: var(--s-6);">
                                <label>"Diagnostic Details"</label>
                                <textarea placeholder="Please describe the issue in detail, including relevant Job IDs or Timestamps if applicable." rows="6" style="width: 100%; resize: vertical;"></textarea>
                            </div>
                            <div style="margin-top: var(--s-8); display: flex; align-items: center; justify-content: space-between;">
                                <p style="font-size: 0.75rem; color: hsl(var(--text-muted)); margin: 0;">"Expected response time: < 24 hours"</p>
                                <button type="submit" class="btn btn-primary" style="padding-left: var(--s-8); padding-right: var(--s-8);">"Submit Request"</button>
                            </div>
                        </form>
                    }.into_any()
                }}
            </div>
            
            <style>
                ".contact-form-card { border: 1px solid var(--glass-border); background: hsl(var(--surface)); }
                .input-group label { font-size: 0.75rem; font-weight: 850; letter-spacing: 0.05em; text-transform: uppercase; margin-bottom: var(--s-2); display: block; color: hsl(var(--text-dim)); }
                .input-group input, .input-group textarea { background: hsl(var(--surface-bright)); border: 1px solid var(--border); border-radius: var(--radius-md); padding: var(--s-3) var(--s-4); color: hsl(var(--text)); font-family: var(--font-sans); }
                .input-group input:focus, .input-group textarea:focus { outline: none; border-color: hsl(var(--accent)); box-shadow: 0 0 0 2px hsl(var(--accent) / 0.1); }
                "
            </style>
        </div>
    }
}
