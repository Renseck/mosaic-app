use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct GrafanaPanelProps {
    pub source_url: String,
}

/* ============================================================================================== */
#[function_component(GrafanaPanel)]
pub fn grafana_panel(props: &GrafanaPanelProps) -> Html {
    if props.source_url.is_empty() {
        return html! {
            <div class="flex items-center justify-center h-full text-xs 
                        text-stone-400 dark:text-stone-500
                        bg-white dark:bg-stone-800"
            >
                {"No source URL configured"}
            </div>
        };
    }
    html! {
        <iframe
            src={props.source_url.clone()}
            class="w-full h-full border-none"
            loading="lazy"
            sandbox="allow-scripts allow-same-origin allow-forms allow-popups"
        />
    }
}