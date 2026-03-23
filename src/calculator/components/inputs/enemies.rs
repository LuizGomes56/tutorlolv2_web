use crate::{
    calculator::{
        components::inputs::{
            banner::Banner,
            checkbox::Checkbox,
            dragon::{DragonInput, use_dragons},
            exceptions::{ChampionExceptionSelector, ExceptionSelector},
            stats::{StatCell, Stats},
        },
        page::{EnemyProps, TargetEntity},
        reducer::{DataAction, EnemyAction, EnemyDataAction, LastAction},
    },
    components::{
        image::{DragonImage, ImageType},
        selector::SelectorButton,
    },
    model::{Dragons, DragonsAction, EnemyStats},
};
use tutorlolv2_gen::{ChampionId, ItemId};
use web_sys::HtmlInputElement;
use yew::prelude::*;

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

    let enemy = enemy_props
        .enemies
        .get(*enemy_props.enemy_index)
        .unwrap_or_else(|| &enemy_props.enemies[0]);

    html! {
        <div class={classes!(
            "flex", "flex-col", "box",
            "w-full", "max-w-80", "sm:w-64",
            "m-2", "order-2", "xl:order-3"
        )}>
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
                        value={dragons.enemy_earth}
                    />
                }
                <ChampionExceptionSelector
                    champion_id={enemy.champion_id}
                    stacks={enemy.stacks}
                    callback={stack_callback}
                    ally={false}
                />
                <ExceptionSelector<{ ItemId::SIZE_OF_EXCEPTIONS }, ItemId>
                    values={enemy.items.values::<Box<_>>()}
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
                    value={enemy.level as i32}
                    placeholder={1}
                    oninput={{
                        let callback = level_callback.clone();
                        Callback::from(move |e: InputEvent| {
                            let value = e.target_unchecked_into::<HtmlInputElement>().value();
                            let number = value.parse().unwrap_or(1).max(1);
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
