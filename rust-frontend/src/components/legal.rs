use leptos::prelude::*;
use crate::components::icons::ShieldCheck;
use crate::auth::use_auth;
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
                <div class="header-main">
                    <h1 class="stagger-1 text-gradient">{title}</h1>
                    <p class="muted stagger-2">{subtitle}</p>
                </div>
            </div>

            <div class="card legal-card">
                <div class="markdown-content" inner_html=html_content></div>
            </div>

            
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
    let auth = use_auth();
    let (submitted, set_submitted) = signal(false);

    view! {
        <div class="history-container fade-in">
            <div class="page-header">
                <div class="header-main">
                    <h1 class="stagger-1 text-gradient">"Support Request"</h1>
                    <p class="muted stagger-2">"Priority assistance for Upsyl Studio usage, billing, and technical integrations."</p>
                </div>
            </div>

            <div class="card shadow-lg stagger-3 contact-form-card" style="padding: var(--s-10) var(--s-12); max-width: 100%; margin: var(--s-8) auto 0 auto;">
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
                                <label>"Related Job ID"</label>
                                <select style="width: 100%;">
                                    <option value="">"No specific job / General inquiry"</option>
                                    {move || {
                                        let auth = auth.clone();
                                        auth.history.get().into_iter().flatten().map(|item| {
                                            let id_short = item.id.to_string()[..8].to_string().to_uppercase();
                                            let ts = item.created_at.clone();
                                            let style = item.style.clone().unwrap_or_else(|| "AUTO".to_string());
                                            view! {
                                                <option value=item.id.to_string()>
                                                    {format!("[{}] {} - #{}", style, ts, id_short)}
                                                </option>
                                            }
                                        }).collect_view()
                                    }}
                                </select>
                            </div>
                            <div class="input-group" style="margin-top: var(--s-8);">
                                <label>"Diagnostic Details"</label>
                                <textarea placeholder="Please describe the issue in detail. If you selected a Job ID above, we will automatically attach the technical logs." rows="6" style="width: 100%; resize: vertical;"></textarea>
                            </div>
                            <div style="margin-top: var(--s-8); display: flex; align-items: center; justify-content: space-between;">
                                <p style="font-size: 0.75rem; color: hsl(var(--text-muted)); margin: 0;">"Expected response time: < 24 hours"</p>
                                <button type="submit" class="btn btn-primary" style="padding-left: var(--s-8); padding-right: var(--s-8);">"Submit Request"</button>
                            </div>
                        </form>
                    }.into_any()
                }}
            </div>
            
            
        </div>
    }
}
