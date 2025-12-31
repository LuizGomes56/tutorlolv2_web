use crate::{
    components::image::Image,
    utils::{EnumCast, get_cache},
};
use tutorlolv2_gen::{
    ABILITY_FORMULAS, CHAMPION_GENERATOR, ChampionId, ITEM_GENERATOR, ItemId, RuneId,
};
use yew::prelude::*;

#[derive(Clone, Copy, Default)]
enum View {
    #[default]
    Champion,
    Item,
    Rune,
}

#[component]
pub fn Documentation() -> Html {
    let champion = use_state(ChampionId::random);
    let item = use_state(ItemId::random);
    let rune = use_state(RuneId::random);
    // let view = use_state(View::default);

    // let number_of_champions = ChampionId::VARIANTS;
    // let number_of_items = ItemId::VARIANTS;
    // let number_of_runes = RuneId::VARIANTS;

    // let new_champion = |champion_id: ChampionId| champion.set(champion_id);
    // let new_item = |item_id: ItemId| item.set(item_id);
    // let new_rune = |rune_id: RuneId| rune.set(rune_id);

    let champion_documentation = champion.html();
    let item_documentation = item.html();
    let rune_documentation = rune.html();

    // let champion_name = champion.name();
    // let item_name = item.name();
    // let rune_name = rune.name();

    // let champion_image_type = champion.image_type();
    // let item_image_type = item.image_type();
    // let rune_image_type = rune.image_type();

    // let champion_image = html! { <Image src={champion_image_type} /> };
    // let item_image = html! { <Image src={item_image_type} /> };
    // let rune_image = html! { <Image src={rune_image_type} /> };

    // let champion_generator = get_cache(CHAMPION_GENERATOR[champion.index()]);
    // let item_generator = get_cache(ITEM_GENERATOR[item.index()]);

    // {
    //     let abilities = ABILITY_FORMULAS[champion.index()];
    //     for i in 0..abilities.len() {
    //         let champion_abilities = get_cache(abilities[i]);
    //     }
    // }

    html! {
        <div>
            <span>{champion_documentation}</span>
            <span>{item_documentation}</span>
            <span>{rune_documentation}</span>
        </div>
    }
}
