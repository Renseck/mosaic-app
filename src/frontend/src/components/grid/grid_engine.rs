use js_sys::Array;
use wasm_bindgen::prelude::*;

/// Bindings for the GridStack v10 library (loaded via CDN).
#[wasm_bindgen]
extern "C" {
    pub type GridStack;

    /// `GridStack.init(options, element)` - initialize on a specific DOM element.
    #[wasm_bindgen(static_method_of = GridStack, js_name = "init", catch)]
    pub fn init_on(opts: &JsValue, el: &web_sys::HtmlElement) -> Result<GridStack, JsValue>;

    /// Register a named event handler. The `change` event fires after drag/resize.
    #[wasm_bindgen(method)]
    pub fn on(this: &GridStack, event: &str, cb: &JsValue);

    /// Enable (`false`) or disable (`true`) drag and resize.
    #[wasm_bindgen(method, js_name = "setStatic")]
    pub fn set_static(this: &GridStack, val: bool);

    /// Tear down the grid instance. Pass `false` to keep DOM nodes (Yew owns them).
    #[wasm_bindgen(method)]
    pub fn destroy(this: &GridStack, remove_dom: bool);
}

/* ============================================================================================== */
/// Parse the items array from a GridStack `change` event.
/// Returns `Vec<(panel_id, x, y, w, h)>` - one entry per moved/resized widget.
pub fn parse_change_items(items: JsValue) -> Vec<(String, i32, i32, i32, i32)> {
    Array::from(&items)
        .iter()
        .filter_map(|item| {
            let get = |key: &str| js_sys::Reflect::get(&item, &key.into()).ok();
            let x = get("x")?.as_f64()? as i32;
            let y = get("y")?.as_f64()? as i32;
            let w = get("w")?.as_f64()? as i32;
            let h = get("h")?.as_f64()? as i32;
            let el: web_sys::Element = get("el")?.dyn_into().ok()?;
            let panel_id = el.get_attribute("data-panel-id")?;
            Some((panel_id, x, y, w, h))
        })
        .collect()
}

/* ============================================================================================== */
/// Build the GridStack init options JsValue.
pub fn make_grid_opts(edit_mode: bool) -> JsValue {
    let opts = js_sys::Object::new();
    let set = |k: &str, v: JsValue| {
        js_sys::Reflect::set(&opts, &k.into(), &v).ok();
    };
    set("column",     JsValue::from_f64(12.0));
    set("cellHeight", JsValue::from_f64(80.0));
    set("animate",    JsValue::from_bool(true));
    set("float",      JsValue::from_bool(false));
    set("staticGrid", JsValue::from_bool(!edit_mode));
    opts.into()
}