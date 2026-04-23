use leptos::prelude::*;
use leptos::html;
use crate::components::icons::{ChevronLeft, ChevronRight};

#[component]
pub fn ComparisonSlider(
    images: Vec<(String, String)>,
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

    view! {
        <div 
            class="comparison-slider" 
            node_ref=slider_ref
            on:mousemove=on_move
            on:touchmove=on_touch
        >
            <div class="image-before" style:background-image=move || format!("url('{}')", current_pair_before()) style:background-color="#e1e1e4"></div>
            
            <div 
                class="image-after" 
                style:background-image=move || format!("url('{}')", current_pair_after())
                style:background-color="#f0f0f2"
                style:clip-path=move || format!("inset(0 0 0 {}%)", position.get())
            ></div>

            <span class="label before-label">"BEFORE"</span>
            <span class="label after-label">"AFTER"</span>

            // Navigation Buttons
            <div class="nav-btn prev-btn" on:click=prev title="Previous Image">
                <ChevronLeft size={20} />
            </div>
            <div class="nav-btn next-btn" on:click=next title="Next Image">
                <ChevronRight size={20} />
            </div>

            // Indicator dots
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
                    height: 100%;
                    min-height: 480px;
                    overflow: hidden;
                    cursor: ew-resize;
                    user-select: none;
                    background: hsl(var(--surface));
                    transition: transform 0.4s cubic-bezier(0.16, 1, 0.3, 1);
                    border-radius: var(--radius-lg);
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
                    box-shadow: inset 0 0 100px rgba(0,0,0,0.2);
                    transition: background-image 0.5s ease-in-out;
                }

                .label {
                    position: absolute;
                    bottom: var(--s-6);
                    padding: var(--s-2) var(--s-4);
                    background: rgba(10, 10, 12, 0.6);
                    backdrop-filter: blur(20px) saturate(180%);
                    color: hsl(var(--text));
                    font-size: 0.625rem;
                    font-weight: 850;
                    text-transform: uppercase;
                    letter-spacing: 0.15em;
                    border-radius: var(--radius-sm);
                    border: 1px solid rgba(255, 255, 255, 0.1);
                    pointer-events: none;
                    box-shadow: 0 4px 12px rgba(0,0,0,0.5);
                    transition: opacity 0.3s;
                    z-index: 5;
                }

                .before-label { left: var(--s-6); }
                .after-label { right: var(--s-6); }

                .nav-btn {
                    position: absolute;
                    top: 50%;
                    transform: translateY(-50%);
                    width: 52px;
                    height: 52px;
                    background: rgba(10, 10, 12, 0.4);
                    backdrop-filter: blur(20px) saturate(180%);
                    border: 1px solid rgba(255, 255, 255, 0.15);
                    border-radius: 50%;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    color: white;
                    cursor: pointer;
                    z-index: 40;
                    transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
                    opacity: 0.6;
                    box-shadow: 0 8px 32px rgba(0,0,0,0.3);
                }

                .comparison-slider:hover .nav-btn {
                    opacity: 1;
                }

                .nav-btn:hover {
                    background: rgba(255, 255, 255, 0.9);
                    border-color: white;
                    color: black;
                    transform: translateY(-50%) scale(1.15);
                    box-shadow: 0 0 30px rgba(255, 255, 255, 0.4);
                }

                .prev-btn { left: var(--s-8); }
                .next-btn { right: var(--s-8); }

                .slider-indicators {
                    position: absolute;
                    top: var(--s-6);
                    left: 50%;
                    transform: translateX(-50%);
                    display: flex;
                    gap: var(--s-2);
                    z-index: 20;
                }

                .indicator-dot {
                    width: 6px;
                    height: 6px;
                    border-radius: 50%;
                    background: rgba(255, 255, 255, 0.2);
                    cursor: pointer;
                    transition: all 0.3s;
                }

                .indicator-dot:hover {
                    background: rgba(255, 255, 255, 0.5);
                }

                .indicator-dot.active {
                    background: white;
                    width: 20px;
                    border-radius: 3px;
                }

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
