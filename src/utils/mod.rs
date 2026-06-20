use crate::utils::cache::CACHE;
use std::ops::Range;
use web_sys::js_sys::Math;

mod cache;
pub mod glue;
pub mod hooks;
mod macros;
mod traits;
pub mod tray;

pub use {
    cache::init_cache,
    hooks::use_setter,
    traits::{ClassCast, EnumCast, Print},
};

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

pub fn encode_offset(range: &[&Range<usize>]) -> String {
    range
        .iter()
        .map(|r| format!("{r:?}"))
        .collect::<Vec<_>>()
        .join("|")
}

pub fn hoverdocs(offsets: Range<usize>) -> &'static str {
    unsafe { core::str::from_utf8_unchecked(CACHE.get_unchecked(offsets)) }
}

#[derive(Debug)]
pub struct Loading;

impl std::fmt::Display for Loading {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Loading")
    }
}

impl std::error::Error for Loading {}
