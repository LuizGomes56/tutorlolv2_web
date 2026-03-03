use crate::{
    components::{
        image::{Image, ImageType},
        selector::Selector,
    },
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
            <div class={classes!("flex", "items-center", "gap-4")}>
                <Image
                    class={classes!("w-12", "h-12")}
                    src={ImageType::from(*item)}
                />
                <h3 class={classes!("text-std-200", "text-3xl", "font-medium")}>
                    {item.name()}
                </h3>
            </div>
            <Section text={"Source code definition"} />
            <Code range={item.formula()} />
            <Section text={"Damaging function definiton"} />
            <Code range={item.closure()} />
            <Section text={"Item generator implementation"} />
            <Code range={item.generator()} />
        </div>
    }
}
