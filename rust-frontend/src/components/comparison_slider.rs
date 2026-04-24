use leptos::prelude::*;
use leptos::html;
use crate::components::icons::{ChevronLeft, ChevronRight};

#[component]
pub fn ComparisonSlider(
    images: Vec<(String, String)>,
    #[prop(default = 1.0)] zoom: f64,
    #[prop(into, default = "compare".to_string().into())] view_mode: Signal<String>,
) -> impl IntoView {
    let (current_index, set_current_index) = signal(0usize);
    let (position, set_position) = signal(50.0);
    let slider_ref = NodeRef::<html::Div>::new();

    let images_count = images.len();
    let images_before = images.clone();
    let current_pair_before = move || images_before.get(current_index.get()).map(|(b, _)| b.clone()).unwrap_or_default();
    let images_after = images;
    let current_pair_after = move || images_after.get(current_index.get()).map(|(_, a)| a.clone()).unwrap_or_default();

    let on_move = move |ev: web_sys::MouseEvent| {
        if let Some(slider) = slider_ref.get() {
            let rect = slider.get_bounding_client_rect();
            let x = ev.client_x() as f64 - rect.left();
            let new_pos = (x / rect.width() * 100.0).clamp(0.0, 100.0);
            set_position.set(new_pos);
        }
    };

    let on_touch = move |ev: web_sys::TouchEvent| {
        if let Some(slider) = slider_ref.get() {
            let touches = ev.touches();
            if let Some(touch) = touches.get(0) {
                let rect = slider.get_bounding_client_rect();
                let x = touch.client_x() as f64 - rect.left();
                let new_pos = (x / rect.width() * 100.0).clamp(0.0, 100.0);
                set_position.set(new_pos);
            }
        }
    };

    let prev = move |ev: web_sys::MouseEvent| {
        ev.stop_propagation();
        set_current_index.update(|i| {
            if *i == 0 {
                *i = images_count - 1;
            } else {
                *i -= 1;
            }
        });
    };

    let next = move |ev: web_sys::MouseEvent| {
        ev.stop_propagation();
        set_current_index.update(|i| {
            *i = (*i + 1) % images_count;
        });
    };

    let is_multi_image = images_count > 1;

    let vm = move || view_mode.get();

    view! {
        <div 
            class="comparison-slider" 
            node_ref=slider_ref
            on:mousemove=on_move
            on:touchmove=on_touch
            style=move || format!("--zoom: {};", zoom)
            style:cursor={move || if zoom > 1.0 { "grab" } else { "ew-resize" }}
        >
            <div 
                class="image-before" 
                style:background-image=move || format!("url('{}')", current_pair_before()) 
                style:background-color="#e1e1e4"
                style:transform=move || format!("scale({})", zoom)
            ></div>
            
            <div 
                class="image-after" 
                style:background-image=move || format!("url('{}')", current_pair_after())
                style:background-color="#f0f0f2"
                style:clip-path=move || {
                    match vm().as_str() {
                        "original" => "inset(0 0 0 100%)".to_string(),
                        "upscaled" => "inset(0 0 0 0%)".to_string(),
                        _ => format!("inset(0 0 0 {}%)", position.get())
                    }
                }
                style:transform=move || format!("scale({})", zoom)
            ></div>

            <Show when=move || vm() == "compare">
                <span class="label before-label">"BEFORE"</span>
                <span class="label after-label">"AFTER"</span>
            </Show>

            // Navigation Buttons
            <Show when=move || is_multi_image>
                <div class="nav-btn prev-btn" on:click=prev title="Previous Image">
                    <ChevronLeft size={20} />
                </div>
                <div class="nav-btn next-btn" on:click=next title="Next Image">
                    <ChevronRight size={20} />
                </div>
            </Show>

            // Indicator dots
            <Show when=move || is_multi_image>
                <div class="slider-indicators">
                    {
                        (0..images_count).map(|i| {
                            view! {
                                <div 
                                    class="indicator-dot" 
                                    class:active=move || current_index.get() == i
                                    on:click=move |ev| {
                                        ev.stop_propagation();
                                        set_current_index.set(i);
                                    }
                                ></div>
                            }
                        }).collect_view()
                    }
                </div>
            </Show>

            <Show when=move || vm() == "compare">
                <div class="slider-handle" style:left=move || format!("{}%", position.get())>
                    <div class="handle-circle">
                        <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="4">
                            <path d="M11 5l-7 7 7 7M13 5l7 7-7 7" />
                        </svg>
                    </div>
                </div>
            </Show>
        </div>
    }
}
