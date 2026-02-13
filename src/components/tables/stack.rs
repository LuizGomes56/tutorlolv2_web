use crate::components::tables::body::Victim;
use std::rc::Rc;
use tutorlolv2_gen::{AbilityId, ChampionId, ItemId, RuneId, TypeMetadata};
use yew::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StackValue {
    Ability(AbilityId),
    Item(ItemId),
    Rune(RuneId),
    BasicAttack,
    CriticalStrike,
    Onhit,
    Ignite,
}

#[derive(PartialEq, Properties)]
pub struct StackProps<T: Victim + PartialEq + 'static> {
    pub champion_id: ChampionId,
    pub enemies: Rc<[T]>,
    pub items_meta: Rc<[TypeMetadata<ItemId>]>,
    pub runes_meta: Rc<[TypeMetadata<RuneId>]>,
}

#[component]
pub fn Stack<T: Victim + PartialEq + 'static>(props: &StackProps<T>) -> Html {
    todo!()
    // let StackProps {} = props;
    // html! {
    //     <div></div>
    // }
}
