use yew::prelude::*;
use yew_router::prelude::*;

use crate::api::templates;
use crate::context::auth_context::AuthContext;
use crate::hooks::use_api::use_api;
use crate::models::template::DatasetTemplate;
use crate::models::user::Role;
use crate::router::Route;

/* ============================================================================================== */
/*                                            List page                                           */
/* ============================================================================================== */

#[function_component(TemplateList)]
pub fn template_list() -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext missing");
    let is_admin = auth.user.as_ref().map(|u| u.role == Role::Admin).unwrap_or(false);
    let (state, reload) = use_api(|| templates::list_templates());

    html! {
        <div class="max-w-5xl mx-auto">

            /* =================================== Page header ================================== */
            <div class="flex items-center justify-between mb-6">
                <div>
                    <h1 class="text-lg font-bold text-stone-900">{"Dataset Templates"}</h1>
                    <p class="text-xs text-stone-400 mt-0.5">
                        {"Provision NocoDB tables, forms, and Grafana dashboards from a single definition"}
                    </p>
                </div>
                <Link<Route>
                    to={Route::TemplateNew}
                    classes="flex items-center gap-1.5 px-4 py-2 text-sm font-semibold text-slate-900
                             bg-amber-500 rounded-lg hover:bg-amber-400 transition-colors"
                >
                    {"+ New Template"}
                </Link<Route>>
            </div>

            /* ============================= Loading / error / data ============================= */
            if state.loading && state.data.is_none() {
                <div class="flex items-center justify-center py-20 text-stone-400 text-sm">
                    {"Loading…"}
                </div>
            } else if let Some(ref err) = state.error {
                <div class="rounded-lg bg-red-50 border border-red-200 px-4 py-3 text-sm text-red-700">
                    { format!("Failed to load templates: {err}") }
                </div>
            } else if let Some(ref templates_list) = state.data {
                if templates_list.is_empty() {
                    <div class="flex flex-col items-center justify-center py-20 text-center">
                        <div class="text-4xl mb-3">{"⚡"}</div>
                        <p class="text-stone-500 text-sm">{"No dataset templates yet."}</p>
                        <p class="text-stone-400 text-xs mt-1">
                            {"Create one to automatically provision NocoDB tables and Grafana dashboards."}
                        </p>
                    </div>
                } else {
                    <div class="space-y-3">
                        { for templates_list.iter().map(|t| {
                            let reload   = reload.clone();
                            let t_id     = t.id.clone();
                            let on_delete = Callback::from(move |()| {
                                let reload = reload.clone();
                                let id     = t_id.clone();
                                wasm_bindgen_futures::spawn_local(async move {
                                    let _ = templates::delete_template(&id).await;
                                    reload.emit(());
                                });
                            });
                            html! {
                                <TemplateCard
                                    template={t.clone()}
                                    is_admin={is_admin}
                                    on_delete={on_delete}
                                />
                            }
                        })}
                    </div>
                }
            }
        </div>
    }
}

/* ============================================================================================== */
/*                                          Template card                                         */
/* ============================================================================================== */

#[derive(Properties, PartialEq)]
struct TemplateCardProps {
    template:  DatasetTemplate,
    is_admin:  bool,
    on_delete: Callback<()>,
}

#[function_component(TemplateCard)]
fn template_card(props: &TemplateCardProps) -> Html {
    let t             = &props.template;
    let field_count   = t.fields.as_array().map(|a| a.len()).unwrap_or(0);
    let is_provisioned = t.nocodb_table_id.is_some() && t.grafana_dashboard_uid.is_some();
    let confirm_delete = use_state(|| false);

    html! {
        <div class="bg-white border border-stone-200 rounded-xl p-5 shadow-sm hover:border-stone-300 transition-colors">
            <div class="flex items-start gap-4">

                /* ==================================== Icon ==================================== */
                <div class="w-10 h-10 rounded-lg bg-amber-100 flex items-center justify-center
                            text-amber-700 shrink-0 text-base">
                    {"⚡"}
                </div>

                /* ==================================== Info ==================================== */
                <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2 flex-wrap">
                        <p class="text-sm font-semibold text-stone-900">{ &t.name }</p>
                        if is_provisioned {
                            <span class="text-xs font-medium px-2 py-0.5 rounded-full bg-green-100 text-green-700">
                                {"provisioned"}
                            </span>
                        } else {
                            <span class="text-xs font-medium px-2 py-0.5 rounded-full bg-yellow-100 text-yellow-700">
                                {"partial"}
                            </span>
                        }
                    </div>

                    if let Some(ref desc) = t.description {
                        <p class="text-xs text-stone-400 mt-0.5 truncate">{ desc }</p>
                    }

                    <p class="text-xs text-stone-400 mt-1">
                        { format!("{} field{}", field_count, if field_count == 1 { "" } else { "s" }) }
                    </p>

                    /* ============================= External links ============================= */
                    <div class="flex flex-wrap gap-4 mt-2">
                        if let Some(ref form_id) = t.nocodb_form_id {
                            <a  href={format!("/proxy/nocodb/nc/form/{form_id}")}
                                target="_blank" rel="noopener noreferrer"
                                class="text-xs text-blue-500 hover:underline"
                            >
                                {"📋 Entry Form"}
                            </a>
                        }
                        if let Some(ref uid) = t.grafana_dashboard_uid {
                            <a  href={format!("/proxy/grafana/d/{uid}")}
                                target="_blank" rel="noopener noreferrer"
                                class="text-xs text-blue-500 hover:underline"
                            >
                                {"📊 Grafana Dashboard"}
                            </a>
                        }
                    </div>
                </div>

                /* ================================ Admin actions =============================== */
                if props.is_admin {
                    <div class="shrink-0 flex items-center">
                        if *confirm_delete {
                            <div class="flex items-center gap-2 text-xs">
                                <span class="text-stone-500">{"Delete?"}</span>
                                <button
                                    onclick={Callback::from({
                                        let on_delete = props.on_delete.clone();
                                        move |_: MouseEvent| on_delete.emit(())
                                    })}
                                    class="text-red-600 font-semibold hover:underline"
                                >
                                    {"Yes"}
                                </button>
                                <button
                                    onclick={Callback::from({
                                        let confirm = confirm_delete.clone();
                                        move |_: MouseEvent| confirm.set(false)
                                    })}
                                    class="text-stone-400 hover:text-stone-600"
                                >
                                    {"Cancel"}
                                </button>
                            </div>
                        } else {
                            <button
                                onclick={Callback::from({
                                    let confirm = confirm_delete.clone();
                                    move |_: MouseEvent| confirm.set(true)
                                })}
                                title="Delete template"
                                class="text-stone-300 hover:text-red-500 transition-colors text-sm"
                            >
                                {"✕"}
                            </button>
                        }
                    </div>
                }
            </div>
        </div>
    }
}
