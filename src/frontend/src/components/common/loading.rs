use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct LoadingProps {
    #[prop_or_default]
    pub message: Option<AttrValue>,
}

/// Centered amber spinner with optional label.
#[function_component(Loading)]
pub fn loading(props: &LoadingProps) -> Html {
    html! {
        <div class="flex flex-col items-center justify-center gap-3">
            <div class="w-6 h-6 border-2 border-amber-500 border-t-transparent rounded-full animate-spin" />
            if let Some(msg) = &props.message {
                <p class="text-sm text-stone-400">{ msg }</p>
            }
        </div>
    }
}