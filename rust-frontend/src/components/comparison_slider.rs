use leptos::prelude::*;
use leptos::html;

#[component]
pub fn ComparisonSlider(
    before_url: String,
    after_url: String,
    #[prop(default = "Before")] before_label: &'static str,
    #[prop(default = "After")] after_label: &'static str,
) -> impl IntoView {
    let (position, set_position) = signal(50.0);
    let slider_ref = NodeRef::<html::Div>::new();

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

    view! {
        <div 
            class="comparison-slider" 
            node_ref=slider_ref
            on:mousemove=on_move
            on:touchmove=on_touch
        >
            <div class="image-after" style:background-image=move || format!("url({})", after_url)>
                <span class="label after-label">{after_label}</span>
            </div>
            
            <div 
                class="image-before" 
                style:background-image=move || format!("url({})", before_url)
                style:width=move || format!("{}%", position.get())
            >
                <span class="label before-label">{before_label}</span>
            </div>

            <div class="slider-handle" style:left=move || format!("{}%", position.get())>
                <div class="handle-line"></div>
                <div class="handle-circle">
                    <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="3">
                        <path d="M11 5l-7 7 7 7M13 5l7 7-7 7" />
                    </svg>
                </div>
            </div>

            <style>
                ".comparison-slider {
                    position: relative;
                    width: 100%;
                    max-width: 1000px;
                    aspect-ratio: 16/9;
                    margin: 0 auto;
                    overflow: hidden;
                    border-radius: 12px;
                    border: 1px solid var(--border-color);
                    box-shadow: var(--shadow);
                    cursor: ew-resize;
                    user-select: none;
                }

                .image-before, .image-after {
                    position: absolute;
                    top: 0;
                    left: 0;
                    width: 100%;
                    height: 100%;
                    background-size: cover;
                    background-position: center;
                }

                .image-before {
                    border-right: 1px solid rgba(255,255,255,0.2);
                }

                .label {
                    position: absolute;
                    bottom: 1.5rem;
                    padding: 0.4rem 0.8rem;
                    background: rgba(0, 0, 0, 0.6);
                    backdrop-filter: blur(4px);
                    color: white;
                    font-size: 0.7rem;
                    font-weight: 700;
                    text-transform: uppercase;
                    letter-spacing: 0.1em;
                    border-radius: 4px;
                    border: 1px solid rgba(255,255,255,0.1);
                    pointer-events: none;
                }

                .before-label { left: 1.5rem; }
                .after-label { right: 1.5rem; }

                .slider-handle {
                    position: absolute;
                    top: 0;
                    bottom: 0;
                    width: 2px;
                    z-index: 10;
                    pointer-events: none;
                }

                .handle-line {
                    height: 100%;
                    width: 2px;
                    background: white;
                    box-shadow: 0 0 10px rgba(0,0,0,0.5);
                }

                .handle-circle {
                    position: absolute;
                    top: 50%;
                    left: 50%;
                    transform: translate(-50%, -50%);
                    width: 40px;
                    height: 40px;
                    background: white;
                    border-radius: 50%;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    color: black;
                    box-shadow: 0 4px 12px rgba(0,0,0,0.3);
                }
                "
            </style>
        </div>
    }
}
