use crate::{components::image::ImageType, utils::random_u64};
use std::fmt::Debug;
use tutorlolv2_gen::{AdaptiveType, CastId, ChampionId, DamageType, ItemId, RuneId};

pub trait EnumCast
where
    Self: CastId + TryFrom<u16> + PartialEq + Copy,
    ImageType: From<Self>,
{
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
