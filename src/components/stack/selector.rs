use crate::{
    components::{
        image::{Image, ImageType},
        stack::{Stack, StackAction, StackTable, StackValue},
        tables::body::Victim,
    },
    model::AbilityKind,
};
use std::rc::Rc;
use tutorlolv2_gen::{ChampionId, ItemId, RuneId, TypeMetadata};
use yew::prelude::*;

#[hook]
fn use_stack<T: 'static>(
    stack: &UseReducerHandle<Stack>,
    action: fn(T) -> StackAction,
) -> Callback<T> {
    let stack = stack.clone();
    use_callback((), move |v, _| stack.dispatch(action(v)))
}

#[derive(PartialEq, Properties)]
pub struct StackSelectorProps<T: Victim + PartialEq + 'static> {
    pub champion_id: ChampionId,
    pub enemies: Rc<[T]>,
    pub items_meta: Rc<[TypeMetadata<ItemId>]>,
    pub runes_meta: Rc<[TypeMetadata<RuneId>]>,
}

#[component]
pub fn StackSelector<T: Victim + PartialEq + 'static>(props: &StackSelectorProps<T>) -> Html {
    let StackSelectorProps {
        champion_id,
        ref enemies,
        ref items_meta,
        ref runes_meta,
    } = *props;

    let stack = use_reducer(Stack::default);

    let stack_push = use_stack(&stack, StackAction::Insert);
    let stack_remove = use_stack(&stack, StackAction::Remove);
    let clear_stack = use_callback((), {
        let stack = stack.clone();
        move |_: (), _| stack.dispatch(StackAction::Clear)
    });

    let class = classes!("w-8", "h-8", "cursor-pointer");

    let abilities = champion_id
        .cache()
        .metadata
        .iter()
        .enumerate()
        .map(|(i, metadata)| {
            html! {
                <button onclick={{
                    let stack_push = stack_push.clone();
                    Callback::from(move |_| {
                        stack_push.emit(StackValue::Ability(i));
                    })
                }}>
                    <Image
                        class={class.clone()}
                        src={ImageType::Ability(
                            champion_id,
                            AbilityKind::Normal(metadata.kind)
                        )}
                    />
                </button>
            }
        })
        .collect::<Html>();

    let items = items_meta
        .iter()
        .enumerate()
        .map(|(i, metadata)| {
            html! {
                <button onclick={{
                    let stack_push = stack_push.clone();
                    Callback::from(move |_| {
                        stack_push.emit(StackValue::Item(i));
                    })
                }}>
                    <Image
                        class={class.clone()}
                        src={ImageType::from(metadata.kind)}
                    />
                </button>
            }
        })
        .collect::<Html>();

    let runes = runes_meta
        .iter()
        .enumerate()
        .map(|(i, metadata)| {
            html! {
                <button onclick={{
                    let stack_push = stack_push.clone();
                    Callback::from(move |_| {
                        stack_push.emit(StackValue::Rune(i));
                    })
                }}>
                    <Image
                        class={class.clone()}
                        src={ImageType::from(metadata.kind)}
                    />
                </button>
            }
        })
        .collect::<Html>();

    html! {
        <div class={classes!("grid", "grid-cols-3")}>
            <div class={classes!("flex", "gap-2")}>
                {abilities}
                {items}
                {runes}
            </div>
            <div>
            </div>
            <div>
                <StackTable<T>
                    champion_id={champion_id}
                    enemies={enemies.clone()}
                    items_meta={items_meta.clone()}
                    runes_meta={runes_meta.clone()}
                    stack={stack.boxed()}
                />
            </div>
        </div>
    }
}
