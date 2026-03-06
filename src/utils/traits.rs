use crate::components::image::ImageType;
use std::{fmt::Debug, ops::Range};
use tutorlolv2_gen::{AdaptiveType, CastId, ChampionId, DamageType, ItemId, RuneId};
use web_sys::js_sys::Math;

pub fn random_u64(range: Range<u64>) -> u64 {
    let start = range.start;
    let end = range.end;
    let x = getrandom::u64().unwrap_or_else(
        #[cold]
        |_| (Math::random() * ((end - start) + start) as f64) as _,
    );
    start + (x % (end - start))
}

pub trait EnumCast: CastId + TryFrom<u16> + Into<ImageType> + PartialEq + Copy {
    fn random() -> Self {
        let index = random_u64(0..Self::VARIANTS as _);
        unsafe { Self::try_from(index as _).unwrap_unchecked() }
    }

    fn image_type(&self) -> ImageType {
        (*self).into()
    }
}

impl EnumCast for ChampionId {}
impl EnumCast for ItemId {}
impl EnumCast for RuneId {}

pub trait ClassCast {
    fn class(&self) -> &'static str;
}

impl ClassCast for DamageType {
    fn class(&self) -> &'static str {
        match self {
            DamageType::Physical => "text-orange-500",
            DamageType::Magic => "text-sky-500",
            DamageType::Mixed => "text-indigo-500",
            DamageType::True => "text-white",
            DamageType::Adaptive => "text-purple-500",
            DamageType::Unknown => "text-emerald-500",
        }
    }
}

impl ClassCast for AdaptiveType {
    fn class(&self) -> &'static str {
        match self {
            Self::Magic => DamageType::Magic.class(),
            Self::Physical => DamageType::Physical.class(),
        }
    }
}

pub trait Print: Debug {
    fn log(&self) {
        web_sys::console::log_1(&format!("{self:#?}").into());
    }
    fn err(&self) {
        web_sys::console::error_1(&format!("{self:#?}").into());
    }
}

impl<T: Debug> Print for T {}

pub trait ReduceApply
where
    Self: Copy + PartialEq + 'static,
    Self::Action: PartialEq + Copy,
{
    type Action;
    fn apply(&mut self, action: Self::Action);
}
