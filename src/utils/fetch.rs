use crate::utils::BASE_URL;
use bincode::{Decode, Encode, config::Configuration};
use gloo_net::http::Headers;
use std::error::Error;
use web_sys::AbortSignal;

const CONFIG: Configuration = bincode::config::standard();

pub struct Fetch<'a> {
    url: &'a str,
    signal: Option<AbortSignal>,
    data: Vec<u8>,
}

impl<'a> Fetch<'a> {
    pub const fn new(url: &'a str) -> Self {
        Self {
            url,
            signal: None,
            data: Vec::new(),
        }
    }

    pub fn signal(mut self, signal: Option<AbortSignal>) -> Self {
        self.signal = signal;
        self
    }

    pub fn set_body(mut self, data: Vec<u8>) -> Self {
        self.data = data;
        self
    }

    pub fn body_with_bincode<T: Encode>(
        mut self,
        data: T,
    ) -> Result<Self, bincode::error::EncodeError> {
        self.data = bincode::encode_to_vec(data, CONFIG)?;
        Ok(self)
    }

    pub async fn post<T: Decode<()>>(self) -> Result<T, Box<dyn Error>> {
        let Self { url, signal, data } = self;
        let target = [BASE_URL, url].concat();
        let builder = gloo_net::http::Request::post(&target);

        let headers = Headers::new();
        headers.set("Content-Type", "application/octet-stream");

        let result = match signal {
            Some(ref signal) => builder.abort_signal(Some(signal)),
            None => builder,
        }
        .headers(headers)
        .body(data)?
        .send()
        .await?
        .binary()
        .await?;

        let (de, _) = bincode::decode_from_slice(result.as_slice(), CONFIG)?;
        Ok(de)
    }
}
