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
        Some(h) => html! {
            <div class={classes!("relative", class)}>
                <img loading={"lazy"} {src} alt={""} />
                {h}
            </div>
        },
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
                let name = ability_id.ability_name();
                let label = match name {
                    AbilityName::Mega => Some(("MEGA", "")),
                    AbilityName::Void => None,
                    name if name < AbilityName::Mega => {
                        let index = name as u8;

                        let number = AbilityName::from_repr(index % AbilityName::JMP)
                            .unwrap()
                            .short();

                        let suffix = match index / AbilityName::JMP {
                            1 => "MIN",
                            2 => "MAX",
                            _ => "",
                        };

                        Some(if number.is_empty() {
                            (suffix, "")
                        } else {
                            (number, suffix)
                        })
                    }
                    _ => None,
                };

                Some(html! {
                    <div class={classes!("img-letter", "text-sm")}>
                        <span>{char}</span>
                        {label.map(|(first, suffix)| html! {
                            <sub>
                                {first}
                                if !suffix.is_empty() {
                                    <sub>{suffix}</sub>
                                }
                            </sub>
                        })}
                    </div>
                })
            }
            _ => None,
        }
    }

    pub fn url(&self) -> AttrValue {
        let path = match self {
            ImageType::Ability(champion_id, kind) => {
                let ability_id = kind.ability_id();
                let char = ability_id.as_char();
                let suffix = if ability_id.ability_name() == AbilityName::Mega {
                    "Mega"
                } else {
                    ""
                };

                return format!("/abilities/{champion_id:?}{char}{suffix}.avif").into();
            }
            ImageType::Champion(champion_id) => {
                return format!("/champions/{champion_id:?}.avif").into();
            }
            ImageType::Centered(champion_id) => {
                return format!(
                    // "https://ddragon.leagueoflegends.com/cdn/img/champion/centered/{champion_id:?}_0.jpg"
                    "/centered/{champion_id:?}_0.avif"
                )
                .into();
            }
            ImageType::Item(item_id) => return format!("/items/{}.avif", item_id.riot_id()).into(),
            ImageType::Rune(rune_id) => return format!("/runes/{}.avif", rune_id.riot_id()).into(),
            ImageType::Tower => "/other/tower.avif",
            ImageType::BasicAttack => "/other/basic_attack.png",
            ImageType::Ignite => "/other/ignite.avif",
            ImageType::CritStrike => "/stats/crit_chance.svg",
            ImageType::OnhitAttack => "/stats/onhit.svg",
            ImageType::Level => "/stats/level.svg",
            ImageType::Position(pos) => match pos {
                Position::Top => "/other/Top.svg",
                Position::Jungle => "/other/Jungle.svg",
                Position::Middle => "/other/Middle.svg",
                Position::Bottom => "/other/Bottom.svg",
                Position::Support => "/other/Support.svg",
            },
            ImageType::Stats(stat) => match stat {
                PlayerStatsField::AbilityPower => "/stats/ability_power.svg",
                PlayerStatsField::Armor => "/stats/armor.svg",
                PlayerStatsField::ArmorPenetrationFlat
                | PlayerStatsField::ArmorPenetrationPercent => "/stats/armor_penetration.svg",
                PlayerStatsField::AttackDamage => "/stats/attack_damage.svg",
                PlayerStatsField::AttackSpeed => "/stats/attack_speed.svg",
                PlayerStatsField::CritChance => "/stats/crit_chance.svg",
                PlayerStatsField::CritDamage => "/stats/crit_damage.svg",
                PlayerStatsField::CurrentHealth | PlayerStatsField::MaxHealth => {
                    "/stats/health.svg"
                }
                PlayerStatsField::CurrentMana | PlayerStatsField::MaxMana => "/stats/mana.svg",
                PlayerStatsField::MagicPenetrationFlat
                | PlayerStatsField::MagicPenetrationPercent => "/stats/magic_penetration.svg",
                PlayerStatsField::MagicResist => "/stats/magic_resist.svg",
            },
            ImageType::StatsFilter(stat) => match stat {
                StatName::AbilityHaste => "/stats/ability_haste.svg",
                StatName::AbilityPower => "/stats/ability_power.svg",
                StatName::AdaptiveForce => "/stats/adaptive_force.svg",
                StatName::Armor => "/stats/armor.svg",
                StatName::ArmorPenetration | StatName::Lethality => "/stats/armor_penetration.svg",
                StatName::AttackDamage => "/stats/attack_damage.svg",
                StatName::AttackSpeed => "/stats/attack_speed.svg",
                StatName::BaseHealthRegen => "/stats/health_regeneration.svg",
                StatName::BaseManaRegen => "/stats/mana_regeneration.svg",
                StatName::CritChance => "/stats/crit_chance.svg",
                StatName::CritDamage => "/stats/crit_damage.svg",
                StatName::GoldPer10Seconds => "/stats/gold.svg",
                StatName::HealAndShieldPower => "/stats/heal_and_shield_power.svg",
                StatName::Health => "/stats/health.svg",
                StatName::LifeSteal => "/stats/life_steal.svg",
                StatName::MagicPenetration | StatName::MagicPenetrationPercent => {
                    "/stats/magic_penetration.svg"
                }
                StatName::MagicResist => "/stats/magic_resist.svg",
                StatName::Mana => "/stats/mana.svg",
                StatName::MoveSpeed | StatName::MoveSpeedPercent => "/stats/move_speed.svg",
                StatName::Omnivamp => "/stats/omnivamp.svg",
                StatName::Tenacity => "/stats/tenacity.svg",
            },
            ImageType::Other(other) => match other {
                OtherImage::Voidgrubs => "/other/voidgrubs.avif",
                OtherImage::Atakhan => "/other/atakhan.avif",
                OtherImage::Baron => "/other/baron.avif",
                OtherImage::Dragon(dragon) => match dragon {
                    DragonImage::Earth => "/other/earth_dragon.avif",
                    DragonImage::Elder => "/other/elder_dragon.avif",
                    DragonImage::Fire => "/other/fire_dragon.avif",
                    DragonImage::Ocean => "/other/ocean_dragon.avif",
                    DragonImage::Chemtech => "/other/chemtech_dragon.avif",
                },
                OtherImage::Monster(monster) => match monster {
                    MonsterImage::Gromp => "/other/gromp.avif",
                    MonsterImage::Wolves => "/other/wolves.avif",
                    MonsterImage::Red => "/other/red_buff.avif",
                    MonsterImage::Blue => "/other/blue_buff.avif",
                    MonsterImage::Krug => "/other/krug.avif",
                    MonsterImage::Raptor => "/other/raptor.avif",
                },
                OtherImage::Minion(minion) => match minion {
                    MinionImage::Melee => "/other/melee_minion.avif",
                    MinionImage::Ranged => "/other/ranged_minion.avif",
                    MinionImage::Cannon => "/other/cannon.avif",
                    MinionImage::Super => "/other/super_minion.avif",
                },
            },
        };

        path.into()
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

    html! {
        <span
            class={classes!("inline-block", "bg-current", "shrink-0", class)}
            style={format!(
                concat!(
                    "-webkit-mask-image:url('{0}');",
                    "-webkit-mask-repeat:no-repeat;",
                    "-webkit-mask-position:center;",
                    "-webkit-mask-size:contain;",
                    "mask-image:url('{0}');",
                    "mask-repeat:no-repeat;",
                    "mask-position:center;",
                    "mask-size:contain;"
                ),
                src
            )}
        />
    }
}

