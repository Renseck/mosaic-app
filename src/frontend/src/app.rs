use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::auth::LoginPage;
use crate::components::common::toast::ToastProvider;
use crate::components::layout::Shell;
use crate::components::templates::{
    template_list::TemplateList,
    template_wizard::TemplateWizard,
};
use crate::context::auth_context::{AuthProvider, AuthContext};
use crate::context::theme_context::ThemeProvider;
use crate::pages::{
    admin_users::AdminUsersPage,
    dashboard_list::DashboardListPage,
    dashboard_view::DashboardViewPage,
    settings::SettingsPage,
};
use crate::router::Route;

/* ============================================================================================== */
/*                                   Placeholder page components                                  */
/* ============================================================================================== */

#[function_component(NotFoundPage)]
fn not_found_page() -> Html {
    html! {
        <div class="text-center mt-16">
            <p class="text-4xl font-bold text-gray-300 dark:text-stone-600">{"404"}</p>
            <p class="mt-2 text-gray-500 dark:text-stone-400">{"Page not found"}</p>
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
            <div class="min-h-screen flex items-center justify-center bg-stone-50 dark:bg-stone-900">
                <div class="flex flex-col items-center gap-3">
                    <div class="w-2 h-2 rounded-sm bg-amber-500 animate-bounce" />
                    <p class="text-stone-400 dark:text-stone-500 text-sm">{"Loading…"}</p>
                </div>
            </div>
        };
    }

    // Unauthenticated: only the login page is accessible.
    if auth.user.is_none() {
        return html! {
            <Switch<Route> render={|_| html! { <LoginPage /> }} />
        };
    }

    let is_admin = auth.user.as_ref().map(|u| u.role.is_admin()).unwrap_or(false);

    // Authenticated: full shell + route switch.
    html! {
        <Shell>
            <Switch<Route> render={move |route| match route {
                Route::Login | Route::Home =>
                    html! { <Redirect<Route> to={Route::DashboardList} /> },
                Route::DashboardList =>
                    html! { <DashboardListPage /> },
                Route::DashboardView { slug } =>
                    html! { <DashboardViewPage slug={slug} /> },
                Route::TemplateList =>
                    html! { <TemplateList /> },
                Route::TemplateNew =>
                    html! { <TemplateWizard /> },
                Route::Settings =>
                    html! { <SettingsPage /> },
                Route::AdminUsers =>
                    if is_admin {
                        html! { <AdminUsersPage /> }
                    } else {
                        html! { <Redirect<Route> to={Route::DashboardList} /> }
                    },
                Route::NotFound =>
                    html! { <NotFoundPage /> },
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
            <ThemeProvider>
                <AuthProvider>
                    <ToastProvider>
                        <AppContent />
                    </ToastProvider>
                </AuthProvider>
            </ThemeProvider>
        </BrowserRouter>
    }
}