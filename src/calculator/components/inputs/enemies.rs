use crate::calculator::{
    page::EnemyProps,
    reducer::{Enemies, EnemyAction, EnemyDataAction, LastAction},
};
use std::{cell::RefCell, rc::Rc};
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct EnemiesInputProps {
    pub enemies: UseReducerHandle<Enemies>,
    pub last_action: Rc<RefCell<LastAction>>,
}

#[hook]
pub fn use_enemy_callback<T: 'static>(
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
        last_action.replace(value.action(index));
        enemies.dispatch(EnemyAction::Change(index, value));
    })
}

#[component]
pub fn EnemiesInput(props: &EnemiesInputProps) -> Html {
    let EnemiesInputProps {
        enemies,
        last_action,
    } = props;

    let enemy_index = use_state(|| 0);
    let enemy_props = EnemyProps {
        enemies: enemies.clone(),
        enemy_index,
        last_action: last_action.clone(),
    };

    html! {
        <div></div>
    }
}
