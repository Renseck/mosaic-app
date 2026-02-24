use std::rc::Rc;
use yew::prelude::*;

/* ============================================================================================== */
/*                                              Types                                             */
/* ============================================================================================== */

#[derive(Clone, PartialEq, Debug)]
pub enum ToastKind {
    Success,
    Error,
    Info,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Toast {
    pub id: u32,
    pub message: String,
    pub kind: ToastKind,
}

/* ============================================================================================== */
/*                                              State                                             */
/* ============================================================================================== */

#[derive(Clone, PartialEq, Debug, Default)]
pub struct ToastState {
    pub toasts: Vec<Toast>,
}

pub enum ToastAction {
    Push(Toast),
    Dismiss(u32),
}

impl Reducible for ToastState {
    type Action = ToastAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            ToastAction::Push(t) => {
                let mut toasts = self.toasts.clone();
                toasts.push(t);
                Rc::new(ToastState { toasts })
            }
            ToastAction::Dismiss(id) => {
                let toasts = self.toasts.iter().filter(|t| t.id != id).cloned().collect();
                Rc::new(ToastState { toasts })
            }
        }
    }
}

/* ============================================================================================== */
/*                                         Context + Hook                                        */
/* ============================================================================================== */

pub type ToastContext = UseReducerHandle<ToastState>;

/// Returns a callback that shows a toast and auto-dismisses it after 4 seconds.
#[hook]
pub fn use_toast() -> Callback<(String, ToastKind)> {
    let ctx = use_context::<ToastContext>().expect("ToastContext not found");
    Callback::from(move |(message, kind): (String, ToastKind)| {
        let id = (js_sys::Math::random() * u32::MAX as f64) as u32;
        ctx.dispatch(ToastAction::Push(Toast { id, message, kind }));
        let ctx_dismiss = ctx.clone();
        wasm_bindgen_futures::spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(4_000).await;
            ctx_dismiss.dispatch(ToastAction::Dismiss(id));
        });
    })
}

/* ============================================================================================== */
/*                                           Provider                                            */
/* ============================================================================================== */

#[derive(Properties, PartialEq)]
pub struct ToastProviderProps {
    pub children: Children,
}

#[function_component(ToastProvider)]
pub fn toast_provider(props: &ToastProviderProps) -> Html {
    let state = use_reducer(ToastState::default);
    html! {
        <ContextProvider<ToastContext> context={state}>
            { for props.children.iter() }
            <ToastContainer />
        </ContextProvider<ToastContext>>
    }
}

/* ============================================================================================== */
/*                                         Toast container                                       */
/* ============================================================================================== */

#[function_component(ToastContainer)]
fn toast_container() -> Html {
    let ctx = use_context::<ToastContext>().expect("ToastContext not found");
    html! {
        <div class="fixed bottom-4 right-4 z-50 flex flex-col gap-2 pointer-events-none">
            { for ctx.toasts.iter().map(|t| html! {
                <ToastItem key={t.id} toast={t.clone()} />
            })}
        </div>
    }
}

/* ============================================================================================== */
/*                                          Single toast                                         */
/* ============================================================================================== */

#[derive(Properties, PartialEq)]
struct ToastItemProps {
    toast: Toast,
}

#[function_component(ToastItem)]
fn toast_item(props: &ToastItemProps) -> Html {
    let ctx = use_context::<ToastContext>().expect("ToastContext not found");
    let id = props.toast.id;
    let on_dismiss = {
        let ctx = ctx.clone();
        Callback::from(move |_: MouseEvent| ctx.dispatch(ToastAction::Dismiss(id)))
    };

    let (bg, fg) = match props.toast.kind {
        ToastKind::Success => ("bg-emerald-600", "text-white"),
        ToastKind::Error   => ("bg-red-600",     "text-white"),
        ToastKind::Info    => ("bg-slate-700",   "text-white"),
    };

    html! {
        <div class={format!(
            "pointer-events-auto flex items-center gap-3 px-4 py-3 rounded-lg shadow-lg \
             min-w-64 max-w-sm {} {}",
            bg, fg
        )}>
            <span class="flex-1 text-sm">{ &props.toast.message }</span>
            <button
                onclick={on_dismiss}
                class="text-white/70 hover:text-white text-xs leading-none"
            >
                { "✕" }
            </button>
        </div>
    }
}
