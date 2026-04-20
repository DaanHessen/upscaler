use leptos::prelude::*;

#[component]
pub fn ImageIcon(#[prop(default = 24)] size: u32, #[prop(optional)] custom_style: String) -> impl IntoView {
    view! {
        <svg 
            xmlns="http://www.w3.org/2000/svg" 
            width=size 
            height=size 
            viewBox="0 0 24 24" 
            fill="none" 
            stroke="currentColor" 
            stroke-width="2" 
            stroke-linecap="round" 
            stroke-linejoin="round"
            style=custom_style
        >
            <rect width="18" height="18" x="3" y="3" rx="2" ry="2"/>
            <circle cx="9" cy="9" r="2"/>
            <path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21"/>
        </svg>
    }
}

#[component]
pub fn Download(#[prop(default = 24)] size: u32) -> impl IntoView {
    view! {
        <svg 
            xmlns="http://www.w3.org/2000/svg" 
            width=size 
            height=size 
            viewBox="0 0 24 24" 
            fill="none" 
            stroke="currentColor" 
            stroke-width="2" 
            stroke-linecap="round" 
            stroke-linejoin="round"
        >
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
            <polyline points="7 10 12 15 17 10"/>
            <line x1="12" x2="12" y1="15" y2="3"/>
        </svg>
    }
}

#[component]
pub fn Calendar(#[prop(default = 24)] size: u32) -> impl IntoView {
    view! {
        <svg 
            xmlns="http://www.w3.org/2000/svg" 
            width=size 
            height=size 
            viewBox="0 0 24 24" 
            fill="none" 
            stroke="currentColor" 
            stroke-width="2" 
            stroke-linecap="round" 
            stroke-linejoin="round"
        >
            <rect width="18" height="18" x="3" y="4" rx="2" ry="2"/>
            <line x1="16" x2="16" y1="2" y2="6"/>
            <line x1="8" x2="8" y1="2" y2="6"/>
            <line x1="3" x2="21" y1="10" y2="10"/>
        </svg>
    }
}

#[component]
pub fn RefreshCw(#[prop(default = 24)] size: u32) -> impl IntoView {
    view! {
        <svg 
            xmlns="http://www.w3.org/2000/svg" 
            width=size 
            height=size 
            viewBox="0 0 24 24" 
            fill="none" 
            stroke="currentColor" 
            stroke-width="2" 
            stroke-linecap="round" 
            stroke-linejoin="round"
        >
            <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>
            <path d="M3 3v5h5"/>
            <path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16"/>
            <path d="M16 16h5v5"/>
        </svg>
    }
}

#[component]
pub fn AlertCircle(#[prop(default = 24)] size: u32, #[prop(optional)] custom_style: String) -> impl IntoView {
    view! {
        <svg 
            xmlns="http://www.w3.org/2000/svg" 
            width=size 
            height=size 
            viewBox="0 0 24 24" 
            fill="none" 
            stroke="currentColor" 
            stroke-width="2" 
            stroke-linecap="round" 
            stroke-linejoin="round"
            style=custom_style
        >
            <circle cx="12" cy="12" r="10"/>
            <line x1="12" x2="12" y1="8" y2="12"/>
            <line x1="12" x2="12.01" y1="16" y2="16"/>
        </svg>
    }
}

#[component]
pub fn Zap(#[prop(default = 24)] size: u32) -> impl IntoView {
    view! {
        <svg 
            xmlns="http://www.w3.org/2000/svg" 
            width=size 
            height=size 
            viewBox="0 0 24 24" 
            fill="none" 
            stroke="currentColor" 
            stroke-width="2" 
            stroke-linecap="round" 
            stroke-linejoin="round"
        >
            <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/>
        </svg>
    }
}

#[component]
pub fn ShieldCheck(#[prop(default = 24)] size: u32, #[prop(optional)] custom_style: String) -> impl IntoView {
    view! {
        <svg 
            xmlns="http://www.w3.org/2000/svg" 
            width=size 
            height=size 
            viewBox="0 0 24 24" 
            fill="none" 
            stroke="currentColor" 
            stroke-width="2" 
            stroke-linecap="round" 
            stroke-linejoin="round"
            style=custom_style
        >
            <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10"/>
            <path d="m9 12 2 2 4-4"/>
        </svg>
    }
}

