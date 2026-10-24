use crate::{components::image::ImageType, utils::random_u64};
use std::fmt::Debug;
use strum::EnumCount;
use tutorlolv2::{AdaptiveType, CastId, ChampionId, DamageType, ItemId, RuneId};

pub trait EnumCast
where
    Self: CastId + PartialEq + Copy + EnumCount,
    ImageType: From<Self>,
{
    fn random() -> Self {
        let index = random_u64(0..Self::COUNT as _);
        unsafe { Self::try_from_u16(index as _).unwrap_unchecked() }
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
            DamageType::Unspecified => "text-std-300",
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
}

impl<T: Debug> Print for T {}
