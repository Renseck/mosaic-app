use yew::prelude::*;
use yew_router::prelude::*;
use crate::{context::auth_context::AuthContext, router::Route};

#[derive(Properties, PartialEq)]
pub struct SidebarProps {
    pub is_open: bool,
    pub on_close: Callback<()>,
}

#[function_component(Sidebar)]
pub fn sidebar(props: &SidebarProps) -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext missing");
    let is_admin = auth.user.as_ref().map(|u| u.role.is_admin()).unwrap_or(false);

    // On desktop (md+) always visible via CSS; on mobile slide in/out.
    let aside_class = if props.is_open {
        "fixed inset-y-0 left-0 z-50 flex flex-col w-56 bg-slate-900 dark:bg-slate-900 \
         transition-transform duration-200 translate-x-0 \
         md:relative md:translate-x-0"
    } else {
        "fixed inset-y-0 left-0 z-50 flex flex-col w-56 bg-slate-900 dark:bg-slate-900 \
         transition-transform duration-200 -translate-x-full \
         md:relative md:translate-x-0"
    };

    html! {
        <>
            // Backdrop — mobile only, tap to close
            if props.is_open {
                <div
                    class="fixed inset-0 z-40 bg-black/50 md:hidden"
                    onclick={Callback::from({
                        let on_close = props.on_close.clone();
                        move |_: MouseEvent| on_close.emit(())
                    })}
                />
            }
            <aside class={aside_class}>
                /* Logo / brand */
                <div class="flex items-center gap-2 px-5 h-14 border-b border-slate-800">
                    <div class="w-2 h-2 rounded-sm bg-amber-500" />
                    <span class="text-sm font-bold tracking-wide text-white">{"MOSAIC"}</span>
                </div>

                /* Nav links */
                <nav class="flex-1 py-4 space-y-0.5 px-2">
                    <NavSection label="Workspace" />
                    <SidebarLink route={Route::DashboardList} label="Dashboards" icon="▦" on_close={props.on_close.clone()} />
                    <SidebarLink route={Route::TemplateList}  label="Templates"  icon="⊞" on_close={props.on_close.clone()} />

                    <div class="pt-4">
                        <NavSection label="System" />
                        <SidebarLink route={Route::Settings} label="Settings" icon="⚙" on_close={props.on_close.clone()} />
                        if is_admin {
                            <SidebarLink route={Route::AdminUsers} label="Users" icon="◎" on_close={props.on_close.clone()} />
                        }
                    </div>
                </nav>

            </aside>
        </>
    }
}

/* ============================================================================================== */
/*                                          Section label                                         */
/* ============================================================================================== */

#[derive(Properties, PartialEq)]
struct NavSectionProps { label: &'static str }

#[function_component(NavSection)]
fn nav_section(props: &NavSectionProps) -> Html {
    html! {
        <p class="px-3 pb-1 text-xs font-semibold uppercase tracking-wider text-slate-600">
            { props.label }
        </p>
    }
}

/* ============================================================================================== */
/*                                          Sidebar link                                          */
/* ============================================================================================== */

#[derive(Properties, PartialEq)]
struct SidebarLinkProps {
    route: Route,
    label: &'static str,
    icon:  &'static str,
    on_close: Callback<()>,
}

#[function_component(SidebarLink)]
fn sidebar_link(props: &SidebarLinkProps) -> Html {
    let current = use_route::<Route>();
    let active = current.map(|r| r == props.route).unwrap_or(false);
    let navigator = use_navigator();

    let class = if active {
        // amber left-border accent, slightly lighter bg
        "flex w-full items-center gap-2.5 rounded-r px-3 py-2 text-sm font-medium \
         text-white bg-slate-800 border-l-2 border-amber-500 -ml-px pl-[11px] text-left"
    } else {
        "flex w-full items-center gap-2.5 rounded px-3 py-2 text-sm font-medium \
         text-slate-400 hover:text-white hover:bg-slate-800 transition-colors text-left"
    };

    let on_click = {
        let on_close = props.on_close.clone();
        let route = props.route.clone();
        let navigator = navigator.clone();

        Callback::from(move |_: MouseEvent| {
            if let Some(nav) = navigator.clone() {
                nav.push(&route);
            }
            on_close.emit(());
        })
    };

    html! {
        <button type="button" class={class} onclick={on_click}>
            <span class="text-base leading-none">{ props.icon }</span>
            { props.label }
        </button>
    }
}