#[component]
pub fn Info(#[prop(default = 24)] size: u32) -> impl IntoView {
    view! {
        <svg 
            xmlns="http://www.w3.org/2000/svg" 
            width=size 
            height=size 
            viewBox="0 0 24 24" 
            fill="none" 
            stroke="currentColor" 
            stroke-width="2" 
            stroke-linecap="round" 
            stroke-linejoin="round"
        >
            <circle cx="12" cy="12" r="10"/>
            <line x1="12" x2="12" y1="16" y2="12"/>
            <line x1="12" x2="12.01" y1="8" y2="8"/>
        </svg>
    }
}

#[component]
pub fn LayoutGrid(#[prop(default = 24)] size: u32) -> impl IntoView {
    view! {
        <svg 
            xmlns="http://www.w3.org/2000/svg" 
            width=size 
            height=size 
            viewBox="0 0 24 24" 
            fill="none" 
            stroke="currentColor" 
            stroke-width="2" 
            stroke-linecap="round" 
            stroke-linejoin="round"
        >
            <rect width="7" height="7" x="3" y="3" rx="1"/>
            <rect width="7" height="7" x="14" y="3" rx="1"/>
            <rect width="7" height="7" x="14" y="14" rx="1"/>
            <rect width="7" height="7" x="3" y="14" rx="1"/>
        </svg>
    }
}

#[component]
pub fn Upload(#[prop(default = 24)] size: u32) -> impl IntoView {
    view! {
        <svg 
            xmlns="http://www.w3.org/2000/svg" 
            width=size 
            height=size 
            viewBox="0 0 24 24" 
            fill="none" 
            stroke="currentColor" 
            stroke-width="2" 
            stroke-linecap="round" 
            stroke-linejoin="round"
        >
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
            <polyline points="17 8 12 3 7 8"/>
            <line x1="12" x2="12" y1="3" y2="15"/>
        </svg>
    }
}

#[component]
pub fn Settings(#[prop(default = 24)] size: u32) -> impl IntoView {
    view! {
        <svg 
            xmlns="http://www.w3.org/2000/svg" 
            width=size 
            height=size 
            viewBox="0 0 24 24" 
            fill="none" 
            stroke="currentColor" 
            stroke-width="2" 
            stroke-linecap="round" 
            stroke-linejoin="round"
        >
            <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/>
            <circle cx="12" cy="12" r="3"/>
        </svg>
    }
}
#[component]
pub fn CreditCard(#[prop(default = 24)] size: u32) -> impl IntoView {
    view! {
        <svg 
            xmlns="http://www.w3.org/2000/svg" 
            width=size 
            height=size 
            viewBox="0 0 24 24" 
            fill="none" 
            stroke="currentColor" 
            stroke-width="2" 
            stroke-linecap="round" 
            stroke-linejoin="round"
        >
            <rect width="20" height="14" x="2" y="5" rx="2"/>
            <line x1="2" x2="22" y1="10" y2="10"/>
        </svg>
    }
}
#[component]
pub fn HistoryIcon(#[prop(default = 24)] size: u32) -> impl IntoView {
    view! {
        <svg 
            xmlns="http://www.w3.org/2000/svg" 
            width=size 
            height=size 
            viewBox="0 0 24 24" 
            fill="none" 
            stroke="currentColor" 
            stroke-width="2" 
            stroke-linecap="round" 
            stroke-linejoin="round"
        >
            <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>
            <path d="M3 3v5h5"/>
            <polyline points="12 7 12 12 15 15"/>
        </svg>
    }
}

#[component]
pub fn LogOut(#[prop(default = 24)] size: u32) -> impl IntoView {
    view! {
        <svg 
            xmlns="http://www.w3.org/2000/svg" 
            width=size 
            height=size 
            viewBox="0 0 24 24" 
            fill="none" 
            stroke="currentColor" 
            stroke-width="2" 
            stroke-linecap="round" 
            stroke-linejoin="round"
        >
            <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/>
            <polyline points="16 17 21 12 16 7"/>
            <line x1="21" x2="9" y1="12" y2="12"/>
        </svg>
    }
}

