use crate::{
    calculator::{
        Player,
        reducer::{DragonAction, Enemies, EnemyAction, EnemyDataAction, PlayerAction},
    },
    model::Dragons,
};
use std::{cell::RefCell, rc::Rc};
use web_sys::AbortController;
use yew::prelude::*;

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
        enemies.dispatch(EnemyAction::Edit(index, callback(v)));
    })
}

#[derive(Clone)]
pub struct DragonCallbackProps {
    pub dragons: UseReducerHandle<Dragons>,
    pub prev_action: Rc<RefCell<LastAction>>,
    pub next_action: LastAction,
}

#[hook]
pub fn use_dragon_callback(
    props: DragonCallbackProps,
    callback: fn(u16) -> DragonAction,
) -> Callback<u16> {
    let DragonCallbackProps {
        dragons,
        prev_action,
        next_action,
    } = props;
    use_callback((), move |v, _| {
        prev_action.replace(next_action);
        dragons.dispatch(callback(v));
    })
}

#[component]
pub fn Calculator() -> Html {
    let player = use_reducer(Player::default);
    let enemies = use_reducer(Enemies::default);

    todo!()
}

/*
#[hook]
pub fn use_damage_stack_callback<T: 'static>(
    damage_stack: UseReducerHandle<Stack>,
    enum_const_fn: fn(T) -> StackAction,
) -> Callback<T> {
    use_callback((), move |v, _| damage_stack.dispatch(enum_const_fn(v)))
}

#[function_component(Calculator)]
pub fn calculator() -> Html {
    let input_current_player = use_reducer(InputActivePlayer::default);
    let input_enemy_players = use_reducer(InputEnemies::new);
    let input_dragons = use_reducer(Dragons::default);

    let output_game = use_state(|| None::<OutputGame>);
    let abort_controller = use_state(|| None::<AbortController>);
    let damage_stack = use_reducer(Stack::default);
    let action_tracker = use_mut_ref(|| LastAction::Init);

    let cb_insert_stack = use_damage_stack_callback(damage_stack.clone(), StackAction::Insert);
    let cb_remove_stack = use_damage_stack_callback(damage_stack, StackAction::Remove);

    // use_effect_with(damage_stack.clone(), move |damage_stack| {
    //     web_sys::console::log_1(&format!("{:#?}", damage_stack.clone_inner()).into());
    // });

    {
        let output_game = output_game.clone();
        let abort_controller = abort_controller.clone();
        let input_current_player = input_current_player.clone();
        let input_enemy_players = input_enemy_players.clone();
        let action_tracker = action_tracker.clone();
        use_effect_with(
            (input_current_player.clone(), input_enemy_players.clone()),
            move |_| {
                let current_action = *action_tracker.borrow();
                if current_action == LastAction::Replace {
                    action_tracker.replace(LastAction::Any);
                    return;
                };

                if let Some(controller) = &*abort_controller {
                    controller.abort();
                }

                let new_controller = AbortController::new().ok();
                let signal = new_controller.as_ref().map(|c| c.signal());
                abort_controller.set(new_controller);
                spawn_local(async move {
                    let input_game = InputGame {
                        active_player: (&*input_current_player).into(),
                        enemy_players: (*input_enemy_players).as_ref(),
                        dragons: Dragons::default(),
                    };

                    // web_sys::console::log_1(&format!("{:#?}", input_game).into());

                    let response =
                        encode_bytes(url!("/api/games/calculator"), &input_game, signal).await;

                    if let Some(res) = response
                        && let Some(data) = decode_bytes::<OutputGame>(res).await
                    {
                        let last_action = *action_tracker.borrow();
                        let infer_current_player_stats = || {
                            let mut action_ref_mut = action_tracker.borrow_mut();
                            if input_current_player.data.infer_stats {
                                *action_ref_mut = LastAction::Replace;
                                input_current_player.dispatch(InputActivePlayerAction::Data(
                                    InputDataAction::Stats(
                                        &data.current_player.current_stats as *const _,
                                    ),
                                ));
                            }
                        };
                        let infer_enemy_player_stats = |index| unsafe {
                            let mut action_ref_mut = action_tracker.borrow_mut();
                            let input_enemies = input_enemy_players.as_ref();
                            let current_enemy: &Rc<
                                MinData<SimpleStats, Vec<ItemId>, Vec<ValueException>>,
                            > = input_enemies.get_unchecked(index);
                            if current_enemy.infer_stats {
                                *action_ref_mut = LastAction::Replace;
                                input_enemy_players.dispatch(EnemyDataAction::Edit(
                                    index,
                                    InputDataAction::Stats(
                                        &data.enemies.get_unchecked(index).current_stats
                                            as *const _,
                                    ),
                                ));
                            }
                        };
                        match last_action {
                            LastAction::Init => {
                                infer_current_player_stats();
                                for i in 0..data.enemies.len() {
                                    infer_enemy_player_stats(i);
                                }
                            }
                            LastAction::CurrentPlayer => {
                                infer_current_player_stats();
                            }
                            LastAction::EnemyPlayer(index) => {
                                infer_enemy_player_stats(index);
                            }
                            _ => {}
                        };
                        // web_sys::console::log_1(&format!("{:#?}", data).into());
                        output_game.set(Some(data));
                    }
                });
            },
        );
    }

    let make_td = |text: i32, damage_type: DamageType| -> Html {
        html! {
            <td class={classes!{
                "text-center", "text-sm", "px-2", "h-10",
                "max-w-24", "truncate", damage_type.as_static_str(),
            }}>
                {text}
            </td>
        }
    };

    html! {
        <div class={classes!(
            "oxanium", "gap-4", "grid", "grid-cols-[auto_1fr_auto]", "w-full"
        )}>
            <ActivePlayerData
                input_current_player={input_current_player}
                input_dragons={input_dragons.clone()}
                action_tracker={action_tracker.clone()}
            />
            <div>
                // {
                //     if let Some(output_game) = &*output_game {
                //         html! {
                //             <div class={classes!(
                //                 "flex", "flex-col", "gap-6", "py-2"
                //             )}>
                //                 <BaseTable
                //                     damaging_items={output_game.current_player.damaging_items.clone()}
                //                     damaging_runes={output_game.current_player.damaging_runes.clone()}
                //                     champion_id={current_player_champion_id}
                //                     damages={
                //                         output_game.enemies
                //                             .iter()
                //                             .map(|(enemy_champion_id, enemy)| {
                //                                 html! {
                //                                     <tr>
                //                                         <td class={classes!("w-10", "h-10")}>
                //                                             <ImageCell instance={Instances::Champions(*enemy_champion_id)} />
                //                                         </td>
                //                                         {enemy.damages.attacks.display_damage()}
                //                                         {enemy.damages.abilities.display_damage()}
                //                                         {enemy.damages.items.display_damage()}
                //                                         {enemy.damages.runes.display_damage()}
                //                                     </tr>
                //                                 }
                //                             })
                //                             .collect::<Html>()
                //                     }
                //                 />
                //                 <DamageStackSelector
                //                     items={output_game.current_player.damaging_items.clone()}
                //                     runes={output_game.current_player.damaging_runes.clone()}
                //                     champion_id={current_player_champion_id}
                //                     stack={(*damage_stack).clone_inner()}
                //                     push_callback={push_stack_callback}
                //                     remove_callback={remove_stack_callback}
                //                     damages={
                //                         output_game.enemies
                //                             .iter()
                //                             .map(|(enemy_champion_id, enemy)| {
                //                                 let mut total_damage = 0;
                //                                 for value in (*damage_stack).get_ref() {
                //                                     match value {
                //                                         StackValue::Ability(ability) => {
                //                                             if let Some((_, instance_damage)) = enemy.damages.abilities.iter().find(|(a, _)| a == ability) {
                //                                                 total_damage += instance_damage.minimum_damage;
                //                                             }
                //                                         },
                //                                         StackValue::BasicAttack => {
                //                                             total_damage += enemy.damages.attacks.basic_attack.minimum_damage;
                //                                         }
                //                                         StackValue::CriticalStrike => {
                //                                             total_damage += enemy.damages.attacks.critical_strike.minimum_damage;
                //                                         },
                //                                         StackValue::Onhit => {
                //                                             total_damage += enemy.damages.attacks.onhit_damage.minimum_damage;
                //                                         },
                //                                         StackValue::Item(item_id) => {
                //                                             if let Ok(index) = enemy.damages.items.binary_search_by_key(item_id, |(key, _)| *key) {
                //                                                 let instance_damage = &enemy.damages.items[index].1;
                //                                                 total_damage += instance_damage.minimum_damage;
                //                                             }
                //                                         },
                //                                         StackValue::Rune(rune_id) => {
                //                                             if let Ok(index) = enemy.damages.runes.binary_search_by_key(rune_id, |(key, _)| *key) {
                //                                                 let instance_damage = &enemy.damages.runes[index].1;
                //                                                 total_damage += instance_damage.minimum_damage;
                //                                             }
                //                                         },
                //                                         StackValue::Ignite => {
                //                                             total_damage += 50i32 + 20i32 * input_current_player.level as i32;
                //                                         }
                //                                     }
                //                                 }
                //                                 html! {
                //                                     <tr>
                //                                         <td class={classes!("w-10", "h-10")}>
                //                                             <ImageCell instance={Instances::Champions(*enemy_champion_id)} />
                //                                         </td>
                //                                         {make_td(total_damage, DamageType::Mixed)}
                //                                         {make_td(enemy.current_stats.health - total_damage, DamageType::Unknown)}
                //                                         {make_td(total_damage / enemy.current_stats.health * 100, DamageType::True)}
                //                                     </tr>
                //                                 }
                //                             })
                //                             .collect::<Html>()
                //                     }
                //                 />
                //                 <TurretTable
                //                     damages={
                //                         (0..6).into_iter().map(|i| {
                //                             html! {
                //                                 <td class={classes!(
                //                                     "text-center", "text-sm",
                //                                     "px-2", output_game.current_player.adaptative_type.get_color(),
                //                                     "max-w-24", "truncate", "h-10"
                //                                 )}>
                //                                     {output_game.tower_damage[i]}
                //                                 </td>
                //                             }
                //                         })
                //                         .collect::<Html>()
                //                     }
                //                 />
                //                 <BaseTable
                //                     empty_headers={4}
                //                     damaging_items={output_game.current_player.damaging_items.clone()}
                //                     damaging_runes={output_game.current_player.damaging_runes.clone()}
                //                     champion_id={current_player_champion_id}
                //                     damages={
                //                         ([
                //                             &[
                //                                 url!("/img/other/voidgrubs.avif"),
                //                                 url!("/img/other/melee_minion.avif"),
                //                                 url!("/img/other/ranged_minion.avif"),
                //                                 url!("/img/other/cannon.avif"),
                //                             ],
                //                             &[url!("/img/other/super_minion.avif")],
                //                             &[
                //                                 url!("/img/other/elder_dragon.avif"),
                //                                 url!("/img/other/fire_dragon.avif"),
                //                                 url!("/img/other/ocean_dragon.avif"),
                //                                 url!("/img/other/earth_dragon.avif"),
                //                             ],
                //                             &[url!("/img/other/baron.avif")],
                //                             &[url!("/img/other/atakhan.avif")],
                //                             &[
                //                                 url!("/img/other/red_buff.avif"),
                //                                 url!("/img/other/blue_buff.avif"),
                //                                 url!("/img/other/gromp.avif"),
                //                                 url!("/img/other/wolves.avif"),
                //                             ],
                //                             &[
                //                                 url!("/img/other/krug.avif"),
                //                                 url!("/img/other/raptor.avif"),
                //                             ],
                //                         ] as [&[&'static str]; 7])
                //                         .iter()
                //                         .enumerate()
                //                         .map(|(index, urls)| {
                //                             html! {
                //                                 <tr>
                //                                     {
                //                                         for (0..4).map(|i| {
                //                                             html!{
                //                                                 <td class={classes!("h-10", "justify-items-center")}>
                //                                                     {
                //                                                         if let Some(&icon_url) = urls.get(i) {
                //                                                             html! {
                //                                                                 <Image
                //                                                                     class={classes!("w-8", "h-8")}
                //                                                                     source={ImageType::Other(AttrValue::Static(icon_url))}
                //                                                                 />
                //                                                             }
                //                                                         } else {
                //                                                             html!{}
                //                                                         }
                //                                                     }
                //                                                 </td>
                //                                             }
                //                                         })
                //                                     }
                //                                     { output_game.monster_damages.join_td_index(index) }
                //                                 </tr>
                //                             }
                //                         })
                //                         .collect::<Html>()
                //                     }
                //                 />
                //             </div>
                //         }
                //     } else {
                //         html! {}
                //     }
                // }
            </div>
            <EnemyPlayersData
                input_enemy_players={input_enemy_players}
                input_dragons={input_dragons}
                action_tracker={action_tracker}
            />
        </div>
    }
}
*/
