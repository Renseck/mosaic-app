mod api;
mod app;
mod components;
mod context;
mod hooks;
mod models;
mod router;

fn main() {
    yew::Renderer::<app::App>::new().render();
}