#[component]
pub fn Mail(#[prop(default = 24)] size: u32, #[prop(optional)] custom_style: String) -> impl IntoView {
    view! {
        <svg 
            xmlns="http://www.w3.org/2000/svg" 
            width=size 
            height=size 
            viewBox="0 0 24 24" 
            fill="none" 
            stroke="currentColor" 
            stroke-width="2" 
            stroke-linecap="round" 
            stroke-linejoin="round"
            style=custom_style
        >
            <rect width="20" height="16" x="2" y="4" rx="2"/>
            <path d="m22 7-8.97 5.7a1.94 1.94 0 0 1-2.06 0L2 7"/>
        </svg>
    }
}

#[component]
pub fn MessageSquare(#[prop(default = 24)] size: u32, #[prop(optional)] custom_style: String) -> impl IntoView {
    view! {
        <svg 
            xmlns="http://www.w3.org/2000/svg" 
            width=size 
            height=size 
            viewBox="0 0 24 24" 
            fill="none" 
            stroke="currentColor" 
            stroke-width="2" 
            stroke-linecap="round" 
            stroke-linejoin="round"
            style=custom_style
        >
            <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
        </svg>
    }
}
#[component]
pub fn Coffee(#[prop(default = 24)] size: u32, #[prop(optional)] custom_style: String) -> impl IntoView {
    view! {
        <svg 
            xmlns="http://www.w3.org/2000/svg" 
            width=size 
            height=size 
            viewBox="0 0 24 24" 
            fill="none" 
            stroke="currentColor" 
            stroke-width="2" 
            stroke-linecap="round" 
            stroke-linejoin="round"
            style=custom_style
        >
            <path d="M17 8h1a4 4 0 1 1 0 8h-1"/>
            <path d="M3 8h14v9a4 4 0 0 1-4 4H7a4 4 0 0 1-4-4Z"/>
            <line x1="6" x2="6" y1="2" y2="4"/>
            <line x1="10" x2="10" y1="2" y2="4"/>
            <line x1="14" x2="14" y1="2" y2="4"/>
        </svg>
    }
}

#[component]
pub fn Maximize(#[prop(default = 24)] size: u32) -> impl IntoView {
    view! {
        <svg 
            xmlns="http://www.w3.org/2000/svg" 
            width=size 
            height=size 
            viewBox="0 0 24 24" 
            fill="none" 
            stroke="currentColor" 
            stroke-width="2" 
            stroke-linecap="round" 
            stroke-linejoin="round"
        >
            <path d="M8 3H5a2 2 0 0 0-2 2v3"/>
            <path d="M21 8V5a2 2 0 0 0-2-2h-3"/>
            <path d="M3 16v3a2 2 0 0 0 2 2h3"/>
            <path d="M16 21h3a2 2 0 0 0 2-2v-3"/>
        </svg>
    }
}

#[component]
pub fn Target(#[prop(default = 24)] size: u32) -> impl IntoView {
    view! {
        <svg 
            xmlns="http://www.w3.org/2000/svg" 
            width=size 
            height=size 
            viewBox="0 0 24 24" 
            fill="none" 
            stroke="currentColor" 
            stroke-width="2" 
            stroke-linecap="round" 
            stroke-linejoin="round"
        >
            <circle cx="12" cy="12" r="10"/>
            <circle cx="12" cy="12" r="6"/>
            <circle cx="12" cy="12" r="2"/>
        </svg>
    }
}

#[component]
pub fn Sun(#[prop(default = 24)] size: u32, #[prop(optional)] custom_style: String) -> impl IntoView {
    view! {
        <svg 
            xmlns="http://www.w3.org/2000/svg" 
            width=size 
            height=size 
            viewBox="0 0 24 24" 
            fill="none" 
            stroke="currentColor" 
            stroke-width="2" 
            stroke-linecap="round" 
            stroke-linejoin="round"
            style=custom_style
        >
            <circle cx="12" cy="12" r="4"/>
            <path d="M12 2v2"/>
            <path d="M12 20v2"/>
            <path d="m4.93 4.93 1.41 1.41"/>
            <path d="m17.66 17.66 1.41 1.41"/>
            <path d="M2 12h2"/>
            <path d="M20 12h2"/>
            <path d="m6.34 17.66-1.41 1.41"/>
            <path d="m19.07 4.93-1.41 1.41"/>
        </svg>
    }
}
