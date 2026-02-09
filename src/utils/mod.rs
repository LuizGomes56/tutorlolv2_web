use crate::utils::cache::CACHE;
use std::ops::Range;

pub mod cache;
pub mod fetch;
pub mod hooks;
pub mod macros;
pub mod traits;

pub const BASE_URL: &str = "http://localhost:8082";
pub const VOID_MAIN_OFFSET: &str = "0..0";

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

pub use self::{
    fetch::Fetch,
    hooks::use_setter,
    traits::{ClassCast, EnumCast},
};
