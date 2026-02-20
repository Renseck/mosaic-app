use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::auth::LoginPage;
use crate::components::layout::Shell;
use crate::context::auth_context::{AuthProvider, AuthContext};
use crate::router::Route;

/* ============================================================================================== */
/*                                   Placeholder page components                                  */
/* ============================================================================================== */

#[function_component(DashboardListPage)]
fn dashboard_list_page() -> Html {
    html! {
        <div>
            <h2 class="text-xl font-semibold text-gray-800">{"Dashboards"}</h2>
            <p class="mt-2 text-sm text-gray-500">{"No dashboards yet."}</p>
        </div>
    }
}

#[function_component(DashboardViewPage)]
fn dashboard_view_page() -> Html {
    // TODO: In Phase 6 this will receive the slug prop from the router.
    html! {
        <div>{"Dashboard view - coming in Phase 6"}</div>
    }
}

/* ============================================================================================== */
#[function_component(TemplateListPage)]
fn template_list_page() -> Html {
    html! {
        <div>{"Templates - coming in Phase 7"}</div>
    }
}

/* ============================================================================================== */
#[function_component(SettingsPage)]
fn settings_page() -> Html {
    html! {
        <div>{"Settings - coming in Phase 8"}</div>
    }
}

/* ============================================================================================== */
#[function_component(NotFoundPage)]
fn not_found_page() -> Html {
    html! {
        <div class="text-center mt-16">
            <p class="text-4xl font-bold text-gray-300">{"404"}</p>
            <p class="mt-2 text-gray-500">{"Page not found"}</p>
        </div>
    }
}

/* ============================================================================================== */
/*                                          Route switch                                          */
/* ============================================================================================== */

#[function_component(AppContent)]
fn app_content() -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext missing");

    // While the initial session is in flight, show a loading screen.
    if auth.loading {
        return html! {
            <div class="min-h-screen flex items-center justify-center">
                <p class="text-gray-400">{"Loading..."}</p>
            </div>
        };
    }

    // Unauthenticated: only the login page is accessible.
    if auth.user.is_none() {
        return html! {
            <Switch<Route> render={|route| match route {
                _ => html! { <LoginPage /> },
            }} />
        };
    }

    // Authenticated: full shell + route switch.
    html! {
        <Shell>
            <Switch<Route> render={|route| match route {
                Route::Login => html! { <Redirect<Route> to={Route::DashboardList} /> },
                Route::Home => html! { <Redirect<Route> to={Route::DashboardList} /> },
                Route::DashboardList => html! { <DashboardListPage /> },
                Route::DashboardView { .. } => html! { <DashboardViewPage /> },
                Route::TemplateList => html! { <TemplateListPage /> },
                Route::TemplateNew => html! { <TemplateListPage /> },
                Route::Settings => html! { <SettingsPage /> },
                Route::NotFound => html! { <NotFoundPage /> },
            }} />
        </Shell>
    }
}

/* ============================================================================================== */
/*                                         Root component                                         */
/* ============================================================================================== */

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <AuthProvider>
                <AppContent />
            </AuthProvider>
        </BrowserRouter>
    }
}