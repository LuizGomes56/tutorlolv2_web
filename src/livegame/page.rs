use crate::{
    components::{
        errorlog::errorlog,
        image::{Image, ImageType},
        stack::{Stack, StackInsert, StackRemover, StackTable},
        tables::{empty::EmptyTable, header::TableHeader},
        tray::TrayAction,
    },
    livegame::{Enemy, Game},
    utils::{Fetch, Loading, encode_offset, glue::get_data},
};
use tutorlolv2_gen::CastId;
use yew::{
    platform::{spawn_local, time::sleep},
    prelude::*,
};

#[component]
pub fn Livegame() -> Html {
    let game_data = use_state(|| Err::<Game, _>(Loading.into()));
    let stack = use_reducer(Stack::default);

    let stack_push = {
        let stack = stack.clone();
        use_callback((), move |value, _| {
            stack.dispatch(TrayAction::Insert(value))
        })
    };

    {
        let game_data = game_data.clone();
        use_effect_with((), |_| {
            spawn_local(async move {
                loop {
                    let data = get_data().await;
                    game_data.set(data);
                    sleep(Fetch::REFRESH_RATE).await;
                }
            });
        });
    };

    let data = match &*game_data {
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

            let damages = enemies
                .iter()
                .map(|enemy| {
                    let damages =
                        enemy
                            .damages
                            .to_html(current_player.champion_id, items_meta, runes_meta);
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
                .collect::<Html>();

            html! {
                <>
                    <div class={classes!("box", "overflow-auto")}>
                        <table class={classes!("data-table")}>
                            <TableHeader
                                champion_id={current_player.champion_id}
                                items_meta={items_meta.clone()}
                                runes_meta={runes_meta.clone()}
                            />
                            <tbody>{damages}</tbody>
                        </table>
                    </div>
                    <div class={classes!("box", "overflow-auto")}>
                        <div class={classes!(
                            "grid", "grid-cols-1",
                            "items-start", "gap-4",
                            "2xl:grid-cols-3",
                        )}>
                            <StackInsert
                                callback={stack_push.clone()}
                                items_meta={items_meta.clone()}
                                runes_meta={runes_meta.clone()}
                                champion_id={current_player.champion_id}
                            />
                            <StackRemover
                                stack={stack.clone()}
                                champion_id={current_player.champion_id}
                                items_meta={items_meta.clone()}
                                runes_meta={runes_meta.clone()}
                            />
                            <div class={classes!("overflow-auto")}>
                                <StackTable<Enemy>
                                    enemies={enemies.clone()}
                                    stack={stack.reconcile(current_player.champion_id, items_meta, runes_meta)}
                                    level={current_player.level}
                                />
                            </div>
                        </div>
                    </div>
                </>
            }
        }
        Err(e) => {
            html! {
                <>
                    {errorlog(e)}
                    <EmptyTable rows={5} />
                    <div class={classes!("box", "h-96")} />
                </>
            }
        }
    };

    html! {
        <div class={classes!(
            "flex", "flex-col", "gap-4",
            "p-4", "overflow-hidden",
            "flex-1"
        )}>
            {data}
        </div>
    }
}
