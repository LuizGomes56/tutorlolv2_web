use crate::{
    components::image::{Image, ImageType},
    impl_index,
    model::{EnemyStats, PlayerStats, StatType},
    utils::StatHolder,
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
                {oninput}
            />
        </>
    }
}

impl_index! {
    @PlayerStats[StatType] i32 {
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
        MaxHealth,
        MaxMana,
        CurrentMana,
    }
}

impl_index! {
    @EnemyStats[StatType] i32 {
        Armor,
        CurrentHealth,
        MagicResist,
        MaxHealth,
        MissingHealth
    }
}

#[derive(PartialEq, Properties)]
pub struct StatsProps<T: StatHolder> {
    pub infer: bool,
    pub stats: T,
    pub callback: Callback<T::Action>,
}

pub trait StatDisplay
where
    Self: Index<StatType, Output = i32> + IndexMut<StatType, Output = i32> + StatHolder,
{
    const VALUES: &[StatType];
    fn prototype(value: StatType) -> fn(i32) -> Self::Action;
}

macro_rules! impl_stat_display {
    ($type:ty { $($stat:ident),+$(,)? }) => {
        impl StatDisplay for $type {
            const VALUES: &[StatType] = &[$(StatType::$stat),+];
            fn prototype(value: StatType) -> fn(i32) -> Self::Action {
                match value {
                    $(StatType::$stat => Self::Action::$stat,)+
                    _ => unreachable!(),
                }
            }
        }
    };
}

impl_stat_display!(PlayerStats {
    AbilityPower,
    AttackDamage,
    MaxHealth,
    CurrentHealth,
    Armor,
    ArmorPenetrationFlat,
    ArmorPenetrationPercent,
    MagicResist,
    MagicPenetrationFlat,
    MagicPenetrationPercent,
    CritChance,
    CritDamage,
    MaxMana,
    CurrentMana,
    AttackRange,
    AttackSpeed,
});

impl_stat_display!(EnemyStats {
    MaxHealth,
    CurrentHealth,
    MissingHealth,
    Armor,
    MagicResist,
});

#[component]
pub fn Stats<T: StatDisplay>(props: &StatsProps<T>) -> Html {
    let infer = props.infer;
    let stats = props.stats;
    let callback = &props.callback;

    T::VALUES
        .iter()
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
                            callback.emit(T::prototype(stat)(number));
                        })
                    }}
                />
            }
        })
        .collect::<Html>()
}
