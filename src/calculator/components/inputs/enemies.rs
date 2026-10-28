use {
    crate::{
        calculator::{
            components::inputs::{
                banner::Banner,
                checkbox::Checkbox,
                dragon::DragonInput,
                exceptions::{ChampionExceptionSelector, ExceptionSelector},
                stats::{StatCell, Stats},
            },
            page::{EnemyProps, TargetEntity},
            reducer::{DataAction, Enemies, EnemyAction, EnemyDataAction, LastAction},
        },
        components::{
            image::{DragonImage, ImageType},
            selector::SelectorButton,
        },
        utils::EnumCast,
    },
    std::num::NonZeroU8,
    tutorlolv2::{
        ChampionId, ItemId,
        model::{Dragons, EnemyStats},
        yew::dragons::DragonsAction,
    },
    web_sys::HtmlInputElement,
    yew::prelude::*,
};

#[derive(PartialEq, Properties)]
pub struct EnemiesInputProps {
    pub enemy_props: EnemyProps,
    pub open_item_menu: Callback<TargetEntity>,
    pub dragons: UseReducerHandle<Dragons>,
}

#[hook]
pub fn use_enemy_callback<T: 'static>(
    props: &EnemyProps,
    callback: fn(T) -> EnemyAction,
) -> Callback<T> {
    let EnemyProps {
        enemies,
        last_action,
        enemy_index,
        ..
    } = props.clone();

    use_callback(enemy_index.clone(), move |v, enemy_index| {
        let value = callback(v);

        last_action.replace(LastAction::EnemyPlayer(**enemy_index));
        enemies.dispatch(value);
    })
}

#[hook]
pub fn use_enemy_data_callback<T: 'static>(
    props: &EnemyProps,
    callback: fn(T) -> EnemyDataAction,
) -> Callback<T> {
    let EnemyProps {
        enemies,
        enemy_index,
        last_action,
        ..
    } = props.clone();

    use_callback(enemy_index.clone(), move |v, enemy_index| {
        let index = **enemy_index;
        let value = callback(v);

        last_action.replace(value.action(LastAction::EnemyPlayer(index)));
        enemies.dispatch(EnemyAction::Change(index, value));
    })
}

