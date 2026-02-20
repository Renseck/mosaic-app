use yew::prelude::*;
use yew_router::prelude::*;
use crate::router::Route;

#[function_component(Sidebar)]
pub fn sidebar() -> Html {
    html! {
        <aside class="flex flex-col w-56 min-h-screen bg-gray-900 text-gray-100 shrink-0">
            /* Logo / brand */
            <div class="px-5 py-4 border-b border-gray-700">
                <span class="text-lg font-bold tracking-tight">{"Mosaic"}</span>
            </div>

            /* Nav links */
            <nav class="flex-1 px-3 py-4 space-y-1">
                <SidebarLink route={Route::DashboardList} label="Dashboards" />
                <SidebarLink route={Route::TemplateList} label="Templates" />
                <SidebarLink route={Route::Settings} label="Settings" />
            </nav>

        </aside>
    }
}

/* ============================================================================================== */
/*                                          Sidebar link                                          */
/* ============================================================================================== */

#[derive(Properties, PartialEq)]
struct SidebarLinkProps {
    route: Route,
    label: &'static str,
}

#[function_component(SidebarLink)]
fn sidebar_link(props: &SidebarLinkProps) -> Html {
    let current= use_route::<Route>();
    let active = current.map(|r| r == props.route).unwrap_or(false);

    let class = if active {
        "flex items-center gap-2 rounded px-3 py-2 text-sm font-medium bg-indigo-600 text-white"
    } else {
        "flex items-center gap-2 rounded px-3 py-2 text-sm font-medium text-gray-300 hover:bg-gray-800 hover:text-white transition-colors"
    };

    html! {
        <Link<Route> to={props.route.clone()} classes={class}>
            { props.label }
        </Link<Route>>
    }
}