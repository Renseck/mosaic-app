use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct MarkdownPanelProps {
    pub content: String,
}

/* ============================================================================================== */
#[function_component(MarkdownPanel)]
pub fn markdown_panel(props: &MarkdownPanelProps) -> Html {
    html! {
        <div class="h-full overflow-auto p-4">
            <pre class="text-sm text-stone-700 whitespace-pre-wrap font-sans leading-relaxed">
                { &props.content }
            </pre>
        </div>
    }
}