#[component]
pub fn EnemiesInput(props: &EnemiesInputProps) -> Html {
    let EnemiesInputProps {
        enemy_props,
        open_item_menu,
        dragons,
    } = props;

    let add_enemy = use_enemy_callback(enemy_props, EnemyAction::Insert);
    let remove_enemy = use_enemy_callback(enemy_props, EnemyAction::Remove);

    let level_callback = use_enemy_data_callback(enemy_props, DataAction::Level);
    let stats_callback = use_enemy_data_callback(enemy_props, DataAction::Stats);
    let champion_callback = use_enemy_data_callback(enemy_props, DataAction::ChampionId);
    let stack_callback = use_enemy_data_callback(enemy_props, DataAction::Stacks);
    let infer_stats_callback = use_enemy_data_callback(enemy_props, DataAction::InferStats);
    let item_exception_callback = use_enemy_data_callback(enemy_props, DataAction::ModifyItemExc);
    let is_mega_gnar_callback = use_enemy_data_callback(enemy_props, DataAction::IsMegaGnar);

    let enemy_earth = {
        let dragons = dragons.clone();
        let last_action = enemy_props.last_action.clone();

        use_callback(
            enemy_props.enemy_index.clone(),
            move |e: InputEvent, enemy_index| {
                let index = **enemy_index;
                last_action.replace(LastAction::EnemyPlayer(index));

                let target = e.target_unchecked_into::<HtmlInputElement>();
                let value = target.value().parse::<u16>().unwrap_or(0);

                dragons.dispatch(DragonsAction::EnemyEarth(value));
            },
        )
    };

    let mut index = (*enemy_props.enemy_index + 1) as u32;

    let enemy = enemy_props
        .enemies
        .get(*enemy_props.enemy_index)
        .unwrap_or_else(|| {
            index = 1;

            if enemy_props.enemies.is_empty() {
                core::hint::cold_path();
                add_enemy.emit(ChampionId::random());
            }

            &enemy_props.enemies[0]
        });

    let btn_class = classes!(
        "px-3",
        "py-1.5",
        "rounded-md",
        "border",
        "border-std-800",
        "bg-std-900/70",
        "text-xs",
        "font-semibold",
        "text-std-200",
        "enabled:hover:bg-std-800/60",
        "focus-visible:outline",
        "focus-visible:outline-2",
        "focus-visible:outline-blue-500",
        "enabled:cursor-pointer",
        "disabled:opacity-40",
        "disabled:cursor-not-allowed"
    );

    html! {
        <div class={classes!(
            "flex", "flex-col", "box",
            "w-full", "max-w-80", "sm:w-64",
            "m-2", "order-2", "xl:order-3"
        )}>
            <div class={classes!("flex", "items-center", "justify-between", "gap-2", "px-4", "py-3")}>
                <span class={classes!("text-sm", "font-semibold", "text-std-200", "whitespace-nowrap")}>
                    {"Enemy "}{index}
                </span>
                <div class={classes!("flex", "gap-2")}>
                    <button
                        type={"button"}
                        aria-label={"Add enemy"}
                        title={"Add enemy"}
                        disabled={enemy_props.enemies.len() >= Enemies::MAX_ENEMIES}
                        class={&btn_class}
                        onclick={{
                            let enemy_index = enemy_props.enemy_index.clone();
                            let count = enemy_props.enemies.len();

                            Callback::from(move |_| {
                                if count < Enemies::MAX_ENEMIES {
                                    add_enemy.emit(ChampionId::random());
                                    enemy_index.set(count);
                                }
                            })
                        }}
                    >
                        {"Add"}
                    </button>
                    <button
                        type={"button"}
                        aria-label={"Remove enemy"}
                        title={"Remove selected enemy"}
                        disabled={enemy_props.enemies.len() <= 1}
                        class={btn_class}
                        onclick={{
                            let enemy_index = enemy_props.enemy_index.clone();
                            let selected = index as usize - 1;
                            let count = enemy_props.enemies.len();

                            Callback::from(move |_| {
                                if count > 1 {
                                    remove_enemy.emit(selected);
                                    enemy_index.set(selected.min(count - 2));
                                }
                            })
                        }}
                    >
                        {"Remove"}
                    </button>
                </div>
            </div>
            <Banner
                callback={champion_callback}
                champion_id={enemy.champion_id}
            />
            <div class={classes!("w-full", "my-2")}>
                <SelectorButton
                    title={"Items"}
                    onclick={{
                        let open_item_menu = open_item_menu.clone();
                        let index = *enemy_props.enemy_index;

                        Callback::from(move |_| {
                            open_item_menu.emit(TargetEntity::Enemy(index));
                        })
                    }}
                    length={enemy.items.len()}
                />
            </div>
            <div class={classes!(
                "grid", "grid-cols-[auto_1fr_1fr]",
                "gap-x-2", "px-4", "py-3", "gap-y-1.5",
                "empty:hidden"
            )}>
                if enemy.infer_stats {
                    <DragonInput
                        title={"Earth dragons"}
                        oninput={enemy_earth}
                        src={DragonImage::Earth}
                        value={dragons.enemy_earth_dragons}
                    />
                }
                <ChampionExceptionSelector
                    champion_id={enemy.champion_id}
                    stacks={enemy.stacks}
                    callback={stack_callback}
                    ally={false}
                />
                <ExceptionSelector<ItemId>
                    values={enemy.items.clone()}
                    exceptions={enemy.item_exceptions.clone()}
                    callback={item_exception_callback}
                    filter={ItemId::exceptions(false)}
                />
            </div>
            if enemy.champion_id == ChampionId::Gnar {
                <Checkbox
                    checked={enemy.is_mega_gnar}
                    callback={is_mega_gnar_callback}
                    label={"Mega Gnar"}
                />
            }
            <Checkbox
                checked={enemy.infer_stats}
                callback={infer_stats_callback}
                label={"Infer Stats"}
            />
            <div class={classes!(
                "grid", "grid-cols-[auto_1fr_1fr]",
                "gap-x-2", "px-4", "py-3", "gap-y-0.5"
            )}>
                <StatCell
                    image_type={ImageType::Level}
                    name={"Level"}
                    disabled={false}
                    value={enemy.level.get() as f32}
                    placeholder={1}
                    oninput={{
                        let callback = level_callback.clone();

                        Callback::from(move |e: InputEvent| {
                            let value = e.target_unchecked_into::<HtmlInputElement>().value();
                            let number = value.parse::<NonZeroU8>().unwrap_or(NonZeroU8::MIN);

                            callback.emit(number);
                        })
                    }}
                />
                <Stats<EnemyStats>
                    infer={enemy.infer_stats}
                    stats={enemy.stats}
                    callback={stats_callback}
                />
            </div>
        </div>
    }
}
