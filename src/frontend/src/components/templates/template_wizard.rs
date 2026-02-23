use yew::prelude::*;
use yew_router::prelude::*;

use crate::api::templates;
use crate::models::template::{CreateTemplateRequest, DatasetTemplate, FieldDefinition};
use crate::router::Route;
use super::field_editor::FieldEditor;

/* ============================================================================================== */
/*                                            Step enum                                           */
/* ============================================================================================== */

#[derive(Clone, PartialEq, Copy)]
enum Step { Name, Fields, Preview }

/* ============================================================================================== */
/*                                        Wizard component                                        */
/* ============================================================================================== */

#[function_component(TemplateWizard)]
pub fn template_wizard() -> Html {
    let step = use_state(|| Step::Name);
    let name = use_state(String::new);
    let description = use_state(String::new);
    let fields: UseStateHandle<Vec<FieldDefinition>> = use_state(|| vec![
        FieldDefinition {
            name:           "measured_at".to_string(),
            field_type:     "date".to_string(),
            unit:           None,
        },
    ]);
    let submitting  = use_state(|| false);
    let error       = use_state(|| Option::<String>::None);
    let created     = use_state(|| Option::<DatasetTemplate>::None);

    /* ===================================== Success screen ===================================== */
    if let Some(template) = (*created).clone() {
        return html! {
            <div class="max-w-2xl mx-auto space-y-6">
                <div class="rounded-xl bg-green-50 border border-green-200 p-6 text-center space-y-3">
                    <div class="text-3xl">{"✓"}</div>
                    <h2 class="text-base font-bold text-green-800">
                        { format!("\"{}\" provisioned!", template.name) }
                    </h2>
                    <p class="text-sm text-green-700">
                        {"NocoDB table, entry form, and Grafana dashboard created. \
                          A portal dashboard page was added to your sidebar."}
                    </p>
                </div>

                <div class="bg-white border border-stone-200 rounded-xl p-5 space-y-2">
                    <p class="text-xs font-semibold text-stone-500 uppercase tracking-wide mb-3">
                        {"Quick Links"}
                    </p>
                    if let Some(ref form_id) = template.nocodb_form_id {
                         <a  href={format!("/proxy/nocodb/nc/form/{form_id}")}
                            target="_blank" rel="noopener noreferrer"
                            class="flex items-center gap-2 text-sm text-blue-600 hover:underline"
                        >
                            {"📋 Open NocoDB Entry Form"}
                        </a>
                    }
                    if let Some(ref uid) = template.grafana_dashboard_uid {
                        <a  href={format!("/proxy/grafana/d/{uid}")}
                            target="_blank" rel="noopener noreferrer"
                            class="flex items-center gap-2 text-sm text-blue-600 hover:underline"
                        >
                            {"📊 Open Grafana Dashboard"}
                        </a>
                    }
                </div>

                <Link<Route>
                    to={Route::TemplateList}
                    classes="inline-flex items-center px-4 py-2 text-sm font-semibold text-slate-900
                            bg-amber-500 rounded-lg hover:bg-amber-400 transition-colors"
                >
                    {"← Back to Templates"}
                </Link<Route>>
            </div>
        }
    }

    /* ====================================== Wizard shell ====================================== */
    let step_idx = match *step { Step::Name => 0, Step::Fields => 1, Step::Preview => 2 };

    html! {
        <div class="max-w-2xl mx-auto space-y-6">

            /* =================================== Back crumb =================================== */
            <Link<Route>
                to={Route::TemplateList}
                classes="inline-flex items-center text-sm text-stone-400 hover:text-stone-600 transition-colors"
            >
                {"← Templates"}
            </Link<Route>>

            /* =================================== Page header ================================== */
            <div>
                <h1 class="text-lg font-bold text-stone-900">{"New Dataset Template"}</h1>
                <p class="text-xs text-stone-400 mt-0.5">
                    {"Define fields — NocoDB table, form, and Grafana dashboard are provisioned automatically."}
                </p>
            </div>

            /* =============================== Step pill indicator ============================== */
            <div class="flex items-center gap-2 text-xs font-medium select-none">
                { for ["Name", "Fields", "Preview"].iter().enumerate().map(|(i, label)| {
                    let cls = if i == step_idx {
                        "px-3 py-1 rounded-full bg-amber-500 text-slate-900"
                    } else if i < step_idx {
                        "px-3 py-1 rounded-full bg-stone-200 text-stone-500"
                    } else {
                        "px-3 py-1 rounded-full bg-stone-100 text-stone-400"
                    };
                    html! {
                        <>
                            <span class={cls}>{ *label }</span>
                            if i < 2 { <span class="text-stone-300">{"›"}</span> }
                        </>
                    }
                })}
            </div>

            /* ================================== Step content ================================== */
            <div class="bg-white border border-stone-200 rounded-xl p-6 shadow-sm">
                { match *step {

                    /* ======================= Step 1: Name & Description ======================= */
                    Step::Name => html! {
                        <form class="space-y-4"
                              onsubmit={Callback::from({
                                  let step = step.clone();
                                  let name = name.clone();
                                  move |e: SubmitEvent| {
                                      e.prevent_default();
                                      if !(*name).trim().is_empty() {
                                          step.set(Step::Fields);
                                      }
                                  }
                              })}>

                            <div class="space-y-1">
                                <label class="block text-xs font-semibold text-stone-600">
                                    {"Template Name"}
                                    <span class="text-red-400 ml-0.5">{"*"}</span>
                                </label>
                                <input
                                    type="text"
                                    required=true
                                    autofocus=true
                                    placeholder="e.g. Body Metrics, Sleep Log"
                                    value={(*name).clone()}
                                    oninput={Callback::from({
                                        let name = name.clone();
                                        move |e: InputEvent| {
                                            let el: web_sys::HtmlInputElement = e.target_unchecked_into();
                                            name.set(el.value());
                                        }
                                    })}
                                    class="w-full rounded-lg border border-stone-300 px-3 py-2 text-sm
                                           focus:outline-none focus:ring-2 focus:ring-amber-500
                                           focus:border-transparent"
                                />
                                <p class="text-xs text-stone-400">
                                    {"Used as NocoDB table title and Grafana dashboard name."}
                                </p>
                            </div>

                            <div class="space-y-1">
                                <label class="block text-xs font-semibold text-stone-600">
                                    {"Description "}
                                    <span class="text-stone-400 font-normal">{"(optional)"}</span>
                                </label>
                                <textarea
                                    value={(*description).clone()}
                                    placeholder="What is this dataset tracking?"
                                    rows="3"
                                    oninput={Callback::from({
                                        let description = description.clone();
                                        move |e: InputEvent| {
                                            let el: web_sys::HtmlInputElement = e.target_unchecked_into();
                                            description.set(el.value());
                                        }
                                    })}
                                    class="w-full rounded-lg border border-stone-300 px-3 py-2 text-sm
                                           resize-none focus:outline-none focus:ring-2 focus:ring-amber-500
                                           focus:border-transparent"
                                />
                            </div>

                            <div class="flex justify-end pt-1">
                                <button
                                    type="submit"
                                    class="px-5 py-2 text-sm font-semibold text-slate-900 bg-amber-500
                                           rounded-lg hover:bg-amber-400 transition-colors"
                                >
                                    {"Next →"}
                                </button>
                            </div>
                        </form>
                    },

                    /* ========================== Step 2: Field editor ========================== */
                    Step::Fields => html! {
                        <div class="space-y-4">
                            <div>
                                <p class="text-xs font-semibold text-stone-600 mb-1">{"Define Fields"}</p>
                                <p class="text-xs text-stone-400">
                                    {"Numeric fields get a Grafana time-series chart. \
                                      All fields appear in the NocoDB entry form."}
                                </p>
                            </div>

                            <FieldEditor
                                fields={(*fields).clone()}
                                on_change={Callback::from({
                                    let fields = fields.clone();
                                    move |v| fields.set(v)
                                })}
                            />

                            // Hint about measured_at — only visible while users can still see/remove it
                            if fields.iter().any(|f| f.name == "measured_at") {
                                <div class="rounded-lg bg-blue-50 border border-blue-100 px-4 py-3
                                            text-xs text-blue-700 space-y-1">
                                    <p class="font-semibold text-blue-800">
                                        {"💡 About the measured_at field"}
                                    </p>
                                    <p>
                                        {"This field lets you record when a measurement was actually taken, \
                                          even if you enter it into the system later. Grafana will use it as \
                                          the time axis instead of the entry date."}
                                    </p>
                                    <p>
                                        {"If left blank on a row, Grafana will fall back to the entry date \
                                          automatically. You may remove this field if your data is always \
                                          entered on the same day it is measured."}
                                    </p>
                                </div>
                            }

                            if (*fields).is_empty() {
                                <p class="text-xs text-amber-600 font-medium">
                                    {"Add at least one field to continue."}
                                </p>
                            }

                            <div class="flex items-center justify-between pt-1">
                                <button
                                    type="button"
                                    onclick={Callback::from({
                                        let step = step.clone();
                                        move |_: MouseEvent| step.set(Step::Name)
                                    })}
                                    class="px-4 py-2 text-sm text-stone-500 hover:text-stone-700 transition-colors"
                                >
                                    {"← Back"}
                                </button>
                                <button
                                    type="button"
                                    disabled={(*fields).is_empty()}
                                    onclick={Callback::from({
                                        let step = step.clone();
                                        move |_: MouseEvent| step.set(Step::Preview)
                                    })}
                                    class="px-5 py-2 text-sm font-semibold text-slate-900 bg-amber-500 rounded-lg
                                           hover:bg-amber-400 disabled:opacity-40 disabled:cursor-not-allowed
                                           transition-colors"
                                >
                                    {"Next →"}
                                </button>
                            </div>
                        </div>
                    },

                    /* ======================== Step 3: Preview & submit ======================== */
                    Step::Preview => {
                        let fields_snap = (*fields).clone();
                        let name_snap   = (*name).clone();
                        let desc_snap   = (*description).clone();
                        let num_panels  = fields_snap.iter().filter(|f| f.field_type == "number").count();
                        let is_sub      = *submitting;
                        let err_msg     = (*error).clone();

                        html! {
                            <div class="space-y-4">
                                <div>
                                    <p class="text-xs font-semibold text-stone-600 mb-1">{"Review & Provision"}</p>
                                    <p class="text-xs text-stone-400">
                                        {"Confirm the configuration. External resources will be created — \
                                          this may take a few seconds."}
                                    </p>
                                </div>

                                /* ======================== Summary card ======================== */
                                <div class="rounded-lg bg-stone-50 border border-stone-200 p-4 space-y-3">
                                    <div>
                                        <p class="text-xs text-stone-400 font-semibold uppercase tracking-wide">
                                            {"Name"}
                                        </p>
                                        <p class="text-sm font-semibold text-stone-900 mt-0.5">{ &name_snap }</p>
                                        if !desc_snap.is_empty() {
                                            <p class="text-xs text-stone-500 mt-1">{ &desc_snap }</p>
                                        }
                                    </div>
                                    <div>
                                        <p class="text-xs text-stone-400 font-semibold uppercase tracking-wide mb-2">
                                            { format!("Fields ({})", fields_snap.len()) }
                                        </p>
                                        <div class="space-y-1.5">
                                            { for fields_snap.iter().map(|f| {
                                                let badge = match f.field_type.as_str() {
                                                    "number" => "bg-amber-100 text-amber-800",
                                                    "date"   => "bg-blue-100  text-blue-800",
                                                    "select" => "bg-purple-100 text-purple-800",
                                                    _        => "bg-stone-100 text-stone-700",
                                                };
                                                html! {
                                                    <div class="flex items-center gap-2">
                                                        <span class="font-mono text-xs text-stone-800">
                                                            { &f.name }
                                                        </span>
                                                        if let Some(ref unit) = f.unit {
                                                            <span class="text-xs text-stone-400">
                                                                { format!("({})", unit) }
                                                            </span>
                                                        }
                                                        <span class={classes!(
                                                            "text-xs", "font-medium",
                                                            "px-1.5", "py-0.5", "rounded",
                                                            badge
                                                        )}>
                                                            { &f.field_type }
                                                        </span>
                                                    </div>
                                                }
                                            })}
                                        </div>
                                    </div>
                                </div>

                                /* =============== "What will be created" info box ============== */
                                <div class="rounded-lg bg-blue-50 border border-blue-100 p-4 text-xs
                                            text-blue-700 space-y-1">
                                    <p class="font-semibold text-blue-800 mb-1">{"What will be provisioned:"}</p>
                                    <p>{ format!("✓  NocoDB table with {} columns", fields_snap.len()) }</p>
                                    <p>{"✓  Shared NocoDB entry form"}</p>
                                    <p>{ format!("✓  Grafana dashboard with {} time-series panel{}",
                                        num_panels, if num_panels == 1 { "" } else { "s" }) }
                                    </p>
                                    <p>{"✓  Portal dashboard page with embedded views"}</p>
                                </div>

                                if let Some(ref err) = err_msg {
                                    <div class="rounded-lg bg-red-50 border border-red-200 px-4 py-3 text-sm text-red-700">
                                        { format!("Provisioning failed: {err}") }
                                    </div>
                                }

                                <div class="flex items-center justify-between pt-1">
                                    <button
                                        type="button"
                                        disabled={is_sub}
                                        onclick={Callback::from({
                                            let step = step.clone();
                                            move |_: MouseEvent| step.set(Step::Fields)
                                        })}
                                        class="px-4 py-2 text-sm text-stone-500 hover:text-stone-700
                                               disabled:opacity-40 transition-colors"
                                    >
                                        {"← Back"}
                                    </button>
                                    <button
                                        type="button"
                                        disabled={is_sub}
                                        onclick={Callback::from({
                                            let name        = name.clone();
                                            let description = description.clone();
                                            let fields      = fields.clone();
                                            let submitting  = submitting.clone();
                                            let error       = error.clone();
                                            let created     = created.clone();
                                            move |_: MouseEvent| {
                                                let name_v    = (*name).trim().to_string();
                                                let desc_v    = (*description).trim().to_string();
                                                let fields_v  = (*fields).clone();
                                                let submitting = submitting.clone();
                                                let error     = error.clone();
                                                let created   = created.clone();
                                                submitting.set(true);
                                                error.set(None);
                                                wasm_bindgen_futures::spawn_local(async move {
                                                    let req = CreateTemplateRequest {
                                                        name:        name_v,
                                                        description: if desc_v.is_empty() { None } else { Some(desc_v) },
                                                        fields:      fields_v,
                                                    };
                                                    match templates::create_template(&req).await {
                                                        Ok(t)  => created.set(Some(t)),
                                                        Err(e) => error.set(Some(e.to_string())),
                                                    }
                                                    submitting.set(false);
                                                });
                                            }
                                        })}
                                        class="px-5 py-2 text-sm font-semibold text-slate-900 bg-amber-500
                                               rounded-lg hover:bg-amber-400 disabled:opacity-50
                                               disabled:cursor-not-allowed transition-colors"
                                    >
                                        { if is_sub { "Provisioning…" } else { "Provision Dataset" } }
                                    </button>
                                </div>
                            </div>
                        }
                    },
                }}
            </div>
        </div>
    }
}