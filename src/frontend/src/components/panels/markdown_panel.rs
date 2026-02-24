use pulldown_cmark::{Parser, Options, html::push_html};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct MarkdownPanelProps {
    pub content: String,
}

/* ============================================================================================== */
/// Renders a Markdown string as styled HTML.
///
/// Uses `pulldown_cmark` to parse Markdown (with tables, strikethrough, and
/// task lists enabled) and injects the resulting HTML via Yew's
/// `from_html_unchecked`. Tailwind's `prose` classes handle typography.
#[function_component(MarkdownPanel)]
pub fn markdown_panel(props: &MarkdownPanelProps) -> Html {
    let rendered = use_memo(props.content.clone(), |content| {
        let mut opts = Options::empty();
        opts.insert(Options::ENABLE_TABLES);
        opts.insert(Options::ENABLE_STRIKETHROUGH);
        opts.insert(Options::ENABLE_TASKLISTS);

        let parser = Parser::new_ext(content, opts);
        let mut html_output = String::new();
        push_html(&mut html_output, parser);
        html_output
    });

    let html_content = Html::from_html_unchecked(AttrValue::from((*rendered).clone()));

    html! {
        <div class="h-full overflow-auto p-4 prose prose-sm prose-stone dark:prose-invert max-w-none">
            { html_content }
        </div>
    }
}