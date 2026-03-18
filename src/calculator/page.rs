use crate::{
    calculator::{
        FinalEnemy, Game, InputGame, Player, PlayerData,
        components::inputs::{
            enemies::EnemiesInput, item_selector::ItemSelector, player::PlayerInput,
        },
        reducer::{DataAction, Enemies, EnemyAction, LastAction, PlayerAction},
    },
    components::{
        errorlog::errorlog,
        image::{DragonImage, Image, ImageType, MinionImage, MonsterImage, OtherImage},
        stack::{Stack, StackInsert, StackRemover, StackTable},
        tables::{empty::EmptyTable, header::TableHeader, turret::TurretTable},
    },
    model::{Dragons, EnemyStats},
    utils::{ClassCast, Fetch, Loading, Print, encode_offset, tray::TrayAction},
};
use std::{cell::RefCell, rc::Rc};
use tutorlolv2_gen::{CastId, L_MSTR, L_TWRD, TOWER_DAMAGE_FN_OFFSET};
use web_sys::AbortController;
use yew::{platform::spawn_local, prelude::*};

#[derive(Clone, PartialEq, Properties)]
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TargetEntity {
    Player,
    Enemy(usize),
}

#[component]
pub fn Calculator() -> Html {
    let player = use_reducer(Player::default);
    let enemies = use_reducer(Enemies::default);
    let dragons = use_reducer(Dragons::default);
    let enemy_index = use_state(|| 0);
    let stack = use_reducer(Stack::default);
    let stack_push = Stack::use_push(&stack);

    let game_data = use_state(|| Err::<Game, _>(Loading.into()));
    let controller = use_state(|| None::<AbortController>);
    let last_action = use_mut_ref(|| LastAction::Init);
    let entity = use_state(|| TargetEntity::Player);
    let is_item_modal_open = use_state(|| false);

    let open_item_menu = {
        let entity = entity.clone();
        let is_item_modal_open = is_item_modal_open.clone();
        use_callback((), move |v, _| {
            entity.set(v);
            is_item_modal_open.set(true);
        })
    };

    {
        let game_data = game_data.clone();
        let controller = controller.clone();
        let last_action = last_action.clone();

        use_effect_with(
            (player.clone(), enemies.clone(), dragons.clone()),
            move |(player, enemies, dragons)| {
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

                let player = player.clone();
                let enemies = enemies.clone();
                let dragons = dragons.clone();

                spawn_local(async move {
                    let input_game = InputGame {
                        active_player: &player,
                        enemy_players: enemies.as_slice(),
                        dragons: &dragons,
                    };

                    input_game.active_player.data.items.log();

                    if let Ok(req) = Fetch::new("/api/games/calculator")
                        .signal(signal)
                        .body_with_bincode(&input_game)
                    {
                        match req.post::<Game>().await {
                            Ok(data) => {
                                let infer_enemy_player_stats = |index| {
                                    if let Some(enemy) = &data.enemies.get(index)
                                        && let Some(input_enemy) = enemies.get(index)
                                            as Option<&Rc<PlayerData<EnemyStats>>>
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
                                            (0..data.enemies.len())
                                                .for_each(infer_enemy_player_stats);
                                        }
                                    }
                                    LastAction::EnemyPlayer(index) => {
                                        infer_enemy_player_stats(index)
                                    }
                                    _ => {}
                                };

                                game_data.set(Ok(data));
                            }
                            Err(e) => {
                                let error = e.to_string();
                                if error != "AbortError: signal is aborted without reason" {
                                    format!("Failed to request calculator api: {e}").log();
                                    game_data.set(Err(e));
                                }
                            }
                        }
                    }
                });
            },
        );
    }

    let data = match game_data.as_ref() {
        Ok(data) => {
            let Game {
                monster_damages,
                current_player,
                enemies,
                tower_damages,
                items_meta,
                runes_meta,
            } = data;
            html! {
                <>
                    <div class={classes!("box", "overflow-auto")}>
                        <table class={classes!("data-table")}>
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
                                                    class={classes!("w-12")}
                                                    data_offset={encode_offset(&[enemy_id.formula()])}
                                                >
                                                    <button
                                                        class={classes!(
                                                            "cursor-pointer",
                                                            "outline-none",
                                                            "focus:ring-1",
                                                            "focus:ring-blue-500/75",
                                                        )}
                                                        onclick={{
                                                            let enemy_index = enemy_index.clone();
                                                            Callback::from(move |_| {
                                                                enemy_index.set(i);
                                                            })
                                                        }}
                                                    >
                                                        <Image src={ImageType::from(enemy_id)} />
                                                    </button>
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
                        <table class={classes!("data-table")}>
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
                                            images.push(html!(
                                                <td class={classes!(
                                                    "px-1", "w-8", "first:pl-2", "last:pr-2"
                                                )}>
                                                    {cell}
                                                </td>
                                            ));
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
                                <StackTable<FinalEnemy>
                                    callback={{
                                        let enemy_index = enemy_index.clone();
                                        Callback::from(move |i| {
                                            enemy_index.set(i);
                                        })
                                    }}
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
                    <EmptyTable rows={MONSTER_HEADERS.len()} />
                    <EmptyTable rows={1} />
                    <div class={classes!("box", "h-96")} />
                </>
            }
        }
    };

    let player_props = PlayerProps {
        player,
        last_action: last_action.clone(),
    };

    let enemy_props = EnemyProps {
        enemies,
        enemy_index,
        last_action,
    };

    html! {
        <div class={classes!("flex", "mb-96", "w-full", "px-2", "mt-2")}>
            <ItemSelector
                player_props={player_props.clone()}
                enemy_props={enemy_props.clone()}
                {entity}
                is_open={is_item_modal_open}
            />
            <PlayerInput
                {player_props}
                dragons={dragons.clone()}
                open_item_menu={open_item_menu.clone()}
            />
            <div class={classes!(
                "flex", "flex-col", "gap-4",
                "p-2", "overflow-hidden",
                "flex-1"
            )}>
                {data}
            </div>
            <EnemiesInput
                {dragons}
                {enemy_props}
                {open_item_menu}
            />
        </div>
    }
}
