use crate::{
    components::image::{Image, ImageType},
    impl_index,
    model::{PlayerStats, StatType},
};
use std::ops::{Index, IndexMut};
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct StatCellProps {
    pub image_type: ImageType,
    pub name: AttrValue,
    pub disabled: bool,
    pub value: i32,
    pub oninput: Callback<InputEvent>,
    pub placeholder: u8,
}

#[component]
pub fn StatCell(props: &StatCellProps) -> Html {
    let image_type = props.image_type;
    let disabled = props.disabled;
    let name = &props.name;
    let value = &props.value;
    let oninput = &props.oninput;
    let placeholder = props.placeholder;
    html! {
        <>
            <span class={classes!("flex", "items-center", "justify-center", "relative")}>
                <Image
                    class={classes!("h-3.5", "w-3.5")}
                    src={image_type}
                />
            </span>
            <span class={classes!("text-sm", "content-center", "whitespace-nowrap")}>
                {name}
            </span>
            <input
                type={"number"}
                class={classes!(
                    "text-center", "min-w-0", "ml-2", "bg-transparent",
                    if disabled { "text-std-400" }
                    else { "text-white" }
                )}
                {disabled}
                placeholder={placeholder.to_string()}
                value={value.to_string()}
                oninput={oninput}
            />
        </>
    }
}

impl_index! {
    PlayerStats[StatType] i32 {
        StatType::AbilityPower => ability_power,
        StatType::Armor => armor,
        StatType::ArmorPenetrationFlat => armor_penetration_flat,
        StatType::ArmorPenetrationPercent => armor_penetration_percent,
        StatType::AttackDamage => attack_damage,
        StatType::AttackRange => attack_range,
        StatType::AttackSpeed => attack_speed,
        StatType::CritChance => crit_chance,
        StatType::CritDamage => crit_damage,
        StatType::CurrentHealth => current_health,
        StatType::MagicPenetrationFlat => magic_penetration_flat,
        StatType::MagicPenetrationPercent => magic_penetration_percent,
        StatType::MagicResist => magic_resist,
        StatType::Health => health,
        StatType::Mana => mana,
        StatType::CurrentMana => current_mana,
    }
}

#[derive(PartialEq, Properties)]
pub struct StatsProps {
    pub infer: bool,
    pub stats: PlayerStats,
    pub callback: Callback<*const PlayerStats>,
}

#[component]
pub fn Stats(props: &StatsProps) -> Html {
    let infer = props.infer;
    let stats = props.stats;
    let callback = &props.callback;

    [
        StatType::AbilityPower,
        StatType::AttackDamage,
        StatType::Health,
        StatType::CurrentHealth,
        StatType::Armor,
        StatType::ArmorPenetrationFlat,
        StatType::ArmorPenetrationPercent,
        StatType::MagicResist,
        StatType::MagicPenetrationFlat,
        StatType::MagicPenetrationPercent,
        StatType::CritChance,
        StatType::CritDamage,
        StatType::Mana,
        StatType::CurrentMana,
        StatType::AttackRange,
        StatType::AttackSpeed,
    ]
    .into_iter()
    .map(|stat| {
        html! {
            <StatCell
                image_type={ImageType::Stats(stat)}
                name={stat.to_string()}
                disabled={infer}
                placeholder={0}
                value={stats[stat]}
                oninput={{
                    let callback = callback.clone();
                    Callback::from(move |e: InputEvent| {
                        let value = e.target_unchecked_into::<HtmlInputElement>().value();
                        let number = value.parse().unwrap_or(0);
                        let mut result = stats;
                        result[stat] = number;
                        callback.emit(&result as _);
                    })
                }}
            />
        }
    })
    .collect::<Html>()
}
