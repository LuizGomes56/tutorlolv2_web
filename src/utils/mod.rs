#![allow(static_mut_refs)]
use crate::utils::cache::CACHE;
use tutorlolv2_gen::{
    ABILITY_FORMULAS, AbilityId, BASIC_ATTACK_OFFSET, CHAMPION_ABILITIES, CHAMPION_FORMULAS,
    CRITICAL_STRIKE_OFFSET, ChampionId, ITEM_FORMULAS, ITEM_ID_TO_RIOT_ID, ItemId, MergeData,
    RUNE_FORMULAS, RUNE_ID_TO_RIOT_ID, RuneId,
};
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
        match self {
            AbilityKind::Alias(merge) => merge.alias.as_char(),
            AbilityKind::Normal(ability_id) => ability_id.as_char(),
        }
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
                    <div class={classes!("img-letter", "text-sm")}>
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
                let offset = *champion_id as usize;
                let array = ABILITY_FORMULAS[offset];
                let abilities = CHAMPION_ABILITIES[offset];
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
            ImageType::Champion(champion_id) => CHAMPION_FORMULAS[*champion_id as usize],
            ImageType::Item(item_id) => ITEM_FORMULAS[*item_id as usize],
            ImageType::Rune(rune_id) => RUNE_FORMULAS[*rune_id as usize],
            ImageType::BasicAttack => BASIC_ATTACK_OFFSET,
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

pub trait EnumCast: PartialEq + Copy + Into<ImageType> + Into<usize> + TryFrom<usize> {
    const FORMULAS: &[(u32, u32)];
    fn docs(&self) -> Html {
        let offset: usize = (*self).into();
        Html::from_html_unchecked(get_cache(Self::FORMULAS[offset]).into())
    }
    fn image_type(&self) -> ImageType {
        (*self).into()
    }
}

fn get_cache((i, j): (u32, u32)) -> &'static str {
    unsafe { core::str::from_utf8_unchecked(CACHE.get_unchecked(i as usize..j as usize)) }
}
