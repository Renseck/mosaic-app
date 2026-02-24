use yew::prelude::*;
use super::{sidebar::Sidebar, topbar::Topbar};

#[derive(Properties, PartialEq)]
pub struct ShellProps {
    pub children: Children,
    #[prop_or_default]
    pub topbar_actions: Html,
}

/* ============================================================================================== */
#[function_component(Shell)]
pub fn shell(props: &ShellProps) -> Html {
    let sidebar_open = use_state(|| false);

    let on_toggle = {
        let sidebar_open = sidebar_open.clone();
        Callback::from(move |_: ()| sidebar_open.set(!*sidebar_open))
    };
    let on_close = {
        let sidebar_open = sidebar_open.clone();
        Callback::from(move |_: ()| sidebar_open.set(false))
    };

    html! {
        <div class="flex h-screen overflow-hidden bg-stone-50 dark:bg-stone-950">
            <Sidebar is_open={*sidebar_open} on_close={on_close} />
            <div class="flex flex-col flex-1 min-w-0 overflow-hidden">
                <Topbar
                    actions={props.topbar_actions.clone()}
                    on_menu_toggle={on_toggle}
                />
                <main class="flex-1 overflow-auto p-6">
                    { for props.children.iter() }
                </main>
            </div>
        </div>
    }
}
