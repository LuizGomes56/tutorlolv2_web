//! Persistence and JavaScript callbacks for draggable overlay panels.
use crate::utils::storage::{RecordStorage, encode_record, local_storage, read_record};
use bincode::{Decode, Encode};
use wasm_bindgen::{JsCast, JsValue, prelude::*};
use web_sys::js_sys::{Array, Function};

const PANEL_KIND: u8 = 2;

#[derive(Debug, Default, PartialEq, Encode, Decode)]
struct PanelState {
    left: String,
    top: String,
    scale: f64,
}

#[wasm_bindgen(module = "/public/events.js")]
unsafe extern "C" {
    #[wasm_bindgen(js_name = "mouse_events")]
    fn register_mouse_events(save: &Function, load: &Function);
}

/// Keep pointer handling in JavaScript while Rust owns all persisted records.
pub(super) fn initialize_panel_events() {
    let save = Closure::wrap(
        Box::new(|key: String, left: String, top: String, scale: f64| {
            if let Some(mut storage) = local_storage()
                && let Some(record) = encode_record(PANEL_KIND, &PanelState { left, top, scale })
            {
                storage.put(&key, &record);
            }
        }) as Box<dyn FnMut(String, String, String, f64)>,
    )
    .into_js_value();

    let load = Closure::wrap(Box::new(|key: String| -> JsValue {
        let Some(mut storage) = local_storage() else {
            return JsValue::NULL;
        };

        let Some(state) = read_record::<PanelState>(&mut storage, &key, PANEL_KIND) else {
            return JsValue::NULL;
        };

        let values = Array::new();

        values.push(&state.left.into());
        values.push(&state.top.into());
        values.push(&state.scale.into());
        values.into()
    }) as Box<dyn FnMut(String) -> JsValue>)
    .into_js_value();

    register_mouse_events(save.unchecked_ref(), load.unchecked_ref());
}

const WIDGET_KIND: u8 = 3;
const WIDGET_KEY: &str = "overlay-widget-preferences:v1";

pub(super) fn load_widgets() -> super::model::Panel {
    local_storage()
        .and_then(|mut storage| read_record(&mut storage, WIDGET_KEY, WIDGET_KIND))
        .unwrap_or_default()
}

pub(super) fn save_widgets(panels: &super::model::Panel) -> bool {
    if let Some(mut storage) = local_storage()
        && let Some(record) = encode_record(WIDGET_KIND, panels)
    {
        return storage.put(WIDGET_KEY, &record);
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::storage::decode_record;

    #[test]
    fn panel_state_roundtrips_without_json() {
        let state = PanelState {
            left: "122.5px".to_owned(),
            top: "72px".to_owned(),
            scale: 1.25,
        };
        let encoded = encode_record(PANEL_KIND, &state).unwrap();
        assert_eq!(
            decode_record::<PanelState>(PANEL_KIND, &encoded),
            Some(state)
        );
        assert!(
            decode_record::<PanelState>(
                PANEL_KIND,
                "{\"left\":\"1px\",\"top\":\"2px\",\"scale\":1}"
            )
            .is_none()
        );
    }
}
