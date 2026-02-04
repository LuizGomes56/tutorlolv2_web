#![allow(static_mut_refs)]
use crate::utils::cache::CACHE;
use std::{fmt::Display, ops::Range};
use tutorlolv2_gen::{
    ABILITY_FORMULAS, AbilityId, BASIC_ATTACK_OFFSET, CHAMPION_ABILITIES, CRITICAL_STRIKE_OFFSET,
    ChampionId, ITEM_ID_TO_RIOT_ID, ItemId, MergeData, ONHIT_EFFECT_OFFSET, RUNE_ID_TO_RIOT_ID,
    RuneId, TOWER_DAMAGE_OFFSET,
};
use web_sys::js_sys::Math;
use yew::prelude::*;

pub mod cache;
pub mod fetch;
pub mod hooks;

pub const BASE_URL: &str = "http://localhost:8082";

#[derive(Clone, Copy, Debug, PartialEq)]
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

#[derive(Clone, Copy, PartialEq)]
pub enum StatType {
    AbilityPower,
    Armor,
    ArmorPenetrationFlat,
    ArmorPenetrationPercent,
    AttackDamage,
    AttackRange,
    AttackSpeed,
    CritChance,
    CritDamage,
    CurrentHealth,
    MagicPenetrationFlat,
    MagicPenetrationPercent,
    MagicResist,
    Health,
    Mana,
    CurrentMana,
}

impl Display for StatType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatType::AbilityPower => write!(f, "Ability Power"),
            StatType::Armor => write!(f, "Armor"),
            StatType::ArmorPenetrationFlat => write!(f, "Armor Pen. Flat"),
            StatType::ArmorPenetrationPercent => write!(f, "Armor Pen. %"),
            StatType::AttackDamage => write!(f, "Attack Damage"),
            StatType::AttackRange => write!(f, "Attack Range"),
            StatType::AttackSpeed => write!(f, "Attack Speed"),
            StatType::CritChance => write!(f, "Crit Chance"),
            StatType::CritDamage => write!(f, "Crit Damage"),
            StatType::CurrentHealth => write!(f, "Current Health"),
            StatType::MagicPenetrationFlat => write!(f, "Magic Pen. Flat"),
            StatType::MagicPenetrationPercent => write!(f, "Magic Pen. %"),
            StatType::MagicResist => write!(f, "Magic Resist"),
            StatType::Health => write!(f, "Max Health"),
            StatType::Mana => write!(f, "Max Mana"),
            StatType::CurrentMana => write!(f, "Current Mana"),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum ImageType {
    Ability(ChampionId, AbilityKind),
    Champion(ChampionId),
    Centered(ChampionId),
    Item(ItemId),
    Rune(RuneId),
    BasicAttack,
    OnhitAttack,
    CritStrike,
    Level,
    Stats(StatType),
    Tower,
}

