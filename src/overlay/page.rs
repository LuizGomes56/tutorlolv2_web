use crate::{
    components::{
        image::Image,
        tables::{body::TableBody, header::TableHeader},
    },
    overlay::{Enemy, Game, glue::get_data},
    utils::EnumCast,
};
use std::time::Duration;
use yew::{
    platform::{spawn_local, time::sleep},
    prelude::*,
};

#[component]
pub fn Overlay() -> Html {
    let game_data = use_state(|| Err::<Game, String>("Awaiting game start...".into()));

    {
        let game_data = game_data.clone();
        use_effect_with((), |_| {
            spawn_local(async move {
                loop {
                    game_data.set(get_data().await);
                    sleep(Duration::from_millis(1000)).await;
                }
            });
        });
    }

    html! {
        <div>
        {match *game_data {
            Ok(ref game) => {
                let Game {
                    current_player,
                    enemies,
                    scoreboard,
                    abilities_meta,
                    items_meta,
                    runes_meta,
                    siml_meta,
                    abilities_to_merge,
                    game_time,
                    ability_levels,
                    dragons
                } = game;

                html! {
                    <div class={classes!("ml-[400px]")}>
                        <table class={classes!("border-spacing-0", "p-0")}>
                            <TableHeader
                                champion_id={current_player.champion_id}
                                abilities_to_merge={abilities_to_merge.clone()}
                                abilities_meta={abilities_meta.clone()}
                                items_meta={items_meta.clone()}
                                runes_meta={runes_meta.clone()}
                            />
                            <TableBody<Enemy>
                                enemies={enemies}
                                abilities_to_merge={abilities_to_merge.clone()}
                                abilities_meta={abilities_meta.clone()}
                                items_meta={items_meta.clone()}
                                runes_meta={runes_meta.clone()}
                            />
                        </table>
                    </div>
                }
            }
            Err(ref e) => html!(<div>{e}</div>)
        }}
        </div>
    }
}
