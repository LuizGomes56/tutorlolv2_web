use crate::{
    components::{
        image::{Image, ImageType},
        stack::{Stack, StackAction, StackTable, StackValue},
        tables::body::Victim,
    },
    model::AbilityKind,
    utils::{encode_offset, traits::Print},
};
use std::rc::Rc;
use tutorlolv2_gen::{
    BASIC_ATTACK_OFFSET, CRITICAL_STRIKE_OFFSET, CastId, ChampionId, ItemId, ONHIT_EFFECT_OFFSET,
    RuneId, TypeMetadata,
};
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
    pub enemies: Rc<[T]>,
    pub items_meta: Rc<[TypeMetadata<ItemId>]>,
    pub runes_meta: Rc<[TypeMetadata<RuneId>]>,
    pub champion_id: ChampionId,
    pub level: u8,
}

#[component]
pub fn StackSelector<T: Victim + PartialEq + 'static>(props: &StackSelectorProps<T>) -> Html {
    let StackSelectorProps {
        champion_id,
        level,
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
        (items_meta.clone(), runes_meta.clone(), champion_id, level),
        |data| {
            let (items_meta, runes_meta, ..) = data;
            let abilities = champion_id
                .cache()
                .metadata
                .iter()
                .enumerate()
                .map(|(i, metadata)| {
                    let kind = metadata.kind;
                    let data_offset = encode_offset(&[champion_id.get_ability_formula(i)]);

                    html! {
                        <button {data_offset} onclick={{
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
                    let data_offset = encode_offset(&[kind.formula()]);

                    html! {
                        <button {data_offset} onclick={{
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
                    let data_offset = encode_offset(&[kind.formula()]);

                    html! {
                        <button {data_offset} onclick={{
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

            let section = |title: &str, buttons: Html| {
                html! {
                    <div class={classes!("flex", "flex-col", "gap-3")}>
                        <h2 class={classes!("text-xl", "text-std-200", "font-medium")}>{title}</h2>
                        <div class={classes!("flex", "gap-2", "flex-wrap")}>{buttons}</div>
                    </div>
                }
            };

            let other = [
                (
                    ImageType::BasicAttack,
                    &BASIC_ATTACK_OFFSET,
                    StackValue::BasicAttack,
                ),
                (
                    ImageType::CritStrike,
                    &CRITICAL_STRIKE_OFFSET,
                    StackValue::CritStrike,
                ),
                (
                    ImageType::OnhitAttack,
                    &ONHIT_EFFECT_OFFSET,
                    StackValue::OnhitMin,
                ),
                (
                    ImageType::OnhitAttack,
                    &ONHIT_EFFECT_OFFSET,
                    StackValue::OnhitMax,
                ),
                (ImageType::Ignite, &(0..0), StackValue::Ignite(level)),
            ]
            .into_iter()
            .map(|(image_type, offset, stack_value)| {
                let data_offset = encode_offset(&[offset]);

                html! {
                    <button {data_offset} onclick={{
                        let stack_push = stack_push.clone();
                        Callback::from(move |_| stack_push.emit(stack_value))
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
                <div class={classes!("flex", "flex-col", "gap-2", "px-5", "py-4")}>
                    {section("Abilities", abilities)}
                    {section("Items", items)}
                    {section("Runes", runes)}
                    {section("Other", other)}
                </div>
            }
        },
    );

    // clean_stack
    let cleanup = stack
        .iter()
        .copied()
        .enumerate()
        .map(|(i, value)| match value {
            StackValue::Ability(_, id, ..) => champion_id == id,
            StackValue::Item(j, v) => items_meta.get(j).is_some_and(|m| m.kind == v),
            StackValue::Rune(j, v) => runes_meta.get(j).is_some_and(|m| m.kind == v),
            StackValue::Ignite(j) => i != j as usize,
            _ => false,
        })
        .collect::<Vec<_>>();

    let remover = stack
        .iter()
        .copied()
        .enumerate()
        .map(|(i, value)| {
            let (image_type, offset) = match value {
                StackValue::Ability(j, champion_id, ability_id) => (
                    ImageType::Ability(champion_id, ability_id.into()),
                    champion_id.get_ability_formula(j),
                ),
                StackValue::Item(_, item_id) => (ImageType::from(item_id), item_id.formula()),
                StackValue::Rune(_, rune_id) => (ImageType::from(rune_id), rune_id.formula()),
                StackValue::BasicAttack => (ImageType::BasicAttack, &BASIC_ATTACK_OFFSET),
                StackValue::CritStrike => (ImageType::CritStrike, &CRITICAL_STRIKE_OFFSET),
                StackValue::OnhitMin => (ImageType::OnhitAttack, &ONHIT_EFFECT_OFFSET),
                StackValue::OnhitMax => (ImageType::OnhitAttack, &ONHIT_EFFECT_OFFSET),
                StackValue::Ignite(_) => (ImageType::Ignite, &(0..0)),
            };

            let data_offset = encode_offset(&[offset]);

            html! {
                <button {data_offset} onclick={{
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
        <div class={classes!("grid", "grid-cols-3", "gap-4", "items-start")}>
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
