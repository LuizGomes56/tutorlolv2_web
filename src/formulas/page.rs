use crate::formulas::components::{
    champions::ChampionFormulas, items::ItemFormulas, runes::RuneFormulas,
};
use yew::prelude::*;

#[component]
pub fn Formulas() -> Html {
    html! {
        <div class={classes!("w-full", "p-4", "flex", "flex-col", "gap-4")}>
            <ChampionFormulas />
            <ItemFormulas />
            <RuneFormulas />
        </div>
    }
}
