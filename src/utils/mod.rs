use crate::utils::cache::CACHE;
use std::ops::Range;
use web_sys::js_sys::Math;

mod cache;
mod fetch;
pub mod glue;
pub mod hooks;
mod macros;
mod traits;

pub use self::{
    cache::init_cache,
    fetch::Fetch,
    hooks::use_setter,
    traits::{ClassCast, EnumCast, Print, ReduceApply},
};

pub const BASE_URL: &str = "http://localhost:8082";

pub fn random_u64(range: Range<u64>) -> u64 {
    let start = range.start;
    let end = range.end;
    let x = getrandom::u64().unwrap_or_else(
        #[cold]
        |_| (Math::random() * ((end - start) + start) as f64) as _,
    );
    start + (x % (end - start))
}

pub fn encode_offset(range: &[&Range<usize>]) -> String {
    range
        .iter()
        .map(|r| format!("{r:?}"))
        .collect::<Vec<_>>()
        .join("|")
}

pub fn get_cache(offsets: Range<usize>) -> &'static str {
    unsafe { core::str::from_utf8_unchecked(CACHE.get_unchecked(offsets)) }
}
