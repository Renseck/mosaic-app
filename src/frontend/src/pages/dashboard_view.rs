use yew::prelude::*;
use yew_router::prelude::*;

use crate::api::dashboards;
use crate::components::grid::DashboardGrid;
use crate::components::panels::PanelPicker;
use crate::hooks::use_api::use_api;
use crate::models::dashboard::{BatchPositionUpdate, CreatePanel, Panel};
use crate::router::Route;

#[derive(Properties, PartialEq)]
pub struct DashboardViewProps {
    pub slug: String,
}

#[function_component(DashboardViewPage)]
pub fn dashboard_view_page(props: &DashboardViewProps) -> Html {
    let slug = props.slug.clone();
    let (state, reload) = use_api(move || {
        let slug = slug.clone();
        async move { dashboards::get_dashboard(&slug).await }
    });

    let edit_mode   = use_state(|| false);
    let show_picker = use_state(|| false);

    /* ====== Position batch update after drag/resize ====== */
    let on_positions_change = {
        let reload = reload.clone();
        Callback::from(move |updates: Vec<BatchPositionUpdate>| {
            let reload = reload.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let _ = dashboards::batch_update_positions(&updates).await;
                reload.emit(()); // refresh to confirm persisted state
            });
        })
    };

    /* ====== Delete panel ====== */
    let on_delete_panel = {
        let reload = reload.clone();
        Callback::from(move |panel_id: String| {
            let reload = reload.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let _ = dashboards::delete_panel(&panel_id).await;
                reload.emit(());
            });
        })
    };

    /* ====== Derive panels from state ====== */
    let panels: Vec<Panel> = state.data.as_ref()
        .map(|d| d.panels.clone())
        .unwrap_or_default();

    /* ====== Add panel via picker ====== */
    let on_add_panel = {
        let reload      = reload.clone();
        let show_picker = show_picker.clone();
        let dashboard_id = state.data.as_ref().map(|d| d.dashboard.id.clone()).unwrap_or_default();
        let panels_snap  = panels.clone(); // snapshot for position calculation

        Callback::from(move |mut input: CreatePanel| {
            let reload       = reload.clone();
            let show_picker  = show_picker.clone();
            let dashboard_id = dashboard_id.clone();

            // Place below all existing panels to avoid overlap.
            input.grid_y = panels_snap.iter()
                .map(|p| p.grid_y + p.grid_h)
                .max()
                .unwrap_or(0);
            input.grid_x = 0;

            wasm_bindgen_futures::spawn_local(async move {
            let _ = dashboards::create_panel(&dashboard_id, &input).await;
            show_picker.set(false);
            reload.emit(());
            });
        })
    };

    let dashboard_title = state.data.as_ref()
        .map(|d| d.dashboard.title.clone())
        .unwrap_or_default();

    // ── Topbar actions (passed up through Shell) ──────────────────────────
    // We render the edit toggle + add panel button inline above the grid
    // so we don't need Shell prop drilling for Phase 6.

    if state.loading && state.data.is_none() {
        return html! {
            <div class="flex items-center justify-center py-20 text-stone-400 text-sm">
                {"Loading…"}
            </div>
        };
    }

    if state.data.is_none() {
        return html! {
            <div class="flex flex-col items-center justify-center py-20 text-center">
                <p class="text-stone-500 text-sm">{"Dashboard not found."}</p>
                <Link<Route> to={Route::DashboardList}
                    classes="mt-3 text-sm text-amber-600 hover:text-amber-700 font-medium">
                    {"← Back to dashboards"}
                </Link<Route>>
            </div>
        };
    }

    html! {
        <div class="flex flex-col gap-4">

            /* ====== Page header ====== */
            <div class="flex items-center justify-between">
                <div class="flex items-center gap-3">
                    <Link<Route> to={Route::DashboardList}
                        classes="text-stone-400 hover:text-stone-600 transition-colors text-sm">
                        {"←"}
                    </Link<Route>>
                    <h1 class="text-lg font-bold text-stone-900 dark:text-stone-100">{ &dashboard_title }</h1>
                    if state.loading {
                        <span class="text-xs text-stone-400">{"(refreshing…)"}</span>
                    }
                </div>

                <div class="flex items-center gap-2">
                    if *edit_mode {
                        <button
                            onclick={Callback::from({
                                let show_picker = show_picker.clone();
                                move |_| show_picker.set(true)
                            })}
                            class="flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold
                                   text-slate-900 bg-amber-500 rounded-md hover:bg-amber-400 transition-colors"
                        >
                            <span>{"+"}</span>{"Add Panel"}
                        </button>
                    }
                    <button
                        onclick={Callback::from({
                            let edit_mode = edit_mode.clone();
                            move |_| edit_mode.set(!*edit_mode)
                        })}
                        class={if *edit_mode {
                            "px-3 py-1.5 text-xs font-semibold rounded-md border-2 border-amber-500 text-amber-700 bg-amber-50 transition-colors"
                        } else {
                            "px-3 py-1.5 text-xs font-semibold rounded-md border border-stone-300 text-stone-600 hover:border-stone-400 bg-white transition-colors"
                        }}
                    >
                        { if *edit_mode { "Done" } else { "Edit Layout" } }
                    </button>
                </div>
            </div>

            /* ====== Grid ====== */
            if panels.is_empty() {
                <div class="flex flex-col items-center justify-center py-20 text-center
                             border-2 border-dashed border-stone-200 rounded-xl">
                    <div class="text-4xl mb-3 text-stone-300">{"⊞"}</div>
                    <p class="text-stone-500 text-sm">{"No panels yet."}</p>
                    if !*edit_mode {
                        <p class="text-stone-400 text-xs mt-1">
                            {"Click "}<strong>{"Edit Layout"}</strong>{" to add panels."}
                        </p>
                    }
                </div>
            } else {
                <DashboardGrid
                    panels={panels}
                    edit_mode={*edit_mode}
                    on_positions_change={on_positions_change}
                    on_delete_panel={on_delete_panel}
                />
            }

            /* ====== Panel picker modal ====== */
            if *show_picker {
                <PanelPicker
                    on_add={on_add_panel}
                    on_cancel={Callback::from({
                        let show_picker = show_picker.clone();
                        move |_| show_picker.set(false)
                    })}
                />
            }
        </div>
    }
}
