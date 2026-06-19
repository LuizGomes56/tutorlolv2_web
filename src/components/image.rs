use crate::model::AbilityKind;
use tutorlolv2::{
    AbilityName, ChampionId, ItemId, Position, RuneId, StatName, yew::stats::PlayerStatsField,
};
use yew::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DragonImage {
    Elder,
    Fire,
    Ocean,
    Earth,
    Chemtech,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MinionImage {
    Melee,
    Ranged,
    Cannon,
    Super,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MonsterImage {
    Gromp,
    Wolves,
    Red,
    Blue,
    Krug,
    Raptor,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OtherImage {
    Voidgrubs,
    Minion(MinionImage),
    Dragon(DragonImage),
    Monster(MonsterImage),
    Baron,
    Atakhan,
}

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

    match header {
        Some(h) => {
            let mut classes = classes!("relative");
            classes.push(class);
            html! {
                <div class={classes}>
                    <img loading={"lazy"} {src} alt={""} />
                    {h}
                </div>
            }
        }
        None => html! {
            <img {class} loading={"lazy"} {src} alt={""} />
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
    Stats(PlayerStatsField),
    StatsFilter(StatName),
    Other(OtherImage),
    Tower,
    Ignite,
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
                        <span>{char}</span>
                        {name.map(|name| html!(<sub>{name}</sub>))}
                    </div>
                })
            }
            _ => None,
        }
    }

    pub fn url(&self) -> String {
        let path: &dyn core::fmt::Display = match self {
            ImageType::Ability(champion_id, kind) => &{
                let ability_id = kind.ability_id();
                let char = ability_id.as_char();
                let mut result = format!("abilities/{champion_id:?}{char}");
                if ability_id.ability_name() == AbilityName::Mega {
                    result.push_str("Mega");
                }
                result.push_str(".avif");
                result
            },
            ImageType::Champion(champion_id) => &format!("champions/{champion_id:?}.avif"),
            ImageType::Centered(champion_id) => {
                // #[cfg(feature = "server")]
                // {
                return format!(
                    concat!(
                        "https://ddragon.leagueoflegends.com",
                        "/cdn/img/champion/centered",
                        "/{:?}_0.jpg"
                    ),
                    champion_id
                );
                // }

                // #[cfg(not(feature = "server"))]
                // {
                //     &format!("centered/{champion_id:?}_0.avif")
                // }
            }
            ImageType::Item(item_id) => &{
                let riot_id = item_id.to_riot_id();
                format!("items/{riot_id}.avif")
            },
            ImageType::Rune(rune_id) => &{
                let riot_id = rune_id.to_riot_id();
                format!("runes/{riot_id}.avif")
            },
            ImageType::Tower => &"other/tower.avif",
            ImageType::BasicAttack => &"other/basic_attack.png",
            ImageType::Ignite => &"other/ignite.avif",
            ImageType::CritStrike => &"stats/crit_chance.svg",
            ImageType::OnhitAttack => &"stats/onhit.svg",
            ImageType::Level => &"stats/level.svg",
            ImageType::Position(pos) => &match pos {
                Position::Top => "other/Top.svg",
                Position::Jungle => "other/Jungle.svg",
                Position::Middle => "other/Middle.svg",
                Position::Bottom => "other/Bottom.svg",
                Position::Support => "other/Support.svg",
            },
            ImageType::Stats(stat) => &match stat {
                PlayerStatsField::AbilityPower => "stats/ability_power.svg",
                PlayerStatsField::Armor => "stats/armor.svg",
                PlayerStatsField::ArmorPenetrationFlat
                | PlayerStatsField::ArmorPenetrationPercent => "stats/armor_penetration.svg",
                PlayerStatsField::AttackDamage => "stats/attack_damage.svg",
                PlayerStatsField::AttackSpeed => "stats/attack_speed.svg",
                PlayerStatsField::CritChance => "stats/crit_chance.svg",
                PlayerStatsField::CritDamage => "stats/crit_damage.svg",
                PlayerStatsField::CurrentHealth | PlayerStatsField::MaxHealth => "stats/health.svg",
                PlayerStatsField::CurrentMana | PlayerStatsField::MaxMana => "stats/mana.svg",
                PlayerStatsField::MagicPenetrationFlat
                | PlayerStatsField::MagicPenetrationPercent => "stats/magic_penetration.svg",
                PlayerStatsField::MagicResist => "stats/magic_resist.svg",
            },
            ImageType::StatsFilter(stat) => &match stat {
                StatName::AbilityHaste => "stats/ability_haste.svg",
                StatName::AbilityPower => "stats/ability_power.svg",
                StatName::AdaptiveForce => "stats/adaptive_force.svg",
                StatName::Armor => "stats/armor.svg",
                StatName::ArmorPenetration | StatName::Lethality => "stats/armor_penetration.svg",
                StatName::AttackDamage => "stats/attack_damage.svg",
                StatName::AttackSpeed => "stats/attack_speed.svg",
                StatName::BaseHealthRegen => "stats/health_regeneration.svg",
                StatName::BaseManaRegen => "stats/mana_regeneration.svg",
                StatName::CritChance => "stats/crit_chance.svg",
                StatName::CritDamage => "stats/crit_damage.svg",
                StatName::GoldPer10Seconds => "stats/gold.svg",
                StatName::HealAndShieldPower => "stats/heal_and_shield_power.svg",
                StatName::Health => "stats/health.svg",
                StatName::LifeSteal => "stats/life_steal.svg",
                StatName::MagicPenetration | StatName::MagicPenetrationPercent => {
                    "stats/magic_penetration.svg"
                }
                StatName::MagicResist => "stats/magic_resist.svg",
                StatName::Mana => "stats/mana.svg",
                StatName::MoveSpeed | StatName::MoveSpeedPercent => "stats/move_speed.svg",
                StatName::Omnivamp => "stats/omnivamp.svg",
                StatName::Tenacity => "stats/tenacity.svg",
            },
            ImageType::Other(other) => &match other {
                OtherImage::Voidgrubs => "other/voidgrubs.avif",
                OtherImage::Atakhan => "other/atakhan.avif",
                OtherImage::Baron => "other/baron.avif",
                OtherImage::Dragon(dragon) => match dragon {
                    DragonImage::Earth => "other/earth_dragon.avif",
                    DragonImage::Elder => "other/elder_dragon.avif",
                    DragonImage::Fire => "other/fire_dragon.avif",
                    DragonImage::Ocean => "other/ocean_dragon.avif",
                    DragonImage::Chemtech => "other/chemtech_dragon.avif",
                },
                OtherImage::Monster(monster) => match monster {
                    MonsterImage::Gromp => "other/gromp.avif",
                    MonsterImage::Wolves => "other/wolves.avif",
                    MonsterImage::Red => "other/red_buff.avif",
                    MonsterImage::Blue => "other/blue_buff.avif",
                    MonsterImage::Krug => "other/krug.avif",
                    MonsterImage::Raptor => "other/raptor.avif",
                },
                OtherImage::Minion(minion) => match minion {
                    MinionImage::Melee => "other/melee_minion.avif",
                    MinionImage::Ranged => "other/ranged_minion.avif",
                    MinionImage::Cannon => "other/cannon.avif",
                    MinionImage::Super => "other/super_minion.avif",
                },
            },
        };

        // #[cfg(not(feature = "server"))]
        // {
        //     format!("{BASE_URL}/img/{path}")
        // }

        // #[cfg(feature = "server")]
        // {
        format!("/{path}")
        // }
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
    PlayerStatsField => Stats
}

#[derive(PartialEq, Properties)]
pub struct SvgProps {
    #[prop_or_default]
    pub class: Classes,
    pub src: AttrValue,
}

#[component]
pub fn Svg(props: &SvgProps) -> Html {
    let SvgProps { class, src } = props;

    let mut classes = classes!("inline-block", "bg-current", "shrink-0");
    classes.push(class);

    html! {
        <span
            class={classes}
            style={format!(
                concat!(
                    "-webkit-mask-image:url('{}');",
                    "-webkit-mask-repeat:no-repeat;",
                    "-webkit-mask-position:center;",
                    "-webkit-mask-size:contain;",
                    "mask-image:url('{}');",
                    "mask-repeat:no-repeat;",
                    "mask-position:center;",
                    "mask-size:contain;"
                ),
                src, src
            )}
        />
    }
}
