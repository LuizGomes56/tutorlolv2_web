use crate::{
    components::{
        image::{Image, ImageType},
        selector::Selector,
    },
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
        <div class={classes!("flex", "flex-col", "gap-6", "py-4", "px-6", "box")}>
            <Section text={"Runes"} />
            <p class={classes!("text-std-400")}>
                {concat!(
                    "Documentation for the internal source code being used to evaluate ",
                    "the damage of a given rune, if any. Unlike items and champion's abilities, ",
                    "the damage formula of all runes are manually defined and may be outdated"
                )}
            </p>
            <div class={classes!("flex", "items-center", "gap-4")}>
                <Image
                    class={classes!("w-12", "h-12")}
                    src={ImageType::from(*rune)}
                />
                <h3 class={classes!("text-std-200", "text-3xl", "font-medium")}>
                    {rune.name()}
                </h3>
            </div>
            <Section text={"Source code definition"} />
            <Code range={rune.formula()} />
            <Section text={"Damaging function definition"} />
            <Code range={rune.closure()} />
        </div>
    }
}
