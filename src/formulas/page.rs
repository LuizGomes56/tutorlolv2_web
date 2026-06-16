use crate::formulas::components::{ValueFormulas, champions::ChampionFormulas};
use tutorlolv2::{ItemId, RuneId};
use yew::prelude::*;

#[component]
pub fn Formulas() -> Html {
    html! {
        <div class={classes!("w-full", "p-4", "flex", "flex-col", "gap-4")}>
            <ChampionFormulas />
            <ValueFormulas<ItemId> />
            <ValueFormulas<RuneId> />
        </div>
    }
}
