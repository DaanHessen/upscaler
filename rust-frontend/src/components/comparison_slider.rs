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
            <div class="image-after" style:background-image=move || format!("url({})", after_url)></div>
            
            <div 
                class="image-before" 
                style:background-image=move || format!("url({})", before_url)
                style:width=move || format!("{}%", position.get())
            ></div>

            <span class="label before-label">{before_label}</span>
            <span class="label after-label">{after_label}</span>

            <div class="slider-handle" style:left=move || format!("{}%", position.get())>
                <div class="handle-circle">
                    <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="4">
                        <path d="M11 5l-7 7 7 7M13 5l7 7-7 7" />
                    </svg>
                </div>
            </div>

            <style>
                ".comparison-slider {
                    position: relative;
                    width: 100%;
                    max-width: 1040px;
                    aspect-ratio: 16/10;
                    margin: 0 auto;
                    overflow: hidden;
                    border-radius: var(--radius-lg);
                    border: 1px solid var(--glass-border);
                    box-shadow: var(--shadow-xl);
                    cursor: ew-resize;
                    user-select: none;
                    background: hsl(var(--surface));
                    transition: transform 0.4s cubic-bezier(0.16, 1, 0.3, 1), box-shadow 0.4s;
                }
                .comparison-slider:hover {
                    box-shadow: 0 40px 80px -20px rgba(0,0,0,0.9);
                }

                .image-before, .image-after {
                    position: absolute;
                    top: 0;
                    left: 0;
                    width: 100%;
                    height: 100%;
                    background-size: cover;
                    background-position: center;
                    background-repeat: no-repeat;
                }

                .label {
                    position: absolute;
                    bottom: var(--s-6);
                    padding: var(--s-2) var(--s-4);
                    background: rgba(10, 10, 12, 0.4);
                    backdrop-filter: blur(20px) saturate(180%);
                    color: hsl(var(--text));
                    font-size: 0.625rem;
                    font-weight: 800;
                    text-transform: uppercase;
                    letter-spacing: 0.15em;
                    border-radius: var(--radius-sm);
                    border: 1px solid rgba(255, 255, 255, 0.1);
                    pointer-events: none;
                    box-shadow: 0 4px 12px rgba(0,0,0,0.5);
                    transition: opacity 0.3s;
                }

                .before-label { left: var(--s-6); }
                .after-label { right: var(--s-6); }

                .slider-handle {
                    position: absolute;
                    top: 0;
                    bottom: 0;
                    width: 2px;
                    z-index: 10;
                    pointer-events: none;
                    background: white;
                    box-shadow: 0 0 15px white, 0 0 30px rgba(255, 255, 255, 0.5);
                }

                .handle-circle {
                    position: absolute;
                    top: 50%;
                    left: 50%;
                    transform: translate(-50%, -50%);
                    width: 44px;
                    height: 44px;
                    background: rgba(255, 255, 255, 0.1);
                    backdrop-filter: blur(12px);
                    border: 1px solid rgba(255, 255, 255, 0.2);
                    border-radius: 50%;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    color: white;
                    box-shadow: 0 0 20px rgba(0,0,0,0.5);
                    transition: all 0.2s;
                }
                .comparison-slider:hover .handle-circle {
                    background: rgba(255, 255, 255, 0.2);
                    transform: translate(-50%, -50%) scale(1.1);
                    border-color: rgba(255, 255, 255, 0.4);
                }
                "
            </style>
        </div>
    }
}
