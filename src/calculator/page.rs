use crate::{
    components::tables::body::DamageRenderer,
    hover::{self, Builtin, DocView, Docs},
};
use implicit_clone::ImplicitClone;
use tutorlolv2::CtxVar;
use {
    crate::{
        calculator::{
            Game, Player, PlayerData,
            components::inputs::{
                enemies::EnemiesInput, item_selector::ItemSelector, player::PlayerInput,
                rune_selector::RuneSelector,
            },
            reducer::{DataAction, Enemies, EnemyAction, LastAction, PlayerAction},
        },
        components::{
            image::{DragonImage, Image, ImageType, MinionImage, MonsterImage, OtherImage},
            stack::{Stack, StackInsert, StackRemover, StackTable},
            tables::{empty::EmptyTable, header::TableHeader, turret::TurretTable},
        },
        utils::{ClassCast, transport},
    },
    std::{cell::RefCell, rc::Rc},
    tutorlolv2::{
        L_MSTR, L_TWRD,
        model::{Dragons, EnemyStats, InputActivePlayer, InputGame, InputMinData, OutputEnemy},
    },
    yew::{platform::spawn_local, prelude::*},
};

#[derive(Clone, PartialEq, Properties, ImplicitClone)]
pub struct PlayerProps {
    pub player: UseReducerHandle<Player>,
    pub last_action: Rc<RefCell<LastAction>>,
}

#[derive(Clone, PartialEq, ImplicitClone)]
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

async fn game_fetch(
    player: &UseReducerHandle<Player>,
    enemies: &UseReducerHandle<Enemies>,
    dragons: &UseReducerHandle<Dragons>,
) -> Result<Game, transport::Error> {
    let output = transport::calculator(InputGame {
        active_player: InputActivePlayer {
            runes: player.runes.values(),
            rune_exceptions: player.rune_exceptions.values(),
            abilities: player.abilities,
            data: InputMinData {
                stats: (!player.data.infer_stats).then_some(player.data.stats),
                items: player.data.items.values(),
                item_exceptions: player.data.item_exceptions.values(),
                stacks: player.data.stacks,
                level: player.data.level,
                champion_id: player.data.champion_id,
                is_mega_gnar: player.data.is_mega_gnar,
            },
        },
        enemy_players: &enemies
            .iter()
            .map(|enemy| InputMinData {
                stats: (!enemy.infer_stats).then_some(enemy.stats),
                items: enemy.items.values(),
                item_exceptions: enemy.item_exceptions.values(),
                stacks: enemy.stacks,
                level: enemy.level,
                champion_id: enemy.champion_id,
                is_mega_gnar: enemy.is_mega_gnar,
            })
            .collect::<Box<_>>(),
        dragons: **dragons,
    })
    .await?;

    Ok(output.into())
}

