use yew::prelude::*;
use crate::models::dashboard::Panel;
use crate::components::panels::panel_frame::PanelFrame;

#[derive(Properties, PartialEq)]
pub struct GridItemProps {
    pub panel:     Panel,
    pub edit_mode: bool,
    pub on_delete: Callback<String>,
}

/* ============================================================================================== */
/// A single GridStack widget. The outer div carries the `gs-*` data attributes
/// that GridStack reads for initial placement; the inner div is the content area.
#[function_component(GridItem)]
pub fn grid_item(props: &GridItemProps) -> Html {
    let panel = &props.panel;
    html !{
        <div
            class="grid-stack-item"
            gs-x={panel.grid_x.to_string()}
            gs-y={panel.grid_y.to_string()}
            gs-w={panel.grid_w.to_string()}
            gs-h={panel.grid_h.to_string()}
            data-panel-id={panel.id.clone()}
        >
            <div class="grid-stack-item-content">
                <PanelFrame
                    panel={panel.clone()}
                    edit_mode={props.edit_mode}
                    on_delete={props.on_delete.clone()}
                />
            </div>
        </div>
    }
}