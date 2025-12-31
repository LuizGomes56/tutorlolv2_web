#![allow(static_mut_refs)]
use std::{hint::unreachable_unchecked, ops::Range};

use crate::utils::cache::CACHE;
use tutorlolv2_gen::{
    ABILITY_FORMULAS, AbilityId, BASIC_ATTACK_OFFSET, CHAMPION_ABILITIES, CRITICAL_STRIKE_OFFSET,
    ChampionId, ITEM_ID_TO_RIOT_ID, ItemId, MergeData, ONHIT_EFFECT_OFFSET, RUNE_ID_TO_RIOT_ID,
    RuneId,
};
use web_sys::js_sys::Math;
use yew::prelude::*;

pub mod cache;
pub mod fetch;

pub const BASE_URL: &str = "http://localhost:8082";

#[derive(Debug, PartialEq)]
pub enum AbilityKind {
    Alias(MergeData),
    Normal(AbilityId),
}

impl AbilityKind {
    pub const fn ability_id(&self) -> AbilityId {
        match self {
            AbilityKind::Alias(merge) => merge.alias,
            AbilityKind::Normal(ability_id) => *ability_id,
        }
    }

    pub const fn as_char(&self) -> char {
        self.ability_id().as_char()
    }
}

impl From<AbilityId> for AbilityKind {
    fn from(value: AbilityId) -> Self {
        AbilityKind::Normal(value)
    }
}

#[derive(PartialEq)]
pub enum ImageType {
    Ability(ChampionId, AbilityKind),
    Champion(ChampionId),
    Item(ItemId),
    Rune(RuneId),
    BasicAttack,
    OnhitAttack,
    CritStrike,
}

impl ImageType {
    pub fn header(&self) -> Option<Html> {
        match self {
            ImageType::Ability(_, kind) => {
                let ability_id = kind.ability_id();
                let char = ability_id.as_char();
                let name = ability_id.ability_name().display();
                Some(html! {
                    <div class={classes!("img-letter", "text-xs")}>
                        {char}{match name {
                        Some(name) => Some(html!(<sub>{name}</sub>)),
                        None => None
                    }}
                    </div>
                })
            }
            _ => None,
        }
    }

    pub fn offset(&self) -> (String, Option<String>) {
        let mut tuple_exc = None;
        let tuple_main = match self {
            ImageType::Ability(champion_id, kind) => {
                let index = champion_id.index();
                let array = ABILITY_FORMULAS[index];
                let abilities = CHAMPION_ABILITIES[index];
                match kind {
                    AbilityKind::Normal(ability_id) => {
                        array[abilities.iter().position(|id| id == ability_id).unwrap()]
                    }
                    AbilityKind::Alias(merge) => {
                        tuple_exc = Some(array[merge.maximum_damage as usize]);
                        array[merge.minimum_damage as usize]
                    }
                }
            }
            ImageType::Champion(champion_id) => champion_id.offset(),
            ImageType::Item(item_id) => item_id.offset(),
            ImageType::Rune(rune_id) => rune_id.offset(),
            ImageType::BasicAttack => BASIC_ATTACK_OFFSET,
            ImageType::OnhitAttack => ONHIT_EFFECT_OFFSET,
            ImageType::CritStrike => CRITICAL_STRIKE_OFFSET,
        };

        let encode = |tuple| {
            let (start, end) = tuple;
            (start as u64 * (1 << 23) + end as u64).to_string()
        };

        (
            encode(tuple_main),
            match tuple_exc {
                Some(tuple) => Some(encode(tuple)),
                None => None,
            },
        )
    }

    pub fn url(&self) -> String {
        match self {
            ImageType::Ability(champion_id, kind) => {
                let char = kind.as_char();
                format!("{BASE_URL}/img/abilities/{champion_id:?}{char}.avif")
            }
            ImageType::Champion(champion_id) => {
                format!("{BASE_URL}/img/champions/{champion_id:?}.avif")
            }
            ImageType::Item(item_id) => {
                let riot_id = ITEM_ID_TO_RIOT_ID[*item_id as usize];
                format!("{BASE_URL}/img/items/{riot_id:?}.avif")
            }
            ImageType::Rune(rune_id) => {
                let riot_id = RUNE_ID_TO_RIOT_ID[*rune_id as usize];
                format!("{BASE_URL}/img/runes/{riot_id:?}.avif")
            }
            ImageType::BasicAttack => format!("{BASE_URL}/img/other/basic_attack.png"),
            ImageType::CritStrike => format!("{BASE_URL}/img/stats/crit_chance.svg"),
            ImageType::OnhitAttack => format!("{BASE_URL}/img/stats/onhit.svg"),
        }
    }
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

fn random_u16(range: Range<u16>) -> u16 {
    (Math::random() * (range.end - range.start) as f64 + range.start as f64) as u16
}

pub trait EnumCast: PartialEq + Copy + Into<ImageType> + Into<usize> + TryFrom<u16> {
    const FORMULAS: &[(u32, u32)];
    fn random() -> Self {
        let index = random_u16(const { 0..Self::FORMULAS.len() as u16 });
        unsafe { Self::try_from(index).unwrap_unchecked() }
    }
    fn image_type(&self) -> ImageType {
        (*self).into()
    }
    fn offset(&self) -> (u32, u32) {
        let offset: usize = (*self).into();
        Self::FORMULAS[offset]
    }
    fn docs(&self) -> &'static str {
        let offset = self.offset();
        get_cache(offset)
    }
    fn html(&self) -> Html {
        Html::from_html_unchecked(self.docs().into())
    }
}

pub fn get_cache(offsets: (u32, u32)) -> &'static str {
    let (i, j) = offsets;
    unsafe { core::str::from_utf8_unchecked(CACHE.get_unchecked(i as usize..j as usize)) }
}
