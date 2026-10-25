//! Shared Bincode boundary for local calculations and the HTTP calculation service.
use tutorlolv2::{
    model::{InputGame, OutputGame},
    riot::RiotRealtime,
};

#[cfg(not(feature = "local-calculation"))]
use {tutorlolv2::model::Realtime, web_sys::js_sys::Uint8Array};

pub type Error = Box<dyn core::error::Error>;

fn consume_all(consumed: usize, length: usize) -> Result<(), Error> {
    if consumed != length {
        return Err("Unexpected trailing bytes in Bincode payload".into());
    }

    Ok(())
}

pub fn decode_realtime_input(bytes: &[u8]) -> Result<RiotRealtime<'_>, Error> {
    let (input, consumed) = bincode::borrow_decode_from_slice(bytes, bincode::config::standard())?;

    consume_all(consumed, bytes.len())?;

    Ok(input)
}

pub async fn calculator(input: InputGame<'_>) -> Result<OutputGame, Error> {
    #[cfg(feature = "local-calculation")]
    {
        Ok(tutorlolv2::calculator::calculator(input))
    }

    #[cfg(not(feature = "local-calculation"))]
    {
        let body = {
            let bytes = bincode::encode_to_vec(input, bincode::config::standard())?;
            Uint8Array::from(bytes.as_slice())
        };

        let response = post("calculator", body).await?;

        let (output, consumed) =
            bincode::decode_from_slice(&response, bincode::config::standard())?;

        consume_all(consumed, response.len())?;

        Ok(output)
    }
}

pub async fn realtime(
    #[cfg(not(feature = "tauri"))] bytes: &[u8],
    #[cfg(feature = "tauri")] bytes: web_sys::js_sys::Uint8Array,
) -> Result<crate::livegame::Game, Error> {
    #[cfg(feature = "local-calculation")]
    {
        // Borrowed decoding needs the Rust buffer to live through calculation.
        #[cfg(feature = "tauri")]
        let buffer = bytes.to_vec();

        #[cfg(feature = "tauri")]
        let bytes = buffer.as_slice();

        let input = decode_realtime_input(bytes)?;

        let output = tutorlolv2::realtime::realtime(&input).map_err(|error| match error {
            tutorlolv2::realtime::RealtimeError::UnrecognizedCurrentPlayer(player) => {
                format!("Unable to recognize current player with name {player:?}")
            }
        })?;

        Ok(output.into())
    }

    #[cfg(not(feature = "local-calculation"))]
    {
        let bytes = {
            #[cfg(not(feature = "tauri"))]
            {
                #[cfg(debug_assertions)]
                let _ = decode_realtime_input(bytes)?;

                Uint8Array::from(bytes)
            }

            #[cfg(feature = "tauri")]
            {
                #[cfg(debug_assertions)]
                let _ = decode_realtime_input(&bytes.to_vec())?;

                bytes
            }
        };

        let response = post("realtime", bytes).await?;

        let (output, consumed) = bincode::borrow_decode_from_slice::<Realtime<'_>, _>(
            &response,
            bincode::config::standard(),
        )?;

        consume_all(consumed, response.len())?;

        Ok(output.into())
    }
}

#[cfg(not(feature = "local-calculation"))]
async fn post(endpoint: &str, body: Uint8Array) -> Result<Vec<u8>, Error> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Request, RequestInit, Response};

    let base = option_env!("TUTORLOL_API_URL").unwrap_or("http://127.0.0.1:8082/api/games");
    let url = format!("{}/{endpoint}", base.trim_end_matches('/'));

    let options = RequestInit::new();

    options.set_method("POST");
    options.set_body(body.as_ref());

    let request = Request::new_with_str_and_init(&url, &options).map_err(js_error)?;

    request
        .headers()
        .set("Content-Type", "application/octet-stream")
        .map_err(js_error)?;

    request
        .headers()
        .set("Accept", "application/octet-stream")
        .map_err(js_error)?;

    let window = web_sys::window().ok_or("Browser window is unavailable")?;

    let response: Response = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(js_error)?
        .dyn_into()
        .map_err(js_error)?;

    if !response.ok() {
        let detail = JsFuture::from(response.text().map_err(js_error)?)
            .await
            .map_err(js_error)?
            .as_string()
            .unwrap_or_default();

        return Err(format!(
            "Calculation request failed (HTTP {}): {detail}",
            response.status()
        )
        .into());
    }

    let bytes = JsFuture::from(response.array_buffer().map_err(js_error)?)
        .await
        .map_err(js_error)?;

    Ok(Uint8Array::new(&bytes).to_vec())
}

#[cfg(not(feature = "local-calculation"))]
fn js_error(value: wasm_bindgen::JsValue) -> Error {
    format!("Calculation service request failed: {value:?}").into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_is_bincode_and_rejects_trailing_or_truncated_data() {
        let fixture = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/example.bincode"));
        assert!(decode_realtime_input(fixture).is_ok());
        let mut trailing = fixture.to_vec();
        trailing.push(0);
        assert!(decode_realtime_input(&trailing).is_err());
        assert!(decode_realtime_input(&fixture[..fixture.len() / 2]).is_err());
    }
}
