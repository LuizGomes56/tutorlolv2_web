use crate::components::image::ImageType;
use std::ops::Range;
use tutorlolv2_gen::{CastId, ChampionId, ItemId, RuneId};
use web_sys::js_sys::Math;

pub fn random_u16(range: Range<u16>) -> u16 {
    (Math::random() * (range.end - range.start) as f64 + range.start as f64) as u16
}

pub trait EnumCast: CastId + TryFrom<u16> + Into<ImageType> + PartialEq + Copy {
    fn random() -> Self {
        let index = random_u16(0..Self::VARIANTS as u16);
        unsafe { Self::try_from(index).unwrap_unchecked() }
    }

    fn image_type(&self) -> ImageType {
        (*self).into()
    }
}

impl EnumCast for ChampionId {}
impl EnumCast for ItemId {}
impl EnumCast for RuneId {}
