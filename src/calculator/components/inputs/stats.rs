use crate::{
    components::image::{Image, ImageType},
    impl_index,
    model::{EnemyStats, PlayerStats, StatType},
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
    let StatCellProps {
        image_type,
        disabled,
        ref name,
        value,
        ref oninput,
        placeholder,
    } = *props;
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

impl_index! {
    EnemyStats[StatType] i32 {
        StatType::Armor => armor,
        StatType::CurrentHealth => health,
        StatType::MagicResist => magic_resist,
        StatType::Health => max_health,
        StatType::MissingHealth => missing_health
    }
}

#[derive(PartialEq, Properties)]
pub struct StatsProps<T: PartialEq> {
    pub infer: bool,
    pub stats: T,
    pub callback: Callback<*const T>,
}

pub trait StatDisplay
where
    Self: Copy
        + PartialEq
        + Index<StatType, Output = i32>
        + IndexMut<StatType, Output = i32>
        + Clone
        + 'static,
{
    const VALUES: &[StatType];
}

impl StatDisplay for PlayerStats {
    const VALUES: &[StatType] = &[
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
    ];
}

impl StatDisplay for EnemyStats {
    const VALUES: &[StatType] = &[
        StatType::Health,
        StatType::CurrentHealth,
        StatType::MissingHealth,
        StatType::Armor,
        StatType::MagicResist,
    ];
}

#[component]
pub fn Stats<T: StatDisplay>(props: &StatsProps<T>) -> Html {
    let infer = props.infer;
    let stats = props.stats;
    let callback = &props.callback;

    T::VALUES
        .into_iter()
        .map(|&stat| {
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
