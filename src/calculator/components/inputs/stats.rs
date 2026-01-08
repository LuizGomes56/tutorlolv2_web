use crate::{
    components::image::Image,
    model::PlayerStats,
    utils::{ImageType, StatType},
};
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
                    "text-center", "min-w-0", "bg-transparent",
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

impl PlayerStats {
    pub const fn field(&mut self, stat: StatType) -> &mut i32 {
        match stat {
            StatType::AbilityPower => &mut self.ability_power,
            StatType::Armor => &mut self.armor,
            StatType::ArmorPenetrationFlat => &mut self.armor_penetration_flat,
            StatType::ArmorPenetrationPercent => &mut self.armor_penetration_percent,
            StatType::AttackDamage => &mut self.attack_damage,
            StatType::AttackRange => &mut self.attack_range,
            StatType::AttackSpeed => &mut self.attack_speed,
            StatType::CritChance => &mut self.crit_chance,
            StatType::CritDamage => &mut self.crit_damage,
            StatType::CurrentHealth => &mut self.current_health,
            StatType::MagicPenetrationFlat => &mut self.magic_penetration_flat,
            StatType::MagicPenetrationPercent => &mut self.magic_penetration_percent,
            StatType::MagicResist => &mut self.magic_resist,
            StatType::Health => &mut self.health,
            StatType::Mana => &mut self.mana,
            StatType::CurrentMana => &mut self.current_mana,
        }
    }

    pub const fn get(&mut self, stat: StatType) -> i32 {
        *self.field(stat)
    }

    pub const fn set(mut self, stat: StatType, value: i32) -> Self {
        *self.field(stat) = value;
        self
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
    let mut stats = props.stats;
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
                value={stats.get(stat)}
                oninput={{
                    let callback = callback.clone();
                    let stats = stats;
                    Callback::from(move |e: InputEvent| {
                        let value = e.target_unchecked_into::<HtmlInputElement>().value();
                        let number = value.parse().unwrap_or(0);
                        callback.emit(&stats.set(stat, number) as _);
                    })
                }}
            />
        }
    })
    .collect::<Html>()
}