#[component]
pub fn Calculator() -> Html {
    let player = use_reducer(Player::default);
    let enemies = use_reducer(Enemies::default);
    let dragons = use_reducer(Dragons::default);
    let enemy_index = use_state(|| 0);
    let stack = use_reducer(Stack::default);
    let stack_push = Stack::use_push(&stack);

    let game_data = use_state(|| None::<Game>);
    let request_error = use_state(|| None::<transport::Error>);
    let request_pending = use_state(|| true);
    let request_generation = use_mut_ref(|| 0_u64);
    let last_action = use_mut_ref(|| LastAction::Init);
    let entity = use_state(|| TargetEntity::Player);
    let is_item_modal_open = use_state(|| false);
    let is_rune_modal_open = use_state(|| false);

    let open_item_menu = {
        let entity = entity.clone();
        let is_item_modal_open = is_item_modal_open.clone();

        use_callback((), move |v, _| {
            entity.set(v);
            is_item_modal_open.set(true);
        })
    };

    let open_rune_menu = {
        let entity = entity.clone();
        let is_rune_modal_open = is_rune_modal_open.clone();

        use_callback((), move |v, _| {
            entity.set(v);
            is_rune_modal_open.set(true);
        })
    };

    {
        let game_data = game_data.clone();
        let request_error = request_error.clone();
        let request_pending = request_pending.clone();
        let request_generation = request_generation.clone();
        let last_action = last_action.clone();

        use_effect_with(
            (player.clone(), enemies.clone(), dragons.clone()),
            move |(player, enemies, dragons)| {
                let generation = request_generation.borrow().wrapping_add(1);

                request_generation.replace(generation);

                let cleanup_generation = request_generation.clone();
                let action = *last_action.borrow();

                if action == LastAction::Replace {
                    last_action.replace(LastAction::Any);
                } else {
                    let player = player.clone();
                    let enemies = enemies.clone();
                    let dragons = dragons.clone();

                    request_error.set(None);
                    request_pending.set(true);

                    spawn_local(async move {
                        let result = game_fetch(&player, &enemies, &dragons).await;

                        if *request_generation.borrow() != generation {
                            return;
                        }

                        request_pending.set(false);

                        let data = match result {
                            Ok(data) => data,
                            Err(error) => {
                                request_error.set(Some(error));
                                game_data.set(None);
                                return;
                            }
                        };

                        // Every accepted reply represents the full current snapshot.
                        // Update all inferred fields: an earlier request may have been
                        // superseded before it could initialize the other inputs.
                        if player.data.infer_stats
                            && player.data.stats != data.current_player.current_stats
                        {
                            last_action.replace(LastAction::Replace);
                            player.dispatch(PlayerAction::Data(DataAction::ReplaceStats(
                                &data.current_player.current_stats as _,
                            )));
                        }

                        for (index, enemy) in data.enemies.iter().enumerate() {
                            if let Some(input_enemy) =
                                enemies.get(index) as Option<&Rc<PlayerData<EnemyStats>>>
                                && input_enemy.infer_stats
                                && input_enemy.stats != enemy.current_stats
                            {
                                last_action.replace(LastAction::Replace);
                                enemies.dispatch(EnemyAction::Change(
                                    index,
                                    DataAction::ReplaceStats(&enemy.current_stats as _),
                                ));
                            }
                        }

                        game_data.set(Some(data));
                    });
                }
                move || {
                    if *cleanup_generation.borrow() == generation {
                        cleanup_generation.replace(generation.wrapping_add(1));
                    }
                }
            },
        );
    }

    let data = match game_data.as_ref() {
        Some(data) => {
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
                                {items_meta}
                                {runes_meta}
                            />
                            <tbody>
                                {
                                    enemies.iter().enumerate().map(|(i, enemy)| {
                                        let damages = DamageRenderer::new(
                                            &enemy.damages,
                                            current_player.champion_id,
                                            items_meta,
                                            runes_meta,
                                            None
                                        );

                                        let enemy_id = enemy.champion_id;

                                        html! {
                                            <tr>
                                                <td
                                                    data_idents={hover::encode_idents(
                                                        &CtxVar::ENEMY_VARS,
                                                        &enemy.damages.ctx,
                                                    )}
                                                    data_docs={Docs::global(enemy_id).encode()}
                                                    class={classes!("w-12")}
                                                >
                                                    <button
                                                        class={classes!(
                                                            "cursor-pointer",
                                                            "outline-hidden",
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
                                {items_meta}
                                {runes_meta}
                            />
                            <tbody>
                                {
                                    monster_damages.iter().enumerate().map(|(i, damage)| {
                                        let damages = DamageRenderer::new(
                                            damage,
                                            current_player.champion_id,
                                            items_meta,
                                            runes_meta,
                                            None
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

                                        html! {
                                            <tr>
                                                {images}
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
                        <TurretTable
                            damages={{
                                (0..L_TWRD).into_iter().map(|i| {
                                    html! {
                                        <td
                                            data_docs={Docs::Builtin(Builtin::Tower, DocView::Function).encode()}
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
                                callback={stack_push}
                                {items_meta}
                                {runes_meta}
                                champion_id={current_player.champion_id}
                            />
                            <StackRemover
                                stack={&stack}
                                champion_id={current_player.champion_id}
                                {items_meta}
                                {runes_meta}
                            />
                            <div class={classes!("overflow-auto")}>
                                <StackTable<OutputEnemy>
                                    callback={{
                                        let enemy_index = enemy_index.clone();

                                        Callback::from(move |i| enemy_index.set(i))
                                    }}
                                    enemies={enemies}
                                    stack={stack.reconcile(current_player.champion_id, items_meta, runes_meta)}
                                    level={current_player.level}
                                />
                            </div>
                        </div>
                    </div>
                </>
            }
        }
        None => {
            html! {
                <>
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
        <>
            <ItemSelector
                player_props={&player_props}
                enemy_props={&enemy_props}
                {entity}
                is_open={is_item_modal_open}
            />
            <RuneSelector
                player_props={&player_props}
                is_open={is_rune_modal_open}
            />
            <div class={classes!(
                "flex", "flex-wrap", "mb-96",
                "w-full", "px-2", "mt-2"
            )}>
                <PlayerInput
                    {player_props}
                    dragons={&dragons}
                    open_item_menu={&open_item_menu}
                    {open_rune_menu}
                />
                <div class={classes!(
                    "flex", "flex-col", "gap-4",
                    "p-2", "overflow-hidden",
                    "flex-1", "order-3", "xl:order-2",
                    "min-w-full", "xl:min-w-0"
                )}>
                    if *request_pending {
                        <p role={"status"} class={classes!("px-2", "py-1", "text-sm", "text-std-400")}>
                            {"Calculating..."}
                        </p>
                    }

                    if let Some(error) = request_error.as_ref() {
                        {crate::components::errorlog::errorlog(error)}
                    }

                    {data}
                </div>
                <EnemiesInput
                    {dragons}
                    {enemy_props}
                    {open_item_menu}
                />
            </div>
        </>
    }
}
