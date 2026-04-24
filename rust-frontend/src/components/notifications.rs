use leptos::prelude::*;
use crate::GlobalState;
use crate::components::icons::AlertCircle;

#[component]
pub fn NotificationOverlay() -> impl IntoView {
    let gs = use_context::<GlobalState>().expect("GlobalState must be provided");
    let notification = gs.notification;

    // Auto-dismiss logic
    Effect::new(move |_| {
        if notification.get().is_some() {
            leptos::task::spawn_local(async move {
                gloo_timers::future::TimeoutFuture::new(5000).await;
                gs.clear_notification();
            });
        }
    });

    view! {
        <Show when=move || notification.get().is_some()>
            <div class="notification-portal">
                {move || notification.get().map(|(msg, n_type)| {
                    view! {
                        <div class=format!("notification-toast {}", n_type) on:click=move |_| gs.clear_notification()>
                            <div class="toast-icon">
                                <AlertCircle size={18} />
                            </div>
                            <div class="toast-content">
                                <span class="toast-message">{msg}</span>
                            </div>
                        </div>
                    }
                })}
            </div>
        </Show>
    }
}
