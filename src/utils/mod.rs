#![allow(static_mut_refs)]
use crate::utils::cache::CACHE;
use tutorlolv2_gen::{AbilityId, ChampionId, ItemId, RuneId};

pub mod cache;

#[derive(PartialEq)]
pub enum ImageType {
    Ability(ChampionId, AbilityId),
    Champion(ChampionId),
    Item(ItemId),
    Rune(RuneId),
}

macro_rules! impl_base {
    ($($ty:tt),+) => {
        $(
            pastey::paste! {
                impl EnumCast for $ty {
                    const FORMULAS: &[(u32, u32)] = &tutorlolv2_gen::[<$ty:replace("Id", ""):upper _FORMULAS>];
                }

                impl From<$ty> for ImageType {
                    fn from(value: $ty) -> Self {
                        Self::[<$ty:replace("Id", "")>](value)
                    }
                }

                impl From<&$ty> for ImageType {
                    fn from(value: &$ty) -> Self {
                        Self::[<$ty:replace("Id", "")>](*value)
                    }
                }
            }
        )+
    };
}

impl_base!(ChampionId, RuneId, ItemId);

pub trait EnumCast: PartialEq + Copy + Into<ImageType> + Into<usize> + TryFrom<usize> {
    const FORMULAS: &[(u32, u32)];
    fn docs(&self) -> &'static str {
        let offset: usize = (*self).into();
        get_cache(Self::FORMULAS[offset])
    }
    fn image_type(&self) -> ImageType {
        (*self).into()
    }
}

fn get_cache((i, j): (u32, u32)) -> &'static str {
    unsafe { core::str::from_utf8_unchecked(CACHE.get_unchecked(i as usize..j as usize)) }
}
