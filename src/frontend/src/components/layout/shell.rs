use yew::prelude::*;
use super::{sidebar::Sidebar, topbar::Topbar};

#[derive(Properties, PartialEq)]
pub struct ShellProps {
    pub children: Children,
    /// Forwarded to the topbar's actions slot
    #[prop_or_default]
    pub topbar_actions: Html,
}

/* ============================================================================================== */
#[function_component(Shell)]
pub fn shell(props: &ShellProps) -> Html {
    html! {
        <div class="flex h-screen overflow-hidden bg-stone-50">
            <Sidebar />
            <div class="flex flex-col flex-1 min-w-0 overflow-hidden">
                <Topbar actions={props.topbar_actions.clone()} />
                <main class="flex-1 overflow-auto p-6">
                    { for props.children.iter() }
                </main>
            </div>
        </div>
    }
}