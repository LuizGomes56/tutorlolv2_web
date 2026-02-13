use crate::{
    calculator::{
        components::inputs::{
            banner::Banner,
            stats::{StatCell, Stats},
        },
        page::EnemyProps,
        reducer::{DataAction, Enemies, EnemyAction, EnemyDataAction, LastAction},
    },
    components::image::ImageType,
    model::EnemyStats,
    utils::EnumCast,
};
use std::{cell::RefCell, rc::Rc};
use tutorlolv2_gen::ChampionId;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct EnemiesInputProps {
    pub enemies: UseReducerHandle<Enemies>,
    pub enemy_index: UseStateHandle<usize>,
    pub last_action: Rc<RefCell<LastAction>>,
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
    } = props.clone();
    use_callback((), move |v, _| {
        let value = callback(v);
        last_action.replace(LastAction::EnemyPlayer(*enemy_index));
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
    } = props.clone();
    use_callback((), move |v, _| {
        let index = *enemy_index;
        let value = callback(v);
        last_action.replace(value.action(LastAction::EnemyPlayer(index)));
        enemies.dispatch(EnemyAction::Change(index, value));
    })
}

#[component]
pub fn EnemiesInput(props: &EnemiesInputProps) -> Html {
    let EnemiesInputProps {
        enemies,
        enemy_index,
        last_action,
    } = props;

    let enemy_props = EnemyProps {
        enemies: enemies.clone(),
        enemy_index: enemy_index.clone(),
        last_action: last_action.clone(),
    };

    let add_enemy = use_enemy_callback(&enemy_props, EnemyAction::Insert);
    let remove_enemy = use_enemy_callback(&enemy_props, EnemyAction::Remove);

    let level_cb = use_enemy_data_callback(&enemy_props, DataAction::Level);
    let stats_cb = use_enemy_data_callback(&enemy_props, DataAction::Stats);

    let enemy = enemies.get(**enemy_index).unwrap_or(&enemies[0]);

    html! {
        <>
            <div class={classes!("flex", "flex-col", "w-64", "box", "m-2")}>
                <Banner champion_id={enemy.champion_id} />
                <div class={classes!(
                    "grid", "grid-cols-[auto,1fr,1fr]",
                    "gap-x-2", "px-4", "py-3", "gap-y-0.5"
                )}>
                    <StatCell
                        image_type={ImageType::Level}
                        name={"Level"}
                        disabled={false}
                        value={enemy.level as i32}
                        placeholder={1}
                        oninput={{
                            let callback = level_cb.clone();
                            Callback::from(move |e: InputEvent| {
                                let value = e.target_unchecked_into::<HtmlInputElement>().value();
                                let number = value.parse().unwrap_or(1);
                                callback.emit(number);
                            })
                        }}
                    />
                    <Stats<EnemyStats>
                        infer={enemy.infer_stats}
                        stats={enemy.stats}
                        callback={stats_cb}
                    />
                </div>
            </div>
        </>
    }
}
