use crate::{
    calculator::{
        AbilityLevels, FinalEnemy, Game, InputGame, Player, PlayerData,
        components::inputs::player::PlayerInput,
        reducer::{DataAction, Enemies, EnemyAction, LastAction, PlayerAction},
    },
    components::tables::{body::TableBody, header::TableHeader},
    model::{Dragons, PlayerStats},
    utils::{EnumCast, fetch::Fetch},
};
use std::{cell::RefCell, rc::Rc};
use tutorlolv2_gen::ChampionId;
use web_sys::AbortController;
use yew::{platform::spawn_local, prelude::*};

#[derive(Clone, PartialEq)]
pub struct PlayerProps {
    pub player: UseReducerHandle<Player>,
    pub last_action: Rc<RefCell<LastAction>>,
}

#[derive(Clone, PartialEq)]
pub struct EnemyProps {
    pub enemies: UseReducerHandle<Enemies>,
    pub enemy_index: UseStateHandle<usize>,
    pub last_action: Rc<RefCell<LastAction>>,
}

#[component]
pub fn Calculator() -> Html {
    let player = use_reducer(Player::default);
    let enemies = use_reducer(Enemies::default);
    let dragons = use_reducer(Dragons::default);

    let game_data = use_state(|| None::<Game>);
    let controller = use_state(|| None::<AbortController>);
    let last_action = use_mut_ref(|| LastAction::Init);

    {
        let enemies = enemies.clone();
        let player = player.clone();
        use_effect_with((), move |_| {
            player.dispatch(PlayerAction::Data(DataAction::Stats(unsafe {
                &core::mem::transmute::<_, PlayerStats>([100; 16]) as *const _
            })));
            player.dispatch(PlayerAction::AbilityLevels(AbilityLevels {
                q: 5,
                w: 5,
                e: 5,
                r: 3,
            }));
            player.dispatch(PlayerAction::Data(DataAction::ChampionId(
                ChampionId::Aatrox,
            )));
            (0..5).for_each(|i| {
                enemies.dispatch(EnemyAction::Insert);
                enemies.dispatch(EnemyAction::Change(
                    i,
                    DataAction::ChampionId(ChampionId::random()),
                ))
            });
        })
    };

    {
        let game_data = game_data.clone();
        let controller = controller.clone();
        let player = player.clone();
        let enemies = enemies.clone();
        let dragons = dragons.clone();
        let last_action = last_action.clone();
        use_effect_with((player.clone(), enemies.clone()), move |_| {
            if *last_action.borrow() == LastAction::Replace {
                last_action.replace(LastAction::Any);
                return;
            };

            if let Some(controller) = &*controller {
                controller.abort();
            }

            let new_controller = AbortController::new().ok();
            let signal = new_controller.as_ref().map(|c| c.signal());
            controller.set(new_controller);

            spawn_local(async move {
                let input_game = InputGame {
                    active_player: (*player).clone(),
                    enemy_players: (*enemies).to_vec(),
                    dragons: *dragons,
                };

                web_sys::console::log_1(&format!("{input_game:#?}").into());

                match Fetch::new("/api/games/calculator")
                    .signal(signal)
                    .body_with_bincode(&input_game)
                    .unwrap()
                    .post::<Game>()
                    .await
                {
                    Ok(data) => {
                        let infer_enemy_player_stats = |index| {
                            let enemy: &Rc<PlayerData<_>> = &enemies[index];
                            if enemy.infer_stats {
                                last_action.replace(LastAction::Replace);
                                enemies.dispatch(EnemyAction::Change(
                                    index,
                                    DataAction::Stats(&enemy.stats as _),
                                ));
                            }
                        };
                        let action = *last_action.borrow();
                        match action {
                            LastAction::Init | LastAction::CurrentPlayer => {
                                if player.data.infer_stats {
                                    last_action.replace(LastAction::Replace);
                                    player.dispatch(PlayerAction::Data(DataAction::Stats(
                                        &data.current_player.current_stats as _,
                                    )));
                                }
                                if action == LastAction::Init {
                                    (0..data.enemies.len()).for_each(infer_enemy_player_stats);
                                }
                            }
                            LastAction::EnemyPlayer(index) => infer_enemy_player_stats(index),
                            _ => {}
                        };

                        web_sys::console::log_1(&format!("{data:#?}").into());

                        game_data.set(Some(data));
                    }
                    Err(e) => web_sys::console::error_1(
                        &format!("Failed to request calculator api: {e:?}").into(),
                    ),
                }
            });
        });
    }

    let player_props = PlayerProps {
        player: player.clone(),
        last_action: last_action.clone(),
    };

    html! {
        <div class={classes!("flex", "mb-72", "p-4", "gap-4")}>
            <PlayerInput {player_props} />
            {match *game_data {
                Some(ref data) => {
                    let Game {
                        monster_damages,
                        current_player,
                        enemies,
                        tower_damages,
                        items_meta,
                        runes_meta
                    } = data;
                    html! {
                        <div>
                            <div class={classes!("box")}>
                                <table>
                                    <TableHeader
                                        champion_id={current_player.champion_id}
                                        items_meta={items_meta.clone()}
                                        runes_meta={runes_meta.clone()}
                                    />
                                    <tbody>
                                        <TableBody<FinalEnemy>
                                            champion_id={current_player.champion_id}
                                            enemies={enemies}
                                            items_meta={items_meta.clone()}
                                            runes_meta={runes_meta.clone()}
                                        />
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    }
                },
                None => html! { "No data" }
            }}
        </div>
    }
}
