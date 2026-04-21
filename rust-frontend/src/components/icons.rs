use leptos::prelude::*;

#[component]
pub fn LoadingSpinner() -> impl IntoView {
    view! {
        <div class="studio-loader"></div>
    }
}


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
#[component]
pub fn ChevronLeft(#[prop(default = 24)] size: u32) -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" width=size height=size viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="15 18 9 12 15 6" />
        </svg>
    }
}

#[component]
pub fn ChevronRight(#[prop(default = 24)] size: u32) -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" width=size height=size viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="9 18 15 12 9 6" />
        </svg>
    }
}

#[component]
pub fn FileText(#[prop(default = 24)] size: u32) -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" width=size height=size viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z" />
            <polyline points="14 2 14 8 20 8" />
            <line x1="16" y1="13" x2="8" y2="13" />
            <line x1="16" y1="17" x2="8" y2="17" />
            <line x1="10" y1="9" x2="8" y2="9" />
        </svg>
    }
}

#[component]
pub fn Lock(#[prop(default = 24)] size: u32) -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" width=size height=size viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect width="18" height="11" x="3" y="11" rx="2" ry="2" />
            <path d="M7 11V7a5 5 0 0 1 10 0v4" />
        </svg>
    }
}

#[component]
pub fn GithubIcon(#[prop(default = 18)] size: u32) -> impl IntoView {
    view! {
        <svg viewBox="0 0 24 24" width=size height=size fill="currentColor">
            <path d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"/>
        </svg>
    }
}

#[component]
pub fn AppleIcon(#[prop(default = 18)] size: u32) -> impl IntoView {
    view! {
        <svg viewBox="0 0 24 24" width=size height=size fill="currentColor">
            <path d="M17.05 20.28c-.98.95-2.05.8-3.08.35-1.09-.46-2.09-.48-3.24 0-1.44.62-2.2.44-3.06-.35C4.24 16.73 3.65 10.15 6.64 8.7c1.37-.67 2.94-.38 3.7.11.45.29.98.37 1.44.11.7-.38 2.67-.86 4.14.54 1.32 1.3.9 4.3.17 5.2-.5 1.06-1.08 2.05-.1 3.62zM12.03 7.25c-.15-2.23 1.66-4.07 3.66-4.25.26 2.53-2.12 4.41-3.66 4.25z"/>
        </svg>
    }
}
#[component]
pub fn Moon(#[prop(default = 24)] size: u32, #[prop(optional)] custom_style: String) -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" width=size height=size viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style=custom_style>
            <path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z" />
        </svg>
    }
}

#[component]
pub fn UserIcon(#[prop(default = 24)] size: u32, #[prop(optional)] custom_style: String) -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" width=size height=size viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style=custom_style>
            <path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2" />
            <circle cx="12" cy="7" r="4" />
        </svg>
    }
}
