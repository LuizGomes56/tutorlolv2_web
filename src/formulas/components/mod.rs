pub mod champions;
pub mod code;

use crate::{
    components::{h2::H2, image::ImageType, selector::Selector},
    formulas::components::code::Code,
    utils::{EnumCast, use_setter},
};
use tutorlolv2_gen::{AttackType, DamageIndex, EntityId, ValueId};
use yew::prelude::*;

#[component]
pub fn ValueFormulas<T: ValueId + EnumCast>() -> Html
where
    ImageType: From<T>,
{
    let value = use_state(T::random);
    let callback = use_setter(&value);

    html! {
        <div class={classes!("flex", "flex-col", "gap-6", "p-6", "box")}>
            <H2 text={match value.entity() {
                EntityId::Item(_) => {
                    "Items"
                },
                EntityId::Rune(_) => {
                    "Runes"
                },
                _ => panic!("Can't generate formulas for champions in this component")
            }} />
            <p class={classes!("text-std-400")}>
                {concat!(
                    "Documentation for the internal source code being used to evaluate ",
                    "its damage"
                )}
            </p>
            <Selector<T>
                value={*value}
                {callback}
            />
            <H2 text={"Source code definition"} />
            <Code range={value.formula()} />
            <H2 text={"Function definition"} />
            {for [AttackType::Melee, AttackType::Ranged].into_iter().map(|attack_type| {
                [DamageIndex::Min, DamageIndex::Max].into_iter().filter_map(|damage_index| {
                    let range = &value.functions()[attack_type as usize][damage_index as usize];
                    (range.len() > 0).then_some(html!(<Code {range} />))
                })
                .collect::<Html>()
            })}
            <H2 text={"Item generator implementation"} />
            <Code range={value.generator()} />
        </div>
    }
}
