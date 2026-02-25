use yew::prelude::*;
use yew_router::prelude::*;

use crate::api::dashboards;
use crate::hooks::use_api::use_api;
use crate::models::dashboard::{CreateDashboard, Dashboard};
use crate::router::Route;

#[function_component(DashboardListPage)]
pub fn dashboard_list_page() -> Html {
    let (state, reload) = use_api(|| dashboards::list_dashboards());

    let show_form  = use_state(|| false);
    let new_title  = use_state(String::new);
    let creating   = use_state(|| false);

    let on_new_title = {
        let new_title = new_title.clone();
        Callback::from(move |e: InputEvent| {
            let el: web_sys::HtmlInputElement = e.target_unchecked_into();
            new_title.set(el.value());
        })
    };

    let on_create = {
        let new_title = new_title.clone();
        let show_form = show_form.clone();
        let creating  = creating.clone();
        let reload    = reload.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let title     = (*new_title).clone();
            let show_form = show_form.clone();
            let new_title = new_title.clone();
            let creating  = creating.clone();
            let reload    = reload.clone();
            if title.is_empty() { return; }
            creating.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                let _ = dashboards::create_dashboard(&CreateDashboard {
                    title, icon: None, is_shared: Some(false),
                }).await;
                creating.set(false);
                show_form.set(false);
                new_title.set(String::new());
                reload.emit(());
            });
        })
    };

    html! {
        <div class="max-w-5xl mx-auto">
            /* ====== Page header ====== */
            <div class="flex items-center justify-between mb-6">
                <div>
                    <h1 class="text-lg font-bold text-stone-900 dark:text-stone-100">{"Dashboards"}</h1>
                    <p class="text-xs text-stone-400 mt-0.5">{"Your custom views and data layouts"}</p>
                </div>
                <button
                    onclick={Callback::from({
                        let show_form = show_form.clone();
                        move |_| show_form.set(!*show_form)
                    })}
                    class="flex items-center gap-1.5 px-4 py-2 text-sm font-semibold text-slate-900
                           bg-amber-500 rounded-lg hover:bg-amber-400 transition-colors"
                >
                    <span>{"+"}</span>{"New Dashboard"}
                </button>
            </div>

            /* ====== Inline create form ====== */
            if *show_form {
                <div class="mb-6 bg-white dark:bg-stone-800 border border-stone-200 dark:border-stone-700 
                            rounded-lg p-4 shadow-sm">
                    <form onsubmit={on_create} class="flex items-center gap-3">
                        <input
                            type="text"
                            autofocus=true
                            placeholder="Dashboard title"
                            value={(*new_title).clone()}
                            oninput={on_new_title}
                            class="flex-1 rounded-md bg-white dark:bg-stone-700 
                                   border border-stone-300 dark:border-stone-600 
                                   px-3 py-2 text-sm text-slate-900 dark:text-slate-100
                                   focus:outline-none focus:ring-2 focus:ring-amber-500 dark:focus:ring-amber-400 focus:border-transparent"
                        />
                        <button
                            type="submit"
                            disabled={*creating}
                            class="px-4 py-2 text-sm font-semibold bg-amber-500 dark:bg-amber-400 text-stone-900 dark:text-stone-900
                                   rounded-md hover:bg-amber-400 dark:hover:bg-amber-500 disabled:opacity-50 transition-colors"
                        >
                            { if *creating { "Creating…" } else { "Create" } }
                        </button>
                        <button
                            type="button"
                            onclick={Callback::from({
                                let show_form = show_form.clone();
                                move |_| show_form.set(false)
                            })}
                            class="px-3 py-2 text-sm text-stone-500 dark:text-stone-400 hover:text-stone-700 dark:hover:text-stone-200 transition-colors"
                        >
                            {"Cancel"}
                        </button>
                    </form>
                </div>
            }

            /* ====== Loading / error ====== */
            if state.loading && state.data.is_none() {
                <div class="flex items-center justify-center py-20 text-stone-400 dark:text-stone-500 text-sm">
                    {"Loading…"}
                </div>
            } else if let Some(err) = &state.error {
                <div class="rounded-lg bg-red-50 dark:bg-red-900 border border-red-200 dark:border-red-700 px-4 py-3 text-sm text-red-700 dark:text-red-700">
                    { format!("Failed to load dashboards: {err}") }
                </div>
            } else if let Some(dashboards) = &state.data {
                if dashboards.is_empty() {
                    <div class="flex flex-col items-center justify-center py-20 text-center">
                        <div class="text-4xl mb-3">{"▦"}</div>
                        <p class="text-stone-500 dark:text-stone-400 text-sm">{"No dashboards yet."}</p>
                        <p class="text-stone-400 dark:text-stone-500 text-xs mt-1">{"Create one to get started."}</p>
                    </div>
                } else {
                    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                        { for dashboards.iter().map(|d| html! {
                            <DashboardCard dashboard={d.clone()} />
                        })}
                    </div>
                }
            }
        </div>
    }
}

/* ============================================================================================== */
/*                                         Dashboard card                                         */
/* ============================================================================================== */

#[derive(Properties, PartialEq)]
struct DashboardCardProps {
    dashboard: Dashboard,
}

#[function_component(DashboardCard)]
fn dashboard_card(props: &DashboardCardProps) -> Html {
    let d = &props.dashboard;
    html! {
        <Link<Route>
            to={Route::DashboardView { slug: d.slug.clone() }}
            classes="block group bg-white dark:bg-stone-800 border border-stone-200 dark:border-stone-700 rounded-lg p-5 shadow-sm
                     hover:border-amber-400 dark:hover:border-amber-500 hover:shadow-md transition-all"
        >
            <div class="flex items-start gap-3">
                <div class="w-9 h-9 rounded-lg bg-amber-100 dark:bg-amber-900/30 flex items-center justify-center
                            text-amber-700 dark:text-amber-200 text-base shrink-0">
                    { d.icon.as_deref().unwrap_or("▦") }
                </div>
                <div class="min-w-0">
                    <p class="text-sm font-semibold text-stone-900 dark:text-stone-100 group-hover:text-amber-700 dark:group-hover:text-amber-200
                               transition-colors truncate">
                        { &d.title }
                    </p>
                    <p class="text-xs text-stone-400 dark:text-stone-500 mt-0.5 truncate">
                        { format!("/{}", d.slug) }
                    </p>
                </div>
            </div>
            if d.is_shared {
                <span class="mt-3 inline-block text-xs text-amber-600 dark:text-amber-300 font-medium">{"Shared"}</span>
            }
        </Link<Route>>
    }
}
