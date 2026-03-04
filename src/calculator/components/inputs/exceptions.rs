use crate::{
    components::image::{Image, ImageType},
    model::{AbilityKind, ValueException},
};
use tutorlolv2_gen::{AbilityId, AbilityName, ChampionId, ItemId, Key, RuneId};
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct ExceptionsProps {
    pub items: Vec<ItemId>,
    pub runes: Vec<RuneId>,
    pub item_exceptions: Vec<ValueException>,
    pub rune_exceptions: Vec<ValueException>,
    pub item_callback: Callback<(ItemId, u32)>,
    pub rune_callback: Callback<(RuneId, u32)>,
    pub stack_callback: Callback<u32>,
    pub stacks: u32,
    pub champion_id: ChampionId,
}

#[component]
pub fn Exceptions(props: &ExceptionsProps) -> Html {
    let ExceptionsProps {
        ref items,
        ref runes,
        ref item_exceptions,
        ref rune_exceptions,
        ref item_callback,
        ref rune_callback,
        ref stack_callback,
        stacks,
        champion_id,
    } = *props;

    const EXCEPTION_ITEMS: [ItemId; 12] = [
        ItemId::DarkSeal,
        ItemId::DragonheartU44,
        ItemId::DemonKingsCrownU66,
        ItemId::RiteOfRuin,
        ItemId::MejaisSoulstealer,
        ItemId::DemonKingsCrownU44,
        ItemId::Hubris6697,
        ItemId::Hubris126697,
        ItemId::HubrisArena,
        ItemId::BloodlettersCurse4010,
        ItemId::BloodlettersCurse8010,
        ItemId::BlackCleaver,
    ];

    let oninput_stacks = use_callback(
        stack_callback.clone(),
        move |e: InputEvent, stack_callback| {
            let target = e.target_unchecked_into::<HtmlInputElement>();
            let value = target.value().parse::<u32>().unwrap_or(0);
            stack_callback.emit(value);
        },
    );

    let champion_stack_selector = use_memo(
        (oninput_stacks, champion_id, stacks),
        move |(oninput, ..)| {
            let image = match champion_id {
                ChampionId::AurelionSol
                | ChampionId::Bard
                | ChampionId::Belveth
                | ChampionId::Graves
                | ChampionId::Hecarim
                | ChampionId::Kalista
                | ChampionId::Kindred
                | ChampionId::Senna
                | ChampionId::Shyvana
                | ChampionId::Sion
                | ChampionId::Smolder
                | ChampionId::Swain
                | ChampionId::Thresh
                | ChampionId::Veigar => Some(Key::P),
                ChampionId::Nasus => Some(Key::Q),
                ChampionId::Darius => Some(Key::E),
                ChampionId::Chogath => Some(Key::R),
                _ => None,
            };

            image.map(|key| {
                html! {
                    <div class={classes!("flex", "items-center", "gap-2")}>
                        <Image
                            class={classes!("w-8", "h-8")}
                            src={ImageType::Ability(champion_id, key.into())}
                        />
                        <input
                            type={"number"}
                            class={classes!(
                                "text-center", "min-w-0", "ml-2",
                                "bg-transparent", "text-white"
                            )}
                            {oninput}
                            value={stacks.to_string()}
                            placeholder={"0"}
                        />
                    </div>
                }
            })
        },
    );

    let item_exception_selector = use_memo(
        (
            items.clone(),
            item_exceptions.clone(),
            item_callback.clone(),
        ),
        |(items, item_exceptions, callback)| {
            items
                .iter()
                .filter(|item| EXCEPTION_ITEMS.contains(item))
                .filter_map(|item| {
                    item_exceptions
                        .iter()
                        .find(|v| match v.get_item_id() {
                            Some(i) => i == *item,
                            None => false,
                        })
                        .map(|v| {
                            let oninput = {
                                let callback = callback.clone();
                                let item = *item;
                                Callback::from(move |e: InputEvent| {
                                    let target = e.target_unchecked_into::<HtmlInputElement>();
                                    let value = target.value().parse::<u32>().unwrap_or(0);
                                    callback.emit((item, value));
                                })
                            };

                            html! {
                                <div class={classes!("flex", "items-center", "gap-2")}>
                                    <Image
                                        class={classes!("w-8", "h-8")}
                                        src={ImageType::from(item)}
                                    />
                                    <input
                                        type={"number"}
                                        class={classes!(
                                            "text-center", "min-w-0", "ml-2",
                                            "bg-transparent", "text-white"
                                        )}
                                        {oninput}
                                        value={v.stacks().to_string()}
                                        placeholder={"0"}
                                    />
                                </div>
                            }
                        })
                })
                .collect::<Html>()
        },
    );

    html! {
        <div>
            {(*champion_stack_selector).clone()}
            {(*item_exception_selector).clone()}
        </div>
    }
}
