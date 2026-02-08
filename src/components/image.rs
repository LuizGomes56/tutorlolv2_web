use crate::{
    model::{AbilityKind, StatType},
    utils::BASE_URL,
};
use tutorlolv2_gen::{ChampionId, ItemId, Position, RuneId};
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct ImageProps {
    #[prop_or_default]
    pub class: Classes,
    pub src: ImageType,
}

#[component]
pub fn Image(props: &ImageProps) -> Html {
    let ImageProps { class, src } = props;
    let header = src.header();
    let src = src.url();

    let mut classes = classes!("relative");
    classes.push(class);

    match header {
        Some(h) => html! {
            <div class={classes}>
                <img loading={"lazy"} {src} alt={""} />
                {h}
            </div>
        },
        None => html! {
            <img class={classes} loading={"lazy"} {src} alt={""} />
        },
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
    Position(Position),
    Stats(StatType),
    Tower,
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

    pub fn url(&self) -> String {
        let path = match self {
            ImageType::Ability(champion_id, kind) => {
                let char = kind.as_char();
                format!("abilities/{champion_id:?}{char}.avif")
            }
            ImageType::Champion(champion_id) => format!("champions/{champion_id:?}.avif"),
            ImageType::Centered(champion_id) => format!("centered/{champion_id:?}_0.avif"),
            ImageType::Item(item_id) => {
                let riot_id = item_id.to_riot_id();
                format!("items/{riot_id}.avif")
            }
            ImageType::Rune(rune_id) => {
                let riot_id = rune_id.to_riot_id();
                format!("runes/{riot_id}.avif")
            }
            ImageType::Tower => "other/tower.avif".into(),
            ImageType::BasicAttack => "other/basic_attack.png".into(),
            ImageType::CritStrike => "stats/crit_chance.svg".into(),
            ImageType::OnhitAttack => "stats/onhit.svg".into(),
            ImageType::Level => "stats/level.svg".into(),
            ImageType::Position(pos) => match pos {
                Position::Top => "other/Top.svg",
                Position::Jungle => "other/Jungle.svg",
                Position::Middle => "other/Middle.svg",
                Position::Bottom => "other/Bottom.svg",
                Position::Support => "other/Support.svg",
            }
            .into(),
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

macro_rules! impl_conv_image_type {
    ($($ty:ty => $field:ident),+) => {
        pastey::paste! {
            $(
                impl From<&$ty> for ImageType {
                    fn from(value: &$ty) -> Self {
                        ImageType::$field(*value)
                    }
                }

                impl From<$ty> for ImageType {
                    fn from(value: $ty) -> Self {
                        (&value).into()
                    }
                }
            )+
        }
    };
}

impl_conv_image_type! {
    ChampionId => Champion,
    ItemId => Item,
    RuneId => Rune,
    StatType => Stats
}
