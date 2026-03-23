use crate::{
    components::selector::Selector,
    formulas::components::{Section, code::Code},
    utils::{EnumCast, use_setter},
};
use tutorlolv2_gen::{CastId, RuneId};
use yew::prelude::*;

#[component]
pub fn RuneFormulas() -> Html {
    let rune = use_state(RuneId::random);
    let callback = use_setter(&rune);

    html! {
        <div class={classes!("flex", "flex-col", "gap-6", "p-6", "box")}>
            <Section text={"Runes"} />
            <p class={classes!("text-std-400")}>
                {concat!(
                    "Documentation for the internal source code being used to evaluate ",
                    "the damage of a given rune, if any. Unlike items and champion's abilities, ",
                    "the damage formula of all runes are manually defined and may be outdated"
                )}
            </p>
            <Selector<RuneId>
                value={*rune}
                {callback}
            />
            <Section text={"Source code definition"} />
            <Code range={rune.formula()} />
            <Section text={"Function definition"} />
            <Code range={rune.closure()} />
        </div>
    }
}
