use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use yew::prelude::*;

use crate::models::dashboard::{BatchPositionUpdate, Panel};
use super::grid_engine::{parse_change_items, make_grid_opts, GridStack};
use super::grid_item::GridItem;

#[derive(Properties, PartialEq)]
pub struct DashboardGridProps {
    pub panels:               Vec<Panel>,
    pub edit_mode:            bool,
    pub on_positions_change:  Callback<Vec<BatchPositionUpdate>>,
    pub on_delete_panel:      Callback<String>,
}

#[function_component(DashboardGrid)]
pub fn dashboard_grid(props: &DashboardGridProps) -> Html {
    let container_ref   = use_node_ref();
    // UseRef so we can hold a non-Clone JS object across renders.
    let grid_instance: Rc<RefCell<Option<GridStack>>> = use_mut_ref(|| None);

    // GridStack v6+ has a built-in MutationObserver: it automatically detects
    // panel divs added or removed by Yew's virtual DOM — no reinit needed.
    // Calling destroy(false) before reinit would strip the gs-* attributes,
    // causing every panel to fall back to the GridStack default of w=1 h=1.
    {
        let container_ref = container_ref.clone();
        let grid_instance = grid_instance.clone();
        let edit_mode     = props.edit_mode;
        let on_change     = props.on_positions_change.clone();

        use_effect_with((), move |_| {
            if let Some(el) = container_ref.cast::<HtmlElement>() {
                let opts = make_grid_opts(edit_mode);
                if let Ok(grid) = GridStack::init_on(&opts, &el) {
                    let cb = Closure::<dyn FnMut(JsValue, JsValue)>::wrap(Box::new(
                        move |_, items| {
                            let updates: Vec<BatchPositionUpdate> = parse_change_items(items)
                                .into_iter()
                                .map(|(id, x, y, w, h)| BatchPositionUpdate {
                                    id, grid_x: x, grid_y: y, grid_w: w, grid_h: h,
                                })
                                .collect();
                            if !updates.is_empty() {
                                on_change.emit(updates);
                            }
                        },
                    ));
                    grid.on("change", cb.as_ref().unchecked_ref());
                    cb.forget();
                    *grid_instance.borrow_mut() = Some(grid);
                }
            }

            // Cleanup: destroy the GridStack instance when the component unmounts.
            let grid_instance = grid_instance.clone();
            move || {
                if let Some(grid) = grid_instance.borrow_mut().take() {
                    grid.destroy(true);
                }
            }
        });
    }

    // Toggle static/interactive when edit_mode flips — no re-init needed.
    {
        let grid_instance = grid_instance.clone();
        use_effect_with(props.edit_mode, move |&edit| {
            if let Some(g) = grid_instance.borrow().as_ref() {
                g.set_static(!edit);
            }
            || ()
        });
    }

    html! {
        <div class="grid-stack w-full" ref={container_ref}>
            { for props.panels.iter().map(|panel| html! {
                <GridItem
                    key={panel.id.clone()}
                    panel={panel.clone()}
                    edit_mode={props.edit_mode}
                    on_delete={props.on_delete_panel.clone()}
                />
            })}
        </div>
    }
}
