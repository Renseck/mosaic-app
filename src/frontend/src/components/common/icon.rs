use yew::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub enum IconKind {
    AlertTriangle,
    Check,
    ChevronDown,
    Key,
    LayoutGrid,
    LogOut,
    Monitor,
    Moon,
    Plus,
    Settings,
    Sun,
    Template,
    Trash,
    User,
    Users,
    X,
}

#[derive(Properties, PartialEq)]
pub struct IconProps {
    pub kind: IconKind,
    #[prop_or_default]
    pub class: Classes,
}

#[function_component(Icon)]
pub fn icon(props: &IconProps) -> Html {
    // Each entry is a slice of path `d` values (most icons have one path; some have two).
    let paths: &[&str] = match &props.kind {
        IconKind::AlertTriangle => &[
            "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 \
             1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 \
             16c-.77 1.333.192 3 1.732 3z",
        ],
        IconKind::Check => &["M5 13l4 4L19 7"],
        IconKind::ChevronDown => &["M19 9l-7 7-7-7"],
        IconKind::Key => &[
            "M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 \
             1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z",
        ],
        IconKind::LayoutGrid => &[
            "M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 \
             01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 \
             2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 \
             2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 \
             012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z",
        ],
        IconKind::LogOut => &[
            "M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 \
             01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1",
        ],
        IconKind::Monitor => &[
            "M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 \
             002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z",
        ],
        IconKind::Moon => &[
            "M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 \
             21a9.003 9.003 0 008.354-5.646z",
        ],
        IconKind::Plus => &["M12 4v16m8-8H4"],
        IconKind::Settings => &[
            "M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 \
             0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 \
             0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 \
             0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 \
             0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 \
             0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 \
             0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 \
             0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 \
             2.296.07 2.572-1.065z",
            "M15 12a3 3 0 11-6 0 3 3 0 016 0z",
        ],
        IconKind::Sun => &[
            "M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 \
             6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707\
             M16 12a4 4 0 11-8 0 4 4 0 018 0z",
        ],
        IconKind::Template => &[
            "M9 17v-2m3 2v-4m3 4v-6m2 10H7a2 2 0 01-2-2V5a2 2 0 \
             012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 \
             01.293.707V19a2 2 0 01-2 2z",
        ],
        IconKind::Trash => &[
            "M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 \
             01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 \
             1 0 00-1 1v3M4 7h16",
        ],
        IconKind::User => &[
            "M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z",
        ],
        IconKind::Users => &[
            "M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 \
             6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z",
        ],
        IconKind::X => &["M6 18L18 6M6 6l12 12"],
    };

    html! {
        <svg class={classes!("w-5", "h-5", props.class.clone())}
             xmlns="http://www.w3.org/2000/svg" fill="none"
             viewBox="0 0 24 24" stroke="currentColor"
             stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            { for paths.iter().map(|d| html! { <path d={*d} /> }) }
        </svg>
    }
}
