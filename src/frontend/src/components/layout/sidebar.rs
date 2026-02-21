use yew::prelude::*;
use yew_router::prelude::*;
use crate::router::Route;

#[function_component(Sidebar)]
pub fn sidebar() -> Html {
    html! {
        <aside class="flex flex-col w-56 min-h-screen bg-slate-900 shrink-0">
            /* Logo / brand */
            <div class="flex items-center gap-2 px-5 h-14 border-b border-slate-800">
                <div class="w-2 h-2 rounded-sm bg-amber-500" />
                <span class="text-sm font-bold tracking-wide text-white">{"MOSAIC"}</span>
            </div>

            /* Nav links */
            <nav class="flex-1 py-4 space-y-0.5 px-2">
                <NavSection label="Workspace" />
                <SidebarLink route={Route::DashboardList} label="Dashboards" icon="▦" />
                <SidebarLink route={Route::TemplateList}  label="Templates"  icon="⊞" />

                <div class="pt-4">
                    <NavSection label="System" />
                    <SidebarLink route={Route::Settings} label="Settings" icon="⚙"/>
                </div>
            </nav>

        </aside>
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
}

#[function_component(SidebarLink)]
fn sidebar_link(props: &SidebarLinkProps) -> Html {
    let current= use_route::<Route>();
    let active = current.map(|r| r == props.route).unwrap_or(false);

    let class = if active {
        // amber left-border accent, slightly lighter bg
        "flex items-center gap-2.5 rounded-r px-3 py-2 text-sm font-medium \
         text-white bg-slate-800 border-l-2 border-amber-500 -ml-px pl-[11px]"
    } else {
        "flex items-center gap-2.5 rounded px-3 py-2 text-sm font-medium \
         text-slate-400 hover:text-white hover:bg-slate-800 transition-colors"
    };

    html! {
        <Link<Route> to={props.route.clone()} classes={class}>
            <span class="text-base leading-none">{ props.icon }</span>
            { props.label }
        </Link<Route>>
    }
}