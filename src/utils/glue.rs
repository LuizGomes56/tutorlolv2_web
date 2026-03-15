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

pub async fn get_data() -> Result<Game, Box<dyn core::error::Error>> {
    let bytes = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/example.json")).to_vec();
    return realtime(bytes).await.map_err(|e: String| e.into());

    let bytes = get_live_game().await;

    "Called get_data() function".log();

    match bytes {
        Ok(response) => {
            "ok".log();
            let bytes = response.to_vec();
            realtime(bytes).await
        }
        Err(e) => {
            format!("[tauri] Error: {e:?}").log();
            Err("[tauri]: Can't access Riot API or you're not playing League right now".into())
        }
    }
    .map_err(|e: String| e.into())
}
