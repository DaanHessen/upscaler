use leptos::prelude::*;
use crate::api::ApiClient;
use crate::auth::use_auth;
use crate::components::icons::{ShieldCheck, AlertCircle};

#[component]
pub fn AdminPanel() -> impl IntoView {
    let auth = use_auth();
    let token = move || auth.session.get().map(|s| s.access_token);

    let insights_res = LocalResource::new(move || {
        let t = token();
        async move {
            ApiClient::get_admin_insights(t.as_deref()).await
        }
    });

    view! {
        <div class="fade-in" style="max-width: 800px; margin: 0 auto; padding: var(--s-8);">
            <div class="settings-header">
                <ShieldCheck size={32} />
                <h1>"Admin Control Panel"</h1>
                <p class="muted">"System insights and moderation logs."</p>
            </div>

            <Suspense fallback=|| view! { <div class="loading-state">"Loading insights..."</div> }>
                {move || Suspend::new(async move {
                    match insights_res.await {
                        Ok(logs) => {
                            if logs.is_empty() {
                                view! { <p class="muted">"No recent moderation logs."</p> }.into_any()
                            } else {
                                view! {
                                    <div class="card settings-card" style="margin-top: var(--s-6);">
                                        <div class="params-body">
                                            <h3>"Moderation Logs"</h3>
                                            <p class="muted" style="margin-bottom: var(--s-6);">"Recent content filter rejections."</p>
                                            
                                            <div class="table-responsive">
                                                <table class="data-table" style="width: 100%; text-align: left; border-collapse: collapse;">
                                                    <thead>
                                                        <tr style="border-bottom: 1px solid var(--border); color: var(--muted);">
                                                            <th style="padding: var(--s-3) 0;">"Timestamp"</th>
                                                            <th style="padding: var(--s-3) 0;">"User ID"</th>
                                                            <th style="padding: var(--s-3) 0;">"Path/Reason"</th>
                                                        </tr>
                                                    </thead>
                                                    <tbody>
                                                        {logs.into_iter().map(|log| {
                                                            let created = log["created_at"].as_str().unwrap_or("Unknown").to_string();
                                                            let user_id = log["user_id"].as_str().unwrap_or("Unknown").to_string();
                                                            let path = log["path"].as_str().unwrap_or("Unknown").to_string();
                                                            
                                                            view! {
                                                                <tr style="border-bottom: 1px solid var(--border);">
                                                                    <td style="padding: var(--s-3) 0; font-family: var(--font-mono); font-size: 0.8rem;">{created}</td>
                                                                    <td style="padding: var(--s-3) 0; font-family: var(--font-mono); font-size: 0.8rem;">{user_id}</td>
                                                                    <td style="padding: var(--s-3) 0; color: hsl(var(--destructive));">{path}</td>
                                                                </tr>
                                                            }
                                                        }).collect_view()}
                                                    </tbody>
                                                </table>
                                            </div>
                                        </div>
                                    </div>
                                }.into_any()
                            }
                        }
                        Err(e) => {
                            view! {
                                <div class="error-state" style="padding: var(--s-6); text-align: center;">
                                    <AlertCircle size={48} />
                                    <h3 style="margin-top: var(--s-3);">"Access Denied"</h3>
                                    <p class="muted">{e}</p>
                                </div>
                            }.into_any()
                        }
                    }
                })}
            </Suspense>
        </div>
    }
}
