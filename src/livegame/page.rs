use crate::{
    components::{
        image::{Image, ImageType},
        stack::StackSelector,
        tables::header::TableHeader,
    },
    livegame::{Enemy, Game},
    utils::{encode_offset, glue::get_data},
};
use std::time::Duration;
use tutorlolv2_gen::CastId;
use yew::{
    platform::{spawn_local, time::sleep},
    prelude::*,
};

#[component]
pub fn Livegame() -> Html {
    let game = use_state(|| Err("Loading...".into()));

    {
        let game = game.clone();
        use_effect_with((), |_| {
            spawn_local(async move {
                loop {
                    game.set(get_data().await);
                    sleep(Duration::from_millis(1000)).await;
                }
            });
        });
    };

    match &*game {
        Ok(data) => {
            let Game {
                current_player,
                enemies,
                scoreboard,
                items_meta,
                runes_meta,
                game_time,
                ability_levels,
                dragons,
            } = data;

            html! {
                <div class={classes!(
                    "flex", "flex-col", "gap-4",
                    "p-4", "overflow-hidden",
                    "flex-1"
                )}>
                    <div class={classes!("box", "overflow-auto")}>
                        <table class={classes!("data-table")}>
                            <TableHeader
                                champion_id={current_player.champion_id}
                                items_meta={items_meta.clone()}
                                runes_meta={runes_meta.clone()}
                            />
                            <tbody>
                                {
                                    enemies.iter().map(|enemy| {
                                        let damages = enemy.damages.to_html(
                                            current_player.champion_id,
                                            items_meta,
                                            runes_meta
                                        );
                                        let enemy_id = enemy.champion_id;
                                        html! {
                                            <tr>
                                                <td
                                                    class={classes!("w-12")}
                                                    data_offset={encode_offset(&[enemy_id.formula()])}
                                                >
                                                    <Image src={ImageType::from(enemy_id)} />
                                                </td>
                                                {damages}
                                            </tr>
                                        }
                                    })
                                    .collect::<Html>()
                                }
                            </tbody>
                        </table>
                    </div>
                    <div class={classes!("box", "overflow-auto")}>
                        <StackSelector<Enemy>
                            champion_id={current_player.champion_id}
                            level={current_player.level}
                            enemies={enemies.clone()}
                            items_meta={items_meta.clone()}
                            runes_meta={runes_meta.clone()}
                        />
                    </div>
                </div>
            }
        }
        Err(e) => {
            html! {
                <div>{e}</div>
            }
        }
    }
}
