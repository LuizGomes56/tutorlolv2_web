use derive_more::Display;
use std::ops::Range;
use web_sys::js_sys::Math;

mod fetch;
pub mod glue;
pub mod hooks;
mod macros;
mod traits;
pub mod tray;

pub use self::{
    fetch::{Fetch, FetchUrl},
    hooks::use_setter,
    traits::{ClassCast, EnumCast, Print, ReduceApply},
};

pub const BASE_URL: &str = "http://localhost:8082";

pub fn random_u64(range: Range<u64>) -> u64 {
    let start = range.start;
    let end = range.end;

    match end - start {
        0 => 0,
        gap => {
            let x = getrandom::u64().unwrap_or_else(
                #[cold]
                |_| (Math::random() * (gap + start) as f64) as _,
            );
            start + (x % gap)
        }
    }
}

#[derive(Debug, Display)]
#[display("Loading")]
pub struct Loading;

impl std::error::Error for Loading {}
