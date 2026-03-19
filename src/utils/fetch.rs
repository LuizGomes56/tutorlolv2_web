use crate::utils::BASE_URL;
use bincode::{Decode, Encode, config::Configuration};
use gloo_net::http::Headers;
use std::{error::Error, time::Duration};
use web_sys::AbortSignal;

const CONFIG: Configuration = bincode::config::standard();

pub struct Fetch {
    url: FetchUrl,
    signal: Option<AbortSignal>,
    data: Vec<u8>,
}

pub enum FetchUrl {
    Realtime,
    Calculator,
}

impl FetchUrl {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Realtime => "/api/games/realtime",
            Self::Calculator => "/api/games/calculator",
        }
    }
}

impl Fetch {
    pub const REFRESH_RATE: Duration = Duration::from_millis(1000);

    pub const fn new(url: FetchUrl) -> Self {
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

        #[cfg(not(feature = "server"))]
        let result = {
            let target = [BASE_URL, url.as_str()].concat();
            let builder = gloo_net::http::Request::post(&target);

            let headers = Headers::new();
            headers.set("Content-Type", "application/octet-stream");

            match signal {
                Some(ref signal) => builder.abort_signal(Some(signal)),
                None => builder,
            }
            .headers(headers)
            .body(data)?
            .send()
            .await?
            .binary()
            .await?
        };

        #[cfg(feature = "server")]
        let result = match url {
            FetchUrl::Realtime => {
                let game = serde_json::from_slice(data.as_slice())?;
                let data = tutorlolv2::realtime(&game).ok_or("Error on tutorlolv2::realtime")?;
                bincode::encode_to_vec(data, CONFIG)?
            }
            FetchUrl::Calculator => {
                let (game, _) = bincode::decode_from_slice(data.as_slice(), CONFIG)?;
                let data = tutorlolv2::calculator(game);
                bincode::encode_to_vec(data, CONFIG)?
            }
        };

        let (de, _) = bincode::decode_from_slice(result.as_slice(), CONFIG)?;
        Ok(de)
    }
}
