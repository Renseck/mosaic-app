use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct NocodbPanelProps {
    pub source_url: String,
}

/* ============================================================================================== */
#[function_component(NocodbPanel)]
pub fn nocodb_panel(props: &NocodbPanelProps) -> Html {
    if props.source_url.is_empty() {
        return html! {
            <div class="flex items-center justify-center h-full text-xs text-stone-400 dark:text-stone-500">
                {"No source URL configured"}
            </div>
        };
    }
    html! {
        <iframe
            src={props.source_url.clone()}
            class="w-full h-full border-none"
            loading="lazy"
            sandbox="allow-scripts allow-same-origin allow-forms allow-popups allow-top-navigation"
        />
    }
}