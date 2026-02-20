use yew::prelude::*;
use super::{sidebar::Sidebar, topbar::Topbar};

#[derive(Properties, PartialEq)]
pub struct ShellProps {
    pub children: Children,
}

/* ============================================================================================== */
#[function_component(Shell)]
pub fn shell(props: &ShellProps) -> Html {
    html! {
        <div class="flex h-screen overflow-hidden">
            <Sidebar />
            <div class="flex flex-col flex-1 min-w-0 overflow-hidden">
                <Topbar />
                <main class="flex-l overflow-auto p-6 bg-gray-50">
                    { for props.children.iter() }
                </main>
            </div>
        </div>
    }
}