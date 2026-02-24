use crate::{
    calculator::{
        FinalEnemy, Game, InputGame, Player, PlayerData,
        components::inputs::{enemies::EnemiesInput, player::PlayerInput},
        reducer::{DataAction, Enemies, EnemyAction, LastAction, PlayerAction},
    },
    components::{
        image::{DragonImage, Image, ImageType, MinionImage, MonsterImage, OtherImage},
        stack::StackSelector,
        tables::{header::TableHeader, turret::TurretTable},
    },
    model::{AbilityLevelsAction, Dragons, EnemyStats, PlayerStats},
    utils::{
        ClassCast, EnumCast, encode_offset,
        fetch::Fetch,
        traits::{Print, random_u16},
    },
};
use std::{cell::RefCell, rc::Rc};
use tutorlolv2_gen::{CastId, ChampionId, L_MSTR, L_TWRD, TOWER_DAMAGE_FN_OFFSET};
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

const MONSTER_HEADERS: [&[OtherImage]; L_MSTR] = [
    &[
        OtherImage::Voidgrubs,
        OtherImage::Minion(MinionImage::Melee),
        OtherImage::Minion(MinionImage::Ranged),
        OtherImage::Minion(MinionImage::Cannon),
    ],
    &[
        OtherImage::Dragon(DragonImage::Elder),
        OtherImage::Dragon(DragonImage::Fire),
        OtherImage::Dragon(DragonImage::Ocean),
        OtherImage::Dragon(DragonImage::Earth),
    ],
    &[
        OtherImage::Monster(MonsterImage::Red),
        OtherImage::Monster(MonsterImage::Blue),
        OtherImage::Monster(MonsterImage::Gromp),
        OtherImage::Monster(MonsterImage::Wolves),
    ],
    &[
        OtherImage::Monster(MonsterImage::Krug),
        OtherImage::Monster(MonsterImage::Raptor),
    ],
    &[OtherImage::Baron],
    &[OtherImage::Atakhan],
    &[OtherImage::Minion(MinionImage::Super)],
];

const MONSTER_COUNT: usize = {
    let mut i = 0;
    let mut max = 0;
    while i < L_MSTR {
        let len = MONSTER_HEADERS[i].len();
        if len > max {
            max = len;
        }
        i += 1;
    }
    max
};

