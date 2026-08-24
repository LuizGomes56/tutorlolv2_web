use {
    crate::{
        calculator::{
            Player, PlayerData,
            components::inputs::item_selector::{Search, make_row},
            page::{PlayerProps, TargetEntity},
            reducer::{LastAction, PlayerAction},
        },
        components::image::{Image, ImageType},
        utils::{
            hooks::{on_keydown, use_clickout},
            tray::{Tray, TrayAction},
        },
    },
    tutorlolv2::RuneId,
    web_sys::HtmlInputElement,
    yew::prelude::*,
};

#[derive(PartialEq, Properties)]
pub struct RuneSelectorProps {
    pub player_props: PlayerProps,
    pub is_open: UseStateHandle<bool>,
}

#[component]
pub fn RuneSelector(props: &RuneSelectorProps) -> Html {
    let RuneSelectorProps {
        player_props: PlayerProps {
            player,
            last_action,
        },
        is_open,
    } = props;

    let entity = use_state(|| TargetEntity::Player);
    let query = use_state(String::new);
    let insert = use_callback(player.clone(), |rune_id, player| {
        player.dispatch(PlayerAction::Tray(TrayAction::Insert(rune_id)))
    });
    let dropdown_ref = {
        let is_open = is_open.clone();
        use_clickout(Callback::from(move |_| is_open.set(false)), [])
    };

    let oninput = {
        let query = query.clone();
        use_callback((), move |e: InputEvent, _| {
            let value = e.target_unchecked_into::<HtmlInputElement>().value();
            query.set(value);
        })
    };

    {
        let is_open = is_open.clone();
        use_effect_with((), move |_| on_keydown(27, move || is_open.set(false)));
    }

    let options = RuneId::VALUES
        .iter()
        .copied()
        .filter(|item| {
            query.is_empty()
                || item
                    .name()
                    .to_ascii_lowercase()
                    .contains(query.to_ascii_lowercase().as_str())
        })
        .map(|rune| {
            let insert = insert.clone();
            let onclick = Callback::from(move |_| insert.emit(rune));

            html! {
                <button {onclick} class={classes!("flex", "flex-col", "gap-1", "w-fit")}>
                    <Image
                        src={ImageType::from(rune)}
                        class={classes!("w-9", "h-9", "border-2", "border-std-700")}
                    />
                </button>
            }
        })
        .collect::<Html>();

    let tray = {
        let player_tray = {
            let Player {
                ref runes,
                data: PlayerData { champion_id, .. },
                ..
            } = **player;

            let player_remove = {
                let last_action = last_action.clone();
                let player = player.clone();

                Callback::from(move |id| {
                    last_action.replace(LastAction::CurrentPlayer);
                    player.dispatch(PlayerAction::Tray(TrayAction::RemoveById(id)));
                })
            };

            let player_replace = {
                let last_action = last_action.clone();
                let player = player.clone();

                Callback::from(move |v: Tray<RuneId>| {
                    last_action.replace(LastAction::CurrentPlayer);
                    player.dispatch(PlayerAction::Tray(TrayAction::Replace(v)));
                })
            };

            make_row(
                &entity,
                TargetEntity::Player,
                champion_id,
                runes,
                player_remove,
                player_replace,
            )
        };

        html! {
            <div class={classes!("flex", "flex-col", "gap-4", "overflow-auto")}>
                {player_tray}
            </div>
        }
    };

    html! {
        <div class={classes!(
            if **is_open { "flex" } else { "hidden" },
            "fixed", "inset-0", "z-50",
            "items-center", "justify-center",
            "bg-black/70",
        )}>
            <div
                ref={dropdown_ref}
                class={classes!(
                    "grid", "grid-cols-[1fr_auto]",
                    "bg-std-950", "w-full", "max-w-7xl",
                    "h-[80vh]", "overflow-hidden"
                )}
            >
                <div class={classes!(
                    "flex", "flex-col", "gap-4",
                    "w-full", "p-4",
                    "h-full", "min-h-0"
                )}>
                    <Search {oninput} />
                    <div class={classes!(
                        "grid", "grid-cols-[auto_1fr]",
                        "gap-4", "flex-1", "min-h-0"
                    )}>
                        <div class={classes!(
                            "flex", "flex-wrap", "gap-4",
                            "content-start",
                            "min-h-0", "h-full",
                            "overflow-y-auto"
                        )}>
                            {options}
                        </div>
                    </div>
                </div>
                {tray}
            </div>
        </div>
    }
}
