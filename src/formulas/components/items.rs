use crate::{
    components::selector::Selector,
    formulas::components::{Section, code::Code},
    utils::{EnumCast, use_setter},
};
use tutorlolv2_gen::{CastId, ItemId};
use yew::prelude::*;

#[component]
pub fn ItemFormulas() -> Html {
    let item = use_state(ItemId::random);
    let callback = use_setter(&item);

    html! {
        <div class={classes!("flex", "flex-col", "gap-6", "py-4", "px-6", "box")}>
            <Section text={"Items"} />
            <p class={classes!("text-std-400")}>
                {concat!(
                    "Documentation for the internal source code being used to evaluate ",
                    "the damage of a given item, and its bonus stats"
                )}
            </p>
            <Selector<ItemId>
                value={*item}
                {callback}
            />
            <Section text={"Source code definition"} />
            <Code range={item.formula()} />
            <Section text={"Function definition"} />
            <Code range={item.closure()} />
            <Section text={"Item generator implementation"} />
            <Code range={item.generator()} />
        </div>
    }
}
