use crate::{livegame::Game, utils::transport};

#[cfg(feature = "tauri")]
use {
    crate::utils::Print,
    wasm_bindgen::{JsValue, prelude::wasm_bindgen},
    web_sys::js_sys::Uint8Array,
};

#[cfg(feature = "tauri")]
#[wasm_bindgen(module = "/public/invoke.js")]
unsafe extern "C" {
    #[wasm_bindgen(js_name = "invoke_get_live_game", catch)]
    pub async fn get_live_game() -> Result<Uint8Array, JsValue>;
}

/// Tauri supplies a live JSON RiotRealtime payload; browser builds use a Bincode fixture.
/// Calculation mode is independent of the input source.
pub async fn get_desktop_data() -> Result<Game, Box<dyn core::error::Error>> {
    #[cfg(feature = "tauri")]
    let bytes = get_live_game().await.map_err(|error| {
        format!("[tauri] Error: {error:?}").log();
        "[tauri]: Can't access Riot API or you're not playing League right now"
    })?;

    #[cfg(not(feature = "tauri"))]
    let bytes = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/example.bincode")).as_slice();

    transport::realtime(bytes).await
}