#[component]
pub fn Calculator() -> Html {
    let player = use_reducer(Player::default);
    let enemies = use_reducer(Enemies::default);
    let dragons = use_reducer(Dragons::default);
    let enemy_index = use_state(|| 0);

    let game_data = use_state(|| None::<Game>);
    let controller = use_state(|| None::<AbortController>);
    let last_action = use_mut_ref(|| LastAction::Init);

    {
        let player = player.clone();
        let enemies = enemies.clone();
        use_effect_with((), move |_| {
            // player.dispatch(PlayerAction::Data(DataAction::InferStats(false)));
            // let random = |i| random_u16(0..i) as _;
            // let stats = PlayerStats {
            //     ability_power: random(1600),
            //     armor: random(350),
            //     armor_penetration_flat: 0,
            //     armor_penetration_percent: 0,
            //     attack_damage: random(600),
            //     attack_range: random(750),
            //     attack_speed: random(2),
            //     crit_chance: random(100),
            //     crit_damage: random(200),
            //     current_health: random(5000),
            //     magic_penetration_flat: 0,
            //     magic_penetration_percent: 0,
            //     magic_resist: random(300),
            //     max_health: random(5000),
            //     max_mana: random(2000),
            //     current_mana: random(2000),
            // };
            player.dispatch(PlayerAction::Data(DataAction::Level(18)));
            // let champion_id = player.data.champion_id;
            // player.dispatch(PlayerAction::Data(DataAction::SetItemVec(
            //     champion_id.recommended_items(champion_id.main_position()),
            // )));
            // player.dispatch(PlayerAction::Data(DataAction::ReplaceStats(
            //     &stats as *const _,
            // )));
            player.dispatch(PlayerAction::AbilityLevels(AbilityLevelsAction::Q(5)));
            player.dispatch(PlayerAction::AbilityLevels(AbilityLevelsAction::W(5)));
            player.dispatch(PlayerAction::AbilityLevels(AbilityLevelsAction::E(5)));
            player.dispatch(PlayerAction::AbilityLevels(AbilityLevelsAction::R(3)));
            player.dispatch(PlayerAction::Data(DataAction::ChampionId(
                ChampionId::random(),
            )));
            enemies.dispatch(EnemyAction::Change(0, DataAction::Level(18)));
            // enemies.dispatch(EnemyAction::Change(0, DataAction::InferStats(false)));
            // enemies.dispatch(EnemyAction::Change(0, {
            //     let champion_id = enemies[0].champion_id;
            //     DataAction::SetItemVec(champion_id.recommended_items(champion_id.main_position()))
            // }));
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

                input_game.enemy_players[0].stats.log();

                if let Ok(req) = Fetch::new("/api/games/calculator")
                    .signal(signal)
                    .body_with_bincode(&input_game)
                {
                    match req.post::<Game>().await {
                        Ok(data) => {
                            let infer_enemy_player_stats = |index| {
                                if let Some(enemy) = &data.enemies.get(index)
                                    && let Some(input_enemy) =
                                        enemies.get(index) as Option<&Rc<PlayerData<EnemyStats>>>
                                    && input_enemy.infer_stats
                                {
                                    last_action.replace(LastAction::Replace);
                                    enemies.dispatch(EnemyAction::Change(
                                        index,
                                        DataAction::ReplaceStats(&enemy.current_stats as _),
                                    ));
                                }
                            };
                            let action = *last_action.borrow();
                            match action {
                                LastAction::Init | LastAction::CurrentPlayer => {
                                    if player.data.infer_stats {
                                        // data.current_player.log();
                                        last_action.replace(LastAction::Replace);
                                        player.dispatch(PlayerAction::Data(
                                            DataAction::ReplaceStats(
                                                &data.current_player.current_stats as _,
                                            ),
                                        ));
                                    }
                                    if action == LastAction::Init {
                                        (0..data.enemies.len()).for_each(infer_enemy_player_stats);
                                    }
                                }
                                LastAction::EnemyPlayer(index) => infer_enemy_player_stats(index),
                                _ => {}
                            };

                            data.current_player.current_stats.log();

                            game_data.set(Some(data));
                        }
                        Err(e) => format!("Failed to request calculator api: {e:?}").err(),
                    }
                }
            });
        });
    }

    let player_props = PlayerProps {
        player: player.clone(),
        last_action: last_action.clone(),
    };

    html! {
        <div class={classes!("flex", "mb-96", "w-full", "px-2", "mt-2")}>
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
                        <div class={classes!(
                            "flex", "flex-col", "gap-4",
                            "p-2", "overflow-hidden",
                            "flex-1"
                        )}>
                            <div class={classes!("box", "overflow-auto")}>
                                <table>
                                    <TableHeader
                                        champion_id={current_player.champion_id}
                                        items_meta={items_meta.clone()}
                                        runes_meta={runes_meta.clone()}
                                    />
                                    <tbody>
                                        {
                                            enemies.iter().enumerate().map(|(i, enemy)| {
                                                let damages = enemy.damages.to_html(
                                                    current_player.champion_id,
                                                    items_meta,
                                                    runes_meta
                                                );
                                                let enemy_id = enemy.champion_id;
                                                html! {
                                                    <tr>
                                                        <td
                                                            class={classes!("w-8", "h-8")}
                                                            onclick={{
                                                                let enemy_index = enemy_index.clone();
                                                                Callback::from(move |_| {
                                                                    enemy_index.set(i);
                                                                })
                                                            }}
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
                                <table>
                                    <TableHeader
                                        skip={MONSTER_COUNT}
                                        champion_id={current_player.champion_id}
                                        items_meta={items_meta.clone()}
                                        runes_meta={runes_meta.clone()}
                                    />
                                    <tbody>
                                        {
                                            monster_damages.iter().enumerate().map(|(i, damage)| {
                                                let damages = damage.to_html(
                                                    current_player.champion_id,
                                                    items_meta,
                                                    runes_meta
                                                );

                                                let mut images = Vec::with_capacity(MONSTER_COUNT);
                                                for j in (0..MONSTER_COUNT).rev() {
                                                    let cell = MONSTER_HEADERS[i].get(j).map(|&value| {
                                                        html!(<Image src={ImageType::Other(value)} />)
                                                    });
                                                    images.push(html!(<td class={classes!("w-8", "h-8")}>{cell}</td>));
                                                }

                                                html!(<tr>{images}{damages}</tr>)
                                            })
                                            .collect::<Html>()
                                        }
                                    </tbody>
                                </table>
                            </div>
                            <div class={classes!("box", "overflow-auto")}>
                                <TurretTable
                                    damages={{
                                        let offset = encode_offset(&[&TOWER_DAMAGE_FN_OFFSET]);
                                        (0..L_TWRD).into_iter().map(|i| {
                                            html! {
                                                <td
                                                    data_offset={offset.clone()}
                                                    class={classes!(current_player.adaptive_type.class())}
                                                >
                                                    {tower_damages[i]}
                                                </td>
                                            }
                                        })
                                        .collect::<Html>()
                                    }}
                                />
                            </div>
                            <div class={classes!("box", "overflow-auto")}>
                                <StackSelector<FinalEnemy>
                                    champion_id={current_player.champion_id}
                                    enemies={enemies.clone()}
                                    items_meta={items_meta.clone()}
                                    runes_meta={runes_meta.clone()}
                                />
                            </div>
                        </div>
                    }
                },
                None => html!("No data")
            }}
            <EnemiesInput
                enemies={enemies.clone()}
                enemy_index={enemy_index.clone()}
                last_action={last_action.clone()}
            />
        </div>
    }
}
