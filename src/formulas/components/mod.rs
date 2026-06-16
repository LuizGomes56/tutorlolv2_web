pub mod champions;
pub mod code;

use crate::{
    components::{h2::H2, image::ImageType, selector::Selector},
    formulas::components::code::Code,
    utils::{EnumCast, use_setter},
};
use std::collections::HashSet;
use tutorlolv2::{EntityId, ValueId};
use yew::prelude::*;

#[component]
pub fn ValueFormulas<T: ValueId + EnumCast>() -> Html
where
    ImageType: From<T>,
{
    let value = use_state(T::random);
    let callback = use_setter(&value);

    let mut functions = HashSet::with_capacity(4);

    for range in value.functions().iter().flatten() {
        if range.len() > 0 {
            functions.insert(range);
        }
    }

    html! {
        <div class={classes!("flex", "flex-col", "gap-6", "p-6", "box")}>
            <H2 text={match value.entity() {
                EntityId::Item(_) => "Items",
                EntityId::Rune(_) => "Runes",
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
            {(!functions.is_empty()).then_some(html!(
                <>
                    <H2 text={"Function definition"} />
                    for range in functions {
                        <Code range={range} />
                    }
                </>
            ))}
            {(value.generator().len() > 0).then_some(html!(
                <>
                    <H2 text={"Implementation"} />
                    <Code range={value.generator()} />
                </>
            ))}
        </div>
    }
}
