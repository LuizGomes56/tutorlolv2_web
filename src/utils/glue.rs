use crate::{
    livegame::Game,
    utils::{fetch::Fetch, traits::Print},
};
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};
use web_sys::js_sys::Uint8Array;

#[wasm_bindgen(module = "/public/invoke.js")]
unsafe extern "C" {
    #[wasm_bindgen(js_name = "invoke_get_live_game", catch)]
    pub async fn get_live_game() -> Result<Uint8Array, JsValue>;
}

pub async fn get_data() -> Result<Game, Box<dyn core::error::Error>> {
    let bytes = get_live_game().await;

    "Called get_data() function".log();

    async fn realtime(bytes: Vec<u8>) -> Result<Game, String> {
        match bytes.is_empty() {
            true => Err("Desktop application required to use the overlay feature".into()),
            false => Ok(Fetch::new("/api/games/realtime")
                .set_body(bytes)
                .post()
                .await
                .map_err(|e| format!("[gloo_net] Error: {e:?}"))?),
        }
    }

    match bytes {
        Ok(response) => {
            let bytes = response.to_vec();
            realtime(bytes)
        }
        Err(e) => {
            // format!("[tauri] Error: {e:?}").log();
            // Err("[tauri]: Can't access Riot API or you're not playing League right now".into())
            let bytes =
                include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/example.json")).to_vec();
            realtime(bytes)
        }
    }
    .await
    .map_err(|e: String| e.into())
}
