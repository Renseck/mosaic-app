use yew::prelude::*;
use crate::models::dashboard::CreatePanel;

#[derive(Debug, Clone, PartialEq)]
enum PanelType {
    GrafanaPanel,
    GrafanaDashboard,
    NocodbForm,
    NocodbGrid,
    Markdown,
}

impl PanelType {
    fn api_key(&self) -> &'static str {
        match self {
            Self::GrafanaPanel     => "grafana_panel",
            Self::GrafanaDashboard => "grafana_dashboard",
            Self::NocodbForm       => "nocodb_form",
            Self::NocodbGrid       => "nocodb_grid",
            Self::Markdown         => "markdown",
        }
    }
    
    fn label(&self) -> &'static str {
        match self {
            Self::GrafanaPanel     => "Grafana Panel",
            Self::GrafanaDashboard => "Grafana Dashboard",
            Self::NocodbForm       => "NocoDB Form",
            Self::NocodbGrid       => "NocoDB Grid",
            Self::Markdown         => "Markdown",
        }
    }

    fn needs_url(&self) -> bool { !matches!(self, Self::Markdown) }
}

/* ============================================================================================== */
#[derive(Properties, PartialEq)]
pub struct PanelPickerProps {
    pub on_add:    Callback<CreatePanel>,
    pub on_cancel: Callback<()>,
}

