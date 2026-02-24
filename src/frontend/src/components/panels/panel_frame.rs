use yew::prelude::*;
use crate::models::dashboard::Panel;
use super::{
    grafana_panel::GrafanaPanel,
    nocodb_panel::NocodbPanel,
    markdown_panel::MarkdownPanel,
};

#[derive(Properties, PartialEq)]
pub struct PanelFrameProps {
    pub panel: Panel,
    pub edit_mode: bool,
    pub on_delete: Callback<String>,
}

/* ============================================================================================== */
#[function_component(PanelFrame)]
pub fn panel_frame(props: &PanelFrameProps) -> Html {
    let panel = &props.panel;
    let on_delete = {
        let on_delete = props.on_delete.clone();
        let id = panel.id.clone();
        Callback::from(move |_: MouseEvent| on_delete.emit(id.clone()))
    };

    html! {
        <div class="flex flex-col h-full bg-white dark:bg-stone-800 border border-stone-200 dark:border-stone-700 rounded-lg overflow-hidden shadow-sm">

            /* ====== Header ====== */
            <div class={classes!(
                "flex", "items-center", "justify-between",
                "px-3", "py-2", "border-b", "border-stone-100", "dark:border-stone-800",
                "bg-stone-50", "dark:bg-stone-900", "shrink-0",
                // amber top stripe visible only in edit mode
                if props.edit_mode { "border-t-2 border-t-amber-500 dark:border-t-amber-400" } else { "" }
            )}>
                <div class="flex items-center gap-2 min-w-0">
                    if props.edit_mode {
                        // Drag handle
                        <span class="drag-handle cursor-grab text-stone-300 dark:text-stone-600 hover:text-stone-500 dark:hover:text-stone-300 select-none shrink-0">
                            {"⠿"}
                        </span>
                    }
                    <span class="text-xs font-semibold text-stone-700 dark:text-stone-200 truncate">
                        { panel.title.as_deref().unwrap_or("—") }
                    </span>
                </div>

                if props.edit_mode {
                    <button
                        onclick={on_delete}
                        title="Remove panel"
                        class="ml-2 text-stone-400 dark:text-stone-500 hover:text-red-500 dark:hover:text-red-400 transition-colors text-sm shrink-0"
                    >
                        {"✕"}
                    </button>
                }
            </div>

            /* ====== Content ====== */
            <div class="flex-1 overflow-hidden">
                { render_panel_content(panel) }
            </div>
        </div>
    }
}

/* ============================================================================================== */
fn render_panel_content(panel: &Panel) -> Html {
    match panel.panel_type.as_str() {
        "grafana_panel" | "grafana_dashboard" => html! {
            <GrafanaPanel source_url={panel.source_url.clone().unwrap_or_default()} />
        },
        "nocodb_form" | "nocodb_grid" | "nocodb_gallery" => html! {
            <NocodbPanel source_url={panel.source_url.clone().unwrap_or_default()} />
        },
        "markdown" => html! {
            <MarkdownPanel content={
                panel.config.get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string()
            } />
        },
        _ => html! {
            <div class="flex items-center justify-center h-full text-xs text-stone-400 dark:text-stone-500">
                { format!("Unknown panel type: {}", panel.panel_type) }
            </div>
        },
    }
}