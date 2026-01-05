use crate::{components::image::Image, utils::EnumCast};
use tutorlolv2_gen::ChampionId;
use yew::prelude::*;

#[component]
pub fn ChampionDocs() -> Html {
    let champion = use_state(ChampionId::random);

    html! {
        <div class={classes!("flex", "flex-col", "gap-4")}>
            <h1 class={classes!("text-4xl", "font-bold")}>{"Champions"}</h1>
            <div class={classes!("flex", "items-center", "gap-4")}>
                <Image src={champion.image_type()} class={classes!("w-12")} />
                <h2 class={classes!("text-3xl", "font-bold")}>{champion.name()}</h2>
            </div>
            <code>{champion.html()}</code>
        </div>
    }
}
