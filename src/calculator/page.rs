use crate::{
    calculator::{
        Game, InputGame, Player, PlayerData,
        reducer::{DataAction, DragonAction, Enemies, EnemyAction, EnemyDataAction, PlayerAction},
    },
    model::{Dragons, SimpleStats},
    utils::fetch::post_bytes,
};
use std::{cell::RefCell, rc::Rc};
use web_sys::AbortController;
use yew::{platform::spawn_local, prelude::*};

#[derive(PartialEq, Clone, Copy)]
pub enum LastAction {
    Init,
    Any,
    CurrentPlayer,
    EnemyPlayer(usize),
    Replace,
}

#[derive(Clone)]
pub struct PlayerProps {
    pub player: UseReducerHandle<Player>,
    pub last_action: Rc<RefCell<LastAction>>,
}

#[hook]
pub fn use_player_callback<T: 'static>(
    props: PlayerProps,
    callback: fn(T) -> PlayerAction,
) -> Callback<T> {
    let PlayerProps {
        player,
        last_action,
    } = props;
    use_callback((), move |v, _| {
        last_action.replace(LastAction::CurrentPlayer);
        player.dispatch(callback(v));
    })
}

#[derive(Clone)]
pub struct EnemyProps {
    pub enemies: UseReducerHandle<Enemies>,
    pub enemy_index: UseStateHandle<usize>,
    pub last_action: Rc<RefCell<LastAction>>,
}

#[hook]
pub fn use_enemy_callback<T: 'static>(
    props: EnemyProps,
    callback: fn(T) -> EnemyDataAction,
) -> Callback<T> {
    let EnemyProps {
        enemies,
        enemy_index,
        last_action,
    } = props;
    use_callback((), move |v, _| {
        let index = *enemy_index;
        last_action.replace(LastAction::EnemyPlayer(index));
        enemies.dispatch(EnemyAction::Change(index, callback(v)));
    })
}

#[hook]
pub fn use_dragon_callback(
    dragons: UseReducerHandle<Dragons>,
    callback: fn(u16) -> DragonAction,
) -> Callback<u16> {
    use_callback((), move |v, _| {
        dragons.dispatch(callback(v));
    })
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

                match post_bytes::<Game>("/api/games/calculator", &input_game, signal).await {
                    Ok(data) => {
                        let infer_enemy_player_stats = |index| {
                            let enemy: &Rc<PlayerData<SimpleStats>> = &enemies[index];
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
                                    (0..data.enemies.len())
                                        .into_iter()
                                        .for_each(infer_enemy_player_stats);
                                }
                            }
                            LastAction::EnemyPlayer(index) => infer_enemy_player_stats(index),
                            _ => {}
                        };
                        game_data.set(Some(data));
                    }
                    Err(e) => web_sys::console::error_1(
                        &format!("Failed to request calculator api: {e:?}").into(),
                    ),
                }
            });
        });
    }

    let enemy_index = use_state(|| 0);
    let player_props = PlayerProps {
        player,
        last_action: last_action.clone(),
    };
    let enemy_props = EnemyProps {
        enemies,
        enemy_index,
        last_action,
    };

    todo!()
}
