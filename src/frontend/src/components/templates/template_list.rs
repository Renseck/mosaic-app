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

    let form_modal: UseStateHandle<Option<String>> = use_state(|| None);

    html! {
        <div class="max-w-5xl mx-auto">

            /* =================================== Page header ================================== */
            <div class="flex items-center justify-between mb-6">
                <div>
                    <h1 class="text-lg font-bold text-stone-900 dark:text-stone-100">
                        {"Dataset Templates"}
                    </h1>
                    <p class="text-xs text-stone-400 dark:text-stone-500 mt-0.5">
                        {"Provision NocoDB tables, forms, and Grafana dashboards from a single definition"}
                    </p>
                </div>
                <Link<Route>
                    to={Route::TemplateNew}
                    classes="flex items-center gap-1.5 px-4 py-2 
                             text-sm font-semibold text-slate-900 dark:text-slate-100
                             bg-amber-500 dark:bg-amber-400 rounded-lg 
                             hover:bg-amber-400 dark:hover:bg-amber-500 transition-colors"
                >
                    {"+ New Template"}
                </Link<Route>>
            </div>

            /* ============================= Loading / error / data ============================= */
            if state.loading && state.data.is_none() {
                <div class="flex items-center justify-center py-20 text-stone-400 dark:text-stone-500 text-sm">
                    {"Loading…"}
                </div>
            } else if let Some(ref err) = state.error {
                <div class="rounded-lg bg-red-50 dark:bg-red-900/20 border border-red-200
                            dark:border-red-800 px-4 py-3 text-sm text-red-700 dark:text-red-400">
                    { format!("Failed to load templates: {err}") }
                </div>
            } else if let Some(ref templates_list) = state.data {
                if templates_list.is_empty() {
                    <div class="flex flex-col items-center justify-center py-20 text-center">
                        <div class="text-4xl mb-3">{"⚡"}</div>
                        <p class="text-stone-500 dark:text-stone-400 text-sm">
                            {"No dataset templates yet."}
                        </p>
                        <p class="text-stone-400 dark:text-stone-500 text-xs mt-1">
                            {"Create one to automatically provision NocoDB tables and Grafana dashboards."}
                        </p>
                    </div>
                } else {
                    <div class="space-y-3">
                        { for templates_list.iter().map(|t| {
                            let reload   = reload.clone();
                            let t_id     = t.id.clone();
                            let form_modal = form_modal.clone();
                            let on_delete = Callback::from(move |()| {
                                let reload = reload.clone();
                                let id     = t_id.clone();
                                wasm_bindgen_futures::spawn_local(async move {
                                    let _ = templates::delete_template(&id).await;
                                    reload.emit(());
                                });
                            });
                            let on_open_form = {
                                let form_modal = form_modal.clone();
                                Callback::from(move |uuid: String| {
                                    form_modal.set(Some(uuid));
                                })
                            };
                            html! {
                                <TemplateCard
                                    template={t.clone()}
                                    is_admin={is_admin}
                                    on_delete={on_delete}
                                    on_open_form={on_open_form}
                                />
                            }
                        })}
                    </div>
                }
            }

            /* ================================== Form modal ==================================== */
            if let Some(ref uuid) = *form_modal {
                <FormModal
                    share_uuid={uuid.clone()}
                    on_close={Callback::from({
                        let form_modal = form_modal.clone();
                        move |_: ()| form_modal.set(None)
                    })}
                />
            }
        </div>
    }
}

/* ============================================================================================== */
/*                                          Template card                                         */
/* ============================================================================================== */

#[derive(Properties, PartialEq)]
struct TemplateCardProps {
    template:       DatasetTemplate,
    is_admin:       bool,
    on_delete:      Callback<()>,
    on_open_form:   Callback<String>,
}

