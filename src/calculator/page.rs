use crate::{
    calculator::{
        Game, InputGame, Player, PlayerData,
        components::inputs::player::PlayerInput,
        reducer::{DataAction, DragonAction, Enemies, EnemyAction, LastAction, PlayerAction},
    },
    components::image::Image,
    model::{Dragons, SimpleStats},
    utils::{ImageType, fetch::post_bytes},
};
use std::{cell::RefCell, rc::Rc};
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

    let player_props = PlayerProps {
        player: player.clone(),
        last_action: last_action.clone(),
    };

    html! {
        <div>
            <PlayerInput {player_props} />
            {match *game_data {
                Some(ref data) => {
                    let Game {
                        monster_damages,
                        current_player,
                        enemies,
                        tower_damages,
                        abilities_meta,
                        abilities_to_merge,
                        items_meta,
                        runes_meta
                    } = data;
                    html! {
                        <div>
                            <Image src={ImageType::from(current_player.champion_id)} />
                            <span>{ current_player.champion_id.name() }</span>
                            <div class={classes!("flex", "gap-4")}>
                            {
                                abilities_meta
                                    .into_iter()
                                    .map(|metadata| html! {
                                        <Image src={ImageType::Ability(
                                            current_player.champion_id,
                                            metadata.kind
                                        )} />
                                    })
                                    .collect::<Html>()
                            }
                            </div>
                        </div>
                    }
                },
                None => html! { "No data" }
            }}
        </div>
    }
}