#[cfg(test)]
mod header_tests {
    use super::*;
    use tutorlolv2::AbilityId;

    fn markup(node: &Html) -> String {
        use yew::virtual_dom::VNode;
        match node {
            VNode::VTag(tag) => format!(
                "<{}>{}</{}>",
                tag.tag(),
                tag.children().map(markup).unwrap_or_default(),
                tag.tag()
            ),
            VNode::VText(text) => text.text.to_string(),
            VNode::VList(list) => list.iter().map(markup).collect(),
            _ => panic!("Unexpected header node"),
        }
    }

    #[test]
    fn ability_headers_preserve_numbered_nested_and_special_labels() {
        for (name, label) in [
            (AbilityName::Void, ""),
            (AbilityName::_1, "<sub>1</sub>"),
            (AbilityName::_8, "<sub>8</sub>"),
            (AbilityName::Min, "<sub>MIN</sub>"),
            (AbilityName::Max, "<sub>MAX</sub>"),
            (AbilityName::_1Min, "<sub>1<sub>MIN</sub></sub>"),
            (AbilityName::_8Min, "<sub>8<sub>MIN</sub></sub>"),
            (AbilityName::_1Max, "<sub>1<sub>MAX</sub></sub>"),
            (AbilityName::_8Max, "<sub>8<sub>MAX</sub></sub>"),
            (AbilityName::Mega, "<sub>MEGA</sub>"),
            (AbilityName::Minion, ""),
            (AbilityName::MonsterMax, ""),
        ] {
            let image = ImageType::Ability(ChampionId::default(), AbilityId::Q(name).into());
            assert_eq!(
                markup(&image.header().unwrap()),
                format!("<div><span>Q</span>{label}</div>"),
                "{name:?}"
            );
        }
    }
}
