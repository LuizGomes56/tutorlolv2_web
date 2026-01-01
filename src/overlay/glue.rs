use crate::{overlay::Game, utils::fetch::Fetch};
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};
use web_sys::{console, js_sys::Uint8Array};

#[wasm_bindgen(module = "/public/invoke.js")]
unsafe extern "C" {
    #[wasm_bindgen(js_name = "invoke_get_live_game", catch)]
    pub async fn get_live_game() -> Result<Uint8Array, JsValue>;
}

pub async fn get_data() -> Result<Game, String> {
    let bytes = get_live_game().await;

    console::log_1(&"Called get_data() function".into());

    match bytes {
        Ok(response) => {
            let bytes = response.to_vec();
            let bytes: Vec<u8> =
                include_bytes!("../../../tutorlolv2_desktop_app/src-tauri/example.json").into();
            match bytes.is_empty() {
                true => Err("Desktop application required to use the overlay feature".into()),
                false => Ok(Fetch::new("/api/games/realtime")
                    .set_body(bytes)
                    .post()
                    .await
                    .unwrap()),
            }
        }
        Err(e) => {
            console::log_1(&format!("[tauri] Error: {e:?}").into());
            Err(
                "[tauri]: Can't access Riot API or you're not playing a League game right now"
                    .into(),
            )
        }
    }
}
