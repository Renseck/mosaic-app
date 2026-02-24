use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ModalProps {
    pub title: AttrValue,
    pub body: AttrValue,
    pub on_confirm: Callback<()>,
    pub on_cancel: Callback<()>,
    #[prop_or_else(|| AttrValue::from("Confirm"))]
    pub confirm_label: AttrValue,
    #[prop_or_else(|| AttrValue::from("Cancel"))]
    pub cancel_label: AttrValue,
    /// When true the confirm button is styled red.
    #[prop_or(false)]
    pub destructive: bool,
}

#[function_component(Modal)]
pub fn modal(props: &ModalProps) -> Html {
    let confirm_class = if props.destructive {
        "px-4 py-2 text-sm font-medium rounded bg-red-600 text-white \
         hover:bg-red-700 transition-colors"
    } else {
        "px-4 py-2 text-sm font-medium rounded bg-amber-500 text-stone-900 \
         hover:bg-amber-400 transition-colors"
    };

    html! {
        // Backdrop
        <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
            // Panel
            <div class="bg-white dark:bg-stone-800 rounded-lg shadow-xl max-w-sm w-full mx-4 p-6">
                <h2 class="text-base font-semibold text-stone-900 dark:text-stone-100 mb-2">
                    { &props.title }
                </h2>
                <p class="text-sm text-stone-500 dark:text-stone-400 mb-6">
                    { &props.body }
                </p>
                <div class="flex justify-end gap-3">
                    <button
                        onclick={props.on_cancel.reform(|_: MouseEvent| ())}
                        class="px-4 py-2 text-sm font-medium rounded text-stone-600 \
                               dark:text-stone-300 hover:bg-stone-100 dark:hover:bg-stone-700 \
                               transition-colors"
                    >
                        { &props.cancel_label }
                    </button>
                    <button
                        onclick={props.on_confirm.reform(|_: MouseEvent| ())}
                        class={confirm_class}
                    >
                        { &props.confirm_label }
                    </button>
                </div>
            </div>
        </div>
    }
}
