use tutorlolv2_gen::ChampionId;
use yew::prelude::*;

use crate::utils::EnumCast;

#[function_component(Documentation)]
pub fn documentation() -> Html {
    html! {
        <section>
            <h1>{"Documentation"}</h1>
            <div class={classes!("flex", "flex-col", "gap-4")}>
                <span>{"Aatrox"}</span>
                {ChampionId::Aatrox.docs()}
            </div>
        </section>
    }
}
