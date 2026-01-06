use crate::{components::image::Image, model::Stats as StatsI32};
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct StatsProps {
    infer: bool,
    stats: StatsI32,
    callback: Callback<*const StatsI32>,
}

#[derive(Clone)]
struct StatsHook {
    callback: Callback<*const StatsI32>,
    stats: StatsI32,
}

#[hook]
fn use_stats(hook: &StatsHook, func: fn(StatsI32, i32) -> StatsI32) -> Callback<i32> {
    let StatsHook { callback, stats } = hook.clone();
    use_callback((), move |v, _| {
        // let value = target.unchecked_into::<HtmlInputElement>().value();
        // let number = value.parse().unwrap_or(0);
        callback.emit(&func(stats, v) as _);
    })
}

#[component]
pub fn Stats(props: &StatsProps) -> Html {
    let StatsProps {
        infer,
        stats,
        callback,
    } = props;

    let hook = &StatsHook {
        callback: callback.clone(),
        stats: *stats,
    };

    let ability_power = use_stats(hook, StatsI32::set_ability_power);
    let armor = use_stats(hook, StatsI32::set_armor);
    let armor_penetration_flat = use_stats(hook, StatsI32::set_armor_penetration_flat);
    let armor_penetration_percent = use_stats(hook, StatsI32::set_armor_penetration_percent);
    let attack_damage = use_stats(hook, StatsI32::set_attack_damage);
    let attack_range = use_stats(hook, StatsI32::set_attack_range);
    let attack_speed = use_stats(hook, StatsI32::set_attack_speed);
    let crit_chance = use_stats(hook, StatsI32::set_crit_chance);
    let crit_damage = use_stats(hook, StatsI32::set_crit_damage);
    let current_health = use_stats(hook, StatsI32::set_current_health);
    let magic_penetration_flat = use_stats(hook, StatsI32::set_magic_penetration_flat);
    let magic_penetration_percent = use_stats(hook, StatsI32::set_magic_penetration_percent);
    let magic_resist = use_stats(hook, StatsI32::set_magic_resist);
    let health = use_stats(hook, StatsI32::set_health);
    let mana = use_stats(hook, StatsI32::set_mana);
    let current_mana = use_stats(hook, StatsI32::set_current_mana);

    [
        (ability_power, "Ability Power"),
        (armor, "Armor"),
        (armor_penetration_flat, "Armor Penetration Flat"),
        (armor_penetration_percent, "Armor Penetration Percent"),
        (attack_damage, "Attack Damage"),
        (attack_range, "Attack Range"),
        (attack_speed, "Attack Speed"),
        (crit_chance, "Crit Chance"),
        (crit_damage, "Crit Damage"),
        (current_health, "Current Health"),
        (magic_penetration_flat, "Magic Penetration Flat"),
        (magic_penetration_percent, "Magic Penetration Percent"),
        (magic_resist, "Magic Resist"),
        (health, "Health"),
        (mana, "Mana"),
        (current_mana, "Current Mana"),
    ]
    .into_iter()
    .map(|(func, name)| {
        html! {
            <>
                // <span class={classes!("flex", "items-center", "justify-center", "relative")}>
                //     <Image
                //         class={classes!("h-3.5", "w-3.5")}
                //         source={ImageType::Other(url!("/img/stats/{}", props.path).into())}
                //     />
                // </span>
                // <span class={classes!("text-sm")}>{props.display}</span>
                // <input
                //     type={"number"}
                //     class={classes!(
                //         "text-center", "min-w-0", "bg-transparent",
                //         if props.disabled { "_text-400" }
                //         else { "text-white" }
                //     )}
                //     disabled={props.disabled}
                //     placeholder={"0"}
                //     value={props.value.to_string()}
                //     oninput={props.oninput.clone()}
                // />
            </>
        }
    })
    .collect::<Html>()
}