#[function_component(TemplateCard)]
fn template_card(props: &TemplateCardProps) -> Html {
    let t             = &props.template;
    let field_count   = t.fields.as_array().map(|a| a.len()).unwrap_or(0);
    let is_provisioned = t.nocodb_table_id.is_some() && t.grafana_dashboard_uid.is_some();
    let confirm_delete = use_state(|| false);

    html! {
        <div class="bg-white dark:bg-stone-800 border border-stone-200 dark:border-stone-700
                    rounded-xl p-5 shadow-sm hover:border-stone-300 dark:hover:border-stone-600
                    transition-colors">
            <div class="flex items-start gap-4">

                /* ==================================== Icon ==================================== */
                <div class="w-10 h-10 rounded-lg bg-amber-100 dark:bg-amber-900/30
                            flex items-center justify-center text-amber-700 dark:text-amber-400
                            shrink-0 text-base">
                    {"⚡"}
                </div>

                /* ==================================== Info ==================================== */
                <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2 flex-wrap">
                        <p class="text-sm font-semibold text-stone-900 dark:text-stone-100">
                            { &t.name }
                        </p>
                        if is_provisioned {
                            <span class="text-xs font-medium px-2 py-0.5 rounded-full
                                         bg-green-100 dark:bg-green-900/30
                                         text-green-700 dark:text-green-400">
                                {"provisioned"}
                            </span>
                        } else {
                            <span class="text-xs font-medium px-2 py-0.5 rounded-full
                                         bg-yellow-100 dark:bg-yellow-900/30
                                         text-yellow-700 dark:text-yellow-400">
                                {"partial"}
                            </span>
                        }
                    </div>

                    if let Some(ref desc) = t.description {
                        <p class="text-xs text-stone-400 dark:text-stone-500 mt-0.5 truncate">
                            { desc }
                        </p>
                    }

                    <p class="text-xs text-stone-400 dark:text-stone-500 mt-1">
                        { format!("{} field{}", field_count, if field_count == 1 { "" } else { "s" }) }
                    </p>

                    /* ============================= External links ============================= */
                    <div class="flex flex-wrap gap-4 mt-2">
                        if let Some(ref form_uuid) = t.nocodb_form_id {
                            <button
                                onclick={Callback::from({
                                    let on_open = props.on_open_form.clone();
                                    let uuid = form_uuid.clone();
                                    move |_: MouseEvent| on_open.emit(uuid.clone())
                                })}
                                class="text-xs text-blue-500 dark:text-blue-400
                                       hover:underline cursor-pointer"
                            >
                                {"📋 Entry Form"}
                            </button>
                        }
                        if let Some(ref uid) = t.grafana_dashboard_uid {
                            <a  href={format!("/proxy/grafana/d/{uid}")}
                                target="_blank" rel="noopener noreferrer"
                                class="text-xs text-blue-500 dark:text-blue-400 hover:underline"
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
                                <span class="text-stone-500 dark:text-stone-400">{"Delete?"}</span>
                                <button
                                    onclick={Callback::from({
                                        let on_delete = props.on_delete.clone();
                                        move |_: MouseEvent| on_delete.emit(())
                                    })}
                                    class="text-red-600 dark:text-red-400 font-semibold hover:underline"
                                >
                                    {"Yes"}
                                </button>
                                <button
                                    onclick={Callback::from({
                                        let confirm = confirm_delete.clone();
                                        move |_: MouseEvent| confirm.set(false)
                                    })}
                                    class="text-stone-400 dark:text-stone-500
                                           hover:text-stone-600 dark:hover:text-stone-300"
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
                                class="text-stone-300 dark:text-stone-600
                                       hover:text-red-500 dark:hover:text-red-400
                                       transition-colors text-sm"
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

/* ============================================================================================== */
/*                                           Form modal                                           */
/* ============================================================================================== */

#[derive(Properties, PartialEq)]
struct FormModalProps {
    share_uuid: String,
    on_close:   Callback<()>,
}

#[function_component(FormModal)]
fn form_modal(props: &FormModalProps) -> Html {
    let src = format!("/proxy/nocodb/dashboard/#/nc/form/{}", props.share_uuid);

    html! {
        <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
             onclick={Callback::from({
                 let on_close = props.on_close.clone();
                 move |_: MouseEvent| on_close.emit(())
             })}
        >
            <div class="relative bg-white dark:bg-stone-800 rounded-xl shadow-2xl
                        w-full max-w-2xl mx-4"
                 style="height: 80vh;"
                 onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}
            >
                <button
                    onclick={Callback::from({
                        let on_close = props.on_close.clone();
                        move |_: MouseEvent| on_close.emit(())
                    })}
                    class="absolute top-3 right-3 z-10 w-8 h-8 flex items-center justify-center
                           rounded-full bg-stone-100 dark:bg-stone-700
                           hover:bg-stone-200 dark:hover:bg-stone-600
                           text-stone-500 dark:text-stone-400
                           hover:text-stone-700 dark:hover:text-stone-200
                           transition-colors text-sm"
                >
                    {"✕"}
                </button>

                <iframe
                    {src}
                    class="w-full h-full rounded-xl border-0"
                    title="Data Entry Form"
                />
            </div>
        </div>
    }
}