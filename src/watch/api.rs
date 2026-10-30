use crate::utils::BINCODE_CONFIG;
use std::rc::Rc;
use tutorlolv2::riot::{FrameChunk, RiotRealtime};
use tutorlolv2_protocol::{FRAMES_FORMAT, ShareInfo};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{AbortController, RequestInit, Response, js_sys::Uint8Array};

self_cell::self_cell! {
    pub struct CachedChunk {
        owner: Vec<u8>,
        #[covariant]
        dependent: FrameChunk,
    }
}

impl CachedChunk {
    pub fn decode(bytes: Vec<u8>, format: u16) -> Result<Rc<Self>, String> {
        Self::try_new(bytes, |bytes| {
            let (chunk, consumed): (FrameChunk<'_>, _) =
                bincode::borrow_decode_from_slice(bytes, BINCODE_CONFIG)
                    .map_err(|e| e.to_string())?;

            if consumed != bytes.len() || chunk.format != format {
                return Err("Unsupported or damaged replay chunk. Try refreshing.".into());
            }

            Ok(chunk)
        })
        .map(Rc::new)
    }
}

pub fn api_base() -> String {
    if let Some(base) = option_env!("TUTORLOLV2_API_URL") {
        return base.trim_end_matches('/').into();
    }

    option_env!("TUTORLOL_API_URL")
        .unwrap_or("http://127.0.0.1:8082/api/games")
        .trim_end_matches('/')
        .trim_end_matches("/games")
        .into()
}

fn js_error(_: JsValue) -> String {
    "Could not reach the sharing service. Check your connection and retry.".into()
}

pub async fn get(path: &str, abort: &AbortController) -> Result<Response, String> {
    let options = RequestInit::new();

    options.set_signal(Some(&abort.signal()));

    JsFuture::from(
        web_sys::window()
            .ok_or("Browser unavailable")?
            .fetch_with_str_and_init(&format!("{}/share/{path}", api_base()), &options),
    )
    .await
    .map_err(js_error)?
    .dyn_into()
    .map_err(js_error)
}

pub fn status_error(status: u16) -> String {
    match status {
        404 => "Code not found. Check the code and try again.".into(),
        410 => "This match ended or is no longer shared.".into(),
        _ => format!("Sharing service unavailable (HTTP {status}). Retry to reconnect."),
    }
}

pub async fn bytes(response: &Response) -> Result<Vec<u8>, String> {
    let buffer = JsFuture::from(response.array_buffer().map_err(js_error)?)
        .await
        .map_err(js_error)?;
    Ok(Uint8Array::new(&buffer).to_vec())
}

pub async fn info(code: &str, abort: &AbortController) -> Result<ShareInfo, String> {
    let response = get(code, abort).await?;

    if !response.ok() {
        return Err(status_error(response.status()));
    }

    let bytes = bytes(&response).await?;
    let (mut value, consumed): (ShareInfo, _) =
        bincode::decode_from_slice(&bytes, BINCODE_CONFIG).map_err(|e| e.to_string())?;

    if consumed != bytes.len() {
        return Err("Invalid sharing metadata. Retry to refresh.".into());
    }

    if value.frames_format != FRAMES_FORMAT {
        return Err("This replay needs a newer viewer. Refresh the app and retry.".into());
    }

    value
        .chunks
        .sort_by(|a, b| a.first_game_time.total_cmp(&b.first_game_time));

    Ok(value)
}

pub async fn frame(bytes: &[u8]) -> Result<crate::livegame::Game, String> {
    let (input, consumed): (RiotRealtime<'_>, _) =
        bincode::borrow_decode_from_slice(bytes, BINCODE_CONFIG).map_err(|e| e.to_string())?;

    if consumed != bytes.len() {
        return Err("Invalid live frame".into());
    }

    crate::utils::transport::realtime_from(&input)
        .await
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn borrowed_chunk_keeps_its_buffer_and_rejects_wrong_format_or_trailing_bytes() {
        let fixture = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/example.bincode"));
        let (frame, _): (RiotRealtime<'_>, _) =
            bincode::borrow_decode_from_slice(fixture, BINCODE_CONFIG).unwrap();
        let chunk = FrameChunk {
            format: 1,
            frames: vec![(123, frame)],
        };
        let bytes = bincode::encode_to_vec(&chunk, BINCODE_CONFIG).unwrap();
        let cached = CachedChunk::decode(bytes.clone(), 1).unwrap();
        assert_eq!(cached.borrow_dependent().frames[0].0, 123);
        assert_eq!(cached.borrow_dependent().frames[0].1, chunk.frames[0].1);
        assert!(CachedChunk::decode(bytes.clone(), 2).is_err());
        let mut trailing = bytes;
        trailing.push(0);
        assert!(CachedChunk::decode(trailing, 1).is_err());
    }
    #[test]
    fn code_entry_uses_shared_normalization() {
        assert_eq!(
            tutorlolv2_protocol::normalize_code("abcd efgh"),
            Some("ABCDEFGH".into())
        );
        assert_eq!(
            tutorlolv2_protocol::normalize_code("abcd-efgh"),
            Some("ABCDEFGH".into())
        );
        assert_eq!(tutorlolv2_protocol::normalize_code("bad"), None);
    }
}
