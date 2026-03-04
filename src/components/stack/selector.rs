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

    let selector = use_memo(
        (champion_id, items_meta.clone(), runes_meta.clone()),
        |data| {
            let (champion_id, ref items_meta, ref runes_meta) = *data;
            let abilities = champion_id
                .cache()
                .metadata
                .iter()
                .enumerate()
                .map(|(i, metadata)| {
                    let kind = metadata.kind;
                    html! {
                        <button onclick={{
                            let stack_push = stack_push.clone();
                            Callback::from(move |_| {
                                stack_push.emit(StackValue::Ability(i, champion_id, kind));
                            })
                        }}>
                            <Image
                                class={class.clone()}
                                src={ImageType::Ability(
                                    champion_id,
                                    AbilityKind::Normal(kind)
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
                    let kind = metadata.kind;
                    html! {
                        <button onclick={{
                            let stack_push = stack_push.clone();
                            Callback::from(move |_| {
                                stack_push.emit(StackValue::Item(i, kind));
                            })
                        }}>
                            <Image
                                class={class.clone()}
                                src={ImageType::from(kind)}
                            />
                        </button>
                    }
                })
                .collect::<Html>();

            let runes = runes_meta
                .iter()
                .enumerate()
                .map(|(i, metadata)| {
                    let kind = metadata.kind;
                    html! {
                        <button onclick={{
                            let stack_push = stack_push.clone();
                            Callback::from(move |_| {
                                stack_push.emit(StackValue::Rune(i, kind));
                            })
                        }}>
                            <Image
                                class={class.clone()}
                                src={ImageType::from(kind)}
                            />
                        </button>
                    }
                })
                .collect::<Html>();

            html! {
                <div class={classes!("flex", "gap-2", "flex-wrap")}>
                    {abilities}
                    {items}
                    {runes}
                </div>
            }
        },
    );

    let remover = stack
        .iter()
        .copied()
        .enumerate()
        .map(|(i, value)| {
            let image_type = match value {
                StackValue::Ability(_, champion_id, ability_id) => {
                    ImageType::Ability(champion_id, ability_id.into())
                }
                StackValue::Item(_, item_id) => ImageType::from(item_id),
                StackValue::Rune(_, rune_id) => ImageType::from(rune_id),
                StackValue::BasicAttack => ImageType::BasicAttack,
                StackValue::CriticalStrike => ImageType::CritStrike,
                StackValue::OnhitMin => ImageType::OnhitAttack,
                StackValue::OnhitMax => ImageType::OnhitAttack,
                StackValue::Ignite(_) => ImageType::Ignite,
            };

            html! {
                <button onclick={{
                    let stack_remove = stack_remove.clone();
                    Callback::from(move |_| {
                        stack_remove.emit(i);
                    })
                }}>
                    <Image
                        class={class.clone()}
                        src={image_type}
                    />
                </button>
            }
        })
        .collect::<Html>();

    html! {
        <div class={classes!("grid", "grid-cols-3", "gap-4")}>
            {(*selector).clone()}
            <div class={classes!("flex", "gap-2", "flex-wrap")}>
                {remover}
                <button onclick={Callback::from(move |_: MouseEvent| clear_stack.emit(()))}>
                    { "Clear stack" }
                </button>
            </div>
            <div>
                <StackTable<T>
                    enemies={enemies.clone()}
                    stack={stack.0.clone()}
                />
            </div>
        </div>
    }
}