#[function_component(PanelPicker)]
pub fn panel_picker(props: &PanelPickerProps) -> Html {
    let panel_type = use_state(|| PanelType::GrafanaPanel);
    let title = use_state(String::new);
    let source_url = use_state(String::new);
    let content = use_state(String::new); //markdown content

    let on_submit = {
        let on_add = props.on_add.clone();
        let panel_type = panel_type.clone();
        let title = title.clone();
        let source_url = source_url.clone();
        let content = content.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let t = (*title).clone();
            let pt = (*panel_type).clone();

            let (source_url, config) = if pt == PanelType::Markdown {
                (None, Some(serde_json::json!({ "content": *content })))
            } else {
                (Some((*source_url).clone()), None)
            };

            on_add.emit(CreatePanel {
                title: if t.is_empty() { None } else { Some(t) },
                panel_type: pt.api_key().to_string(),
                source_url,
                config,
                grid_x: 0,
                grid_y: 0,
                grid_w: 6,
                grid_h: 4,
            });
        })
    };

    let on_cancel = {
        let cb = props.on_cancel.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };

    // Helper to build type option button
    let panel_type_btn = |pt: PanelType| {
        let current = (*panel_type).clone();
        let is_active = current == pt;
        let pt2 = pt.clone();
        let setter = panel_type.clone();
        let onclick = Callback::from(move |_: MouseEvent| setter.set(pt2.clone()));
        let class = if is_active {
            "flex-1 py-2 px-3 text-xs font-semibold rounded-md 
            border-2 border-amber-500  dark:border-amber-400
            bg-amber-500 dark:bg-amber-400
            text-amber-800 dark:text-amber-900
            transition-colors"
        } else {
            "flex-1 py-2 px-3 text-xs font-semibold rounded-md 
            border border-stone-300 dark:border-stone-600 hover:border-stone-400 dark:hover:border-stone-500
            text-stone-600 dark:text-stone-300
            hover:bg-stone-50 dark:hover:bg-stone-900
            transition-colors"
        };
        html! {
            <button type="button" class={class} onclick={onclick}>{ pt.label() }</button>
        }
    };

    let needs_url = (*panel_type).needs_url();

    html! {
        // Backdrop
        <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm">
            // Modal card
            <div class="w-full max-w-md bg-white dark:bg-stone-800 rounded-xl shadow-2xl border border-stone-200 dark:border-stone-700 overflow-hidden">

                // Header
                <div class="flex items-center justify-between px-6 py-4 border-b border-stone-100 dark:border-stone-800">
                    <h2 class="text-sm font-bold text-stone-900 dark:text-stone-100">{"Add Panel"}</h2>
                    <button onclick={on_cancel.clone()}
                        class="text-stone-400 hover:text-stone-600 dark:hover:text-stone-300 transition-colors text-lg leading-none">
                        {"✕"}
                    </button>
                </div>

                <form onsubmit={on_submit} class="px-6 py-5 space-y-5">

                    // Panel type selection
                    <div class="space-y-2">
                        <label class="block text-xs font-semibold uppercase tracking-wider text-stone-500 dark:text-stone-400">
                            {"Panel Type"}
                        </label>
                        <div class="flex flex-wrap gap-2">
                            { panel_type_btn(PanelType::GrafanaPanel) }
                            { panel_type_btn(PanelType::GrafanaDashboard) }
                            { panel_type_btn(PanelType::NocodbForm) }
                            { panel_type_btn(PanelType::NocodbGrid) }
                            { panel_type_btn(PanelType::Markdown) }
                        </div>
                    </div>

                    // Title
                    <div class="space-y-1">
                        <label class="block text-xs font-semibold uppercase tracking-wider text-stone-500 dark:text-stone-400">
                            {"Title (optional)"}
                        </label>
                        <input
                            type="text"
                            placeholder="Leave blank to auto-name"
                            value={(*title).clone()}
                            oninput={Callback::from({
                                let title = title.clone();
                                move |e: InputEvent| {
                                    let el: web_sys::HtmlInputElement = e.target_unchecked_into();
                                    title.set(el.value());
                                }
                            })}
                            class="w-full rounded-md bg-white dark:bg-stone-800 border border-stone-300 dark:border-stone-600 px-3 py-2 text-sm
                                   focus:outline-none focus:ring-2 focus:ring-amber-500 dark:focus:ring-amber-400 focus:border-transparent"
                        />
                    </div>

                    // Source URL (Grafana / NocoDB)
                    if needs_url {
                        <div class="space-y-1">
                            <label class="block text-xs font-semibold uppercase tracking-wider text-stone-500 dark:text-stone-400">
                                {"Source URL"}
                            </label>
                            <input
                                type="text"
                                required=true
                                placeholder="/proxy/grafana/d-solo/..."
                                value={(*source_url).clone()}
                                oninput={Callback::from({
                                    let source_url = source_url.clone();
                                    move |e: InputEvent| {
                                        let el: web_sys::HtmlInputElement = e.target_unchecked_into();
                                        source_url.set(el.value());
                                    }
                                })}
                                class="w-full rounded-md bg-white dark:bg-stone-800 border border-stone-300 dark:border-stone-600 px-3 py-2 text-sm font-mono
                                       focus:outline-none focus:ring-2 focus:ring-amber-500 dark:focus:ring-amber-400 focus:border-transparent"
                            />
                            <p class="text-xs text-stone-400 dark:text-stone-500">
                                { "Use the proxy path so the request stays same-origin." }
                            </p>
                        </div>
                    }

                    // Markdown content
                    if !needs_url {
                        <div class="space-y-1">
                            <label class="block text-xs font-semibold uppercase tracking-wider text-stone-500 dark:text-stone-400">
                                {"Content"}
                            </label>
                            <textarea
                                rows="5"
                                placeholder="Enter markdown or plain text…"
                                value={(*content).clone()}
                                oninput={Callback::from({
                                    let content = content.clone();
                                    move |e: InputEvent| {
                                        let el: web_sys::HtmlTextAreaElement = e.target_unchecked_into();
                                        content.set(el.value());
                                    }
                                })}
                                class="w-full rounded-md bg-white dark:bg-stone-800 border border-stone-300 dark:border-stone-600 px-3 py-2 text-sm font-mono
                                       focus:outline-none focus:ring-2 focus:ring-amber-500 dark:focus:ring-amber-400 focus:border-transparent
                                       resize-y"
                            />
                        </div>
                    }

                    // Actions
                    <div class="flex justify-end gap-3 pt-1">
                        <button
                            type="button"
                            onclick={on_cancel}
                            class="px-4 py-2 text-sm font-medium text-stone-600 dark:text-stone-300 bg-white dark:bg-stone-700 border border-stone-300 dark:border-stone-600
                                   rounded-md hover:bg-stone-50 dark:hover:bg-stone-900 transition-colors"
                        >
                            {"Cancel"}
                        </button>
                        <button
                            type="submit"
                            class="px-4 py-2 text-sm font-semibold text-slate-900 dark:text-slate-100 bg-amber-500 dark:bg-amber-400
                                   rounded-md hover:bg-amber-400 dark:hover:bg-amber-300 transition-colors"
                        >
                            {"Add Panel"}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    }
}