use yew::prelude::*;

/* ============================================================================================== */

#[derive(Properties, PartialEq)]
pub struct DropdownItemProps {
    pub label:    AttrValue,
    pub on_click: Callback<()>,
    #[prop_or_default]
    pub danger: bool,
}

#[function_component(DropdownItem)]
pub fn dropdown_item(props: &DropdownItemProps) -> Html {
    let on_click = {
        let cb = props.on_click.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };
    let class = if props.danger {
        "w-full text-left px-4 py-2 text-sm text-red-600 hover:bg-red-50 transition-colors"
    } else {
        "w-full text-left px-4 py-2 text-sm text-stone-700 hover:bg-stone-50 transition-colors"
    };
    html! {
        <button type="button" class={class} onclick={on_click}>
            { &props.label }
        </button>
    }
}

/* ============================================================================================== */

#[derive(Properties, PartialEq)]
pub struct DropdownProps {
    /// Text shown in the trigger button.
    pub label:    AttrValue,
    pub children: Children,
    /// "right" (default) or "left" — which side the menu aligns to.
    #[prop_or(AttrValue::Static("right"))]
    pub align: AttrValue,
}

#[function_component(Dropdown)]
pub fn dropdown(props: &DropdownProps) -> Html {
    let open = use_state(|| false);

    let toggle = {
        let open = open.clone();
        Callback::from(move |_: MouseEvent| open.set(!*open))
    };
    let close = {
        let open = open.clone();
        Callback::from(move |_: MouseEvent| open.set(false))
    };

    let menu_pos = if props.align == "left" { "left-0" } else { "right-0" };

    html! {
        <div class="relative">
            <button type="button" onclick={toggle}
                class="px-2 py-1 text-sm text-stone-600 hover:text-stone-900 transition-colors">
                { &props.label }
            </button>
            if *open {
                <>
                    // Invisible backdrop — clicking outside closes the menu
                    <div class="fixed inset-0 z-40" onclick={close} />
                    <div class={classes!(
                        "absolute", "mt-1", "w-48", "bg-white", "rounded-lg", "border", "border-stone-200",
                        "shadow-lg", "z-50", "py-1", "overflow-hidden",
                        menu_pos
                    )}>
                        { for props.children.iter() }
                    </div>
                </>
            }
        </div>
    }
}