pub fn encode_offset(tuple: Option<Range<usize>>) -> Option<u64> {
    tuple.map(|range| range.start as u64 * (1 << 23) + range.end as u64)
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
                        {char}{name.map(|name| html!(<sub>{name}</sub>))}
                    </div>
                })
            }
            _ => None,
        }
    }

    pub fn offset(&self) -> (Option<u64>, Option<u64>) {
        let mut tuple_exc = None;
        let tuple_main = match self {
            ImageType::Ability(champion_id, kind) => {
                let index = champion_id.index();
                let array = ABILITY_FORMULAS[index];
                let abilities = CHAMPION_ABILITIES[index];
                match kind {
                    AbilityKind::Normal(ability_id) => abilities
                        .iter()
                        .position(|id| id == ability_id)
                        .map(|i| array[i].clone()),
                    AbilityKind::Alias(merge) => {
                        tuple_exc = Some(array[merge.maximum_damage as usize].clone());
                        Some(array[merge.minimum_damage as usize].clone())
                    }
                }
            }
            ImageType::Champion(champion_id) => Some(champion_id.offset()),
            ImageType::Item(item_id) => Some(item_id.offset()),
            ImageType::Rune(rune_id) => Some(rune_id.offset()),
            ImageType::BasicAttack => Some(BASIC_ATTACK_OFFSET.clone()),
            ImageType::OnhitAttack => Some(ONHIT_EFFECT_OFFSET.clone()),
            ImageType::CritStrike => Some(CRITICAL_STRIKE_OFFSET.clone()),
            ImageType::Tower => Some(TOWER_DAMAGE_OFFSET.clone()),
            _ => None,
        };

        (encode_offset(tuple_main), encode_offset(tuple_exc))
    }

    pub fn url(&self) -> String {
        let path = match self {
            ImageType::Ability(champion_id, kind) => {
                let char = kind.as_char();
                format!("abilities/{champion_id:?}{char}.avif")
            }
            ImageType::Champion(champion_id) => {
                format!("champions/{champion_id:?}.avif")
            }
            ImageType::Centered(champion_id) => {
                format!("centered/{champion_id:?}_0.avif")
            }
            ImageType::Item(item_id) => {
                let riot_id = ITEM_ID_TO_RIOT_ID[item_id.index()];
                format!("items/{riot_id:?}.avif")
            }
            ImageType::Rune(rune_id) => {
                let riot_id = RUNE_ID_TO_RIOT_ID[rune_id.index()];
                format!("runes/{riot_id:?}.avif")
            }
            ImageType::Tower => "other/tower.avif".into(),
            ImageType::BasicAttack => "other/basic_attack.png".into(),
            ImageType::CritStrike => "stats/crit_chance.svg".into(),
            ImageType::OnhitAttack => "stats/onhit.svg".into(),
            ImageType::Level => "stats/level.svg".into(),
            ImageType::Stats(stat) => match stat {
                StatType::AbilityPower => "stats/ability_power.svg",
                StatType::Armor => "stats/armor.svg",
                StatType::ArmorPenetrationFlat => "stats/armor_penetration.svg",
                StatType::ArmorPenetrationPercent => "stats/armor_penetration.svg",
                StatType::AttackDamage => "stats/attack_damage.svg",
                StatType::AttackRange => "stats/onhit.svg",
                StatType::AttackSpeed => "stats/attack_speed.svg",
                StatType::CritChance => "stats/crit_chance.svg",
                StatType::CritDamage => "stats/crit_damage.svg",
                StatType::CurrentHealth => "stats/health.svg",
                StatType::MagicPenetrationFlat => "stats/magic_penetration.svg",
                StatType::MagicPenetrationPercent => "stats/magic_penetration.svg",
                StatType::MagicResist => "stats/magic_resist.svg",
                StatType::Health => "stats/health.svg",
                StatType::Mana => "stats/mana.svg",
                StatType::CurrentMana => "stats/mana.svg",
            }
            .into(),
        };

        format!("{BASE_URL}/img/{path}")
    }
}

macro_rules! impl_base {
    ($($ty:tt),+) => {
        $(
            pastey::paste! {
                impl EnumCast for $ty {
                    const FORMULAS: &[Range<usize>] = &tutorlolv2_gen::[<$ty:replace("Id", ""):upper _FORMULAS>];
                    const NAMES: &[&'static str] = &tutorlolv2_gen::[<$ty:replace("Id", ""):upper _ID_TO_NAME>];
                    const ARRAY: &[$ty] = &tutorlolv2_gen::$ty::ARRAY;
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

pub trait EnumCast:
    PartialEq + Copy + Into<ImageType> + Into<usize> + TryFrom<u16> + 'static
{
    const FORMULAS: &[Range<usize>];
    const NAMES: &[&'static str];
    const ARRAY: &[Self];
    fn index(&self) -> usize {
        (*self).into()
    }
    fn name(&self) -> &'static str {
        Self::NAMES[self.index()]
    }
    fn random() -> Self {
        let index = random_u16(const { 0..Self::FORMULAS.len() as u16 });
        unsafe { Self::try_from(index).unwrap_unchecked() }
    }
    fn image_type(&self) -> ImageType {
        (*self).into()
    }
    fn offset(&self) -> Range<usize> {
        Self::FORMULAS[self.index()].clone()
    }
    fn docs(&self) -> &'static str {
        let offset = self.offset();
        get_cache(offset)
    }
    fn html(&self) -> Html {
        Html::from_html_unchecked(self.docs().into())
    }
}

pub fn get_cache(offsets: Range<usize>) -> &'static str {
    unsafe { core::str::from_utf8_unchecked(CACHE.get_unchecked(offsets)) }
}
