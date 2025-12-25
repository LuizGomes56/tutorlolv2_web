use bincode::{Decode, Encode, config::Configuration};
use gloo_net::http::Headers;
use std::error::Error;
use web_sys::AbortSignal;

use crate::utils::BASE_URL;

const CONFIG: Configuration = bincode::config::standard();

pub async fn post_bytes<T: Decode<()>>(
    url: &str,
    data: impl Encode,
    signal: Option<AbortSignal>,
) -> Result<T, Box<dyn Error>> {
    let bytes = bincode::encode_to_vec(data, CONFIG)?;
    let target = format!("{BASE_URL}{url}");
    let builder = gloo_net::http::Request::post(&target);

    web_sys::console::log_1(&format!("Target is {target:?}").into());

    let headers = Headers::new();
    headers.set("Content-Type", "application/octet-stream");

    let result = match signal {
        Some(ref signal) => builder.abort_signal(Some(signal)),
        None => builder,
    }
    .headers(headers)
    .body(bytes)?
    .send()
    .await?
    .binary()
    .await?;

    let (de, _) = bincode::decode_from_slice(result.as_slice(), CONFIG)?;
    Ok(de)
}
