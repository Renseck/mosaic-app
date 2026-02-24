use yew_router::prelude::*;

#[derive(Debug, Clone, PartialEq, Routable)]
pub enum Route {
    #[at("/login")]
    Login,
    #[at("/dashboards/:slug")]
    DashboardView { slug: String },
    #[at("/dashboards")]
    DashboardList,
    #[at("/templates/new")]
    TemplateNew,
    #[at("/templates")]
    TemplateList,
    #[at("/settings")]
    Settings,
    #[at("/admin/users")]
    AdminUsers,
    #[at("/")]
    Home,
    #[not_found]
    #[at("/404")]
    NotFound,
}