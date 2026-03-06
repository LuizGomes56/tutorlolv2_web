use crate::{
    components::{
        image::{Image, ImageType},
        stack::{Stack, StackAction, StackEntry, StackTable, StackValue},
        tables::body::Victim,
    },
    model::AbilityKind,
    utils::encode_offset,
};
use std::rc::Rc;
use tutorlolv2_gen::{
    BASIC_ATTACK_OFFSET, CRITICAL_STRIKE_OFFSET, CastId, ChampionId, IGNITE_OFFSET, ItemId,
    ONHIT_EFFECT_OFFSET, RuneId, TypeMetadata,
};
use yew::prelude::*;

fn section(title: &str, iterator: impl ExactSizeIterator<Item = Html>) -> Option<Html> {
    (iterator.len() > 0).then_some(html! {
        <div class={classes!("flex", "flex-col", "gap-3")}>
            <h2 class={classes!("text-xl", "text-std-200", "font-medium")}>{title}</h2>
            <div class={classes!("flex", "gap-2", "flex-wrap")}>{iterator.collect::<Html>()}</div>
        </div>
    })
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

    let stack_push = {
        let stack = stack.clone();
        use_callback((), move |value, _| {
            stack.dispatch(StackAction::Insert(StackEntry::new(value)))
        })
    };

    let stack_remove = {
        let stack = stack.clone();
        use_callback((), move |id, _| stack.dispatch(StackAction::RemoveById(id)))
    };

    let clear_stack = {
        let stack = stack.clone();
        use_callback((), move |_, _| stack.dispatch(StackAction::Clear))
    };

    let safe_stack = stack.reconcile(champion_id, items_meta, runes_meta);

    {
        let stack = stack.clone();
        let items_meta = items_meta.clone();
        let runes_meta = runes_meta.clone();

        use_effect_with(
            (items_meta.clone(), runes_meta.clone(), champion_id),
            move |(items_meta, runes_meta, ..)| {
                let next = stack.reconcile(champion_id, items_meta, runes_meta);
                if *stack != next {
                    stack.dispatch(StackAction::Replace(next));
                }
            },
        );
    }

    let class = classes!("w-8", "h-8", "cursor-pointer");

    let selector = use_memo(
        (items_meta.clone(), runes_meta.clone(), champion_id),
        |data| {
            let (items_meta, runes_meta, ..) = data;

            let abilities = champion_id
                .abilities()
                .iter()
                .enumerate()
                .map(|(slot, metadata)| {
                    let ability_id = metadata.kind;
                    let data_offset = encode_offset(&[champion_id.get_ability_formula(slot)]);

                    html! {
                        <button {data_offset} onclick={{
                            let stack_push = stack_push.clone();
                            Callback::from(move |_| {
                                stack_push.emit(StackValue::Ability {
                                    slot,
                                    champion_id,
                                    ability_id,
                                });
                            })
                        }}>
                            <Image
                                class={class.clone()}
                                src={ImageType::Ability(champion_id, AbilityKind::Normal(ability_id))}
                            />
                        </button>
                    }
                });

            let items = items_meta.iter().enumerate().map(|(i, metadata)| {
                let item_id = metadata.kind;
                let data_offset = encode_offset(&[item_id.formula()]);

                html! {
                    <button {data_offset} onclick={{
                        let stack_push = stack_push.clone();
                        Callback::from(move |_| stack_push.emit(StackValue::Item(i, item_id)))
                    }}>
                        <Image class={class.clone()} src={ImageType::from(item_id)} />
                    </button>
                }
            });

            let runes = runes_meta.iter().enumerate().map(|(i, metadata)| {
                let rune_id = metadata.kind;
                let data_offset = encode_offset(&[rune_id.formula()]);

                html! {
                    <button {data_offset} onclick={{
                        let stack_push = stack_push.clone();
                        Callback::from(move |_| stack_push.emit(StackValue::Rune(i, rune_id)))
                    }}>
                        <Image class={class.clone()} src={ImageType::from(rune_id)} />
                    </button>
                }
            });

            html! {
                <>
                    {section("Abilities", abilities)}
                    {section("Items", items)}
                    {section("Runes", runes)}
                </>
            }
        },
    );

    let other = use_memo(stack_push.clone(), |stack_push| {
        let buttons = [
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
            (ImageType::Ignite, &IGNITE_OFFSET, StackValue::Ignite),
        ]
        .into_iter()
        .map(|(image_type, offset, v)| {
            let data_offset = encode_offset(&[offset]);

            html! {
                <button {data_offset} onclick={{
                    let stack_push = stack_push.clone();
                    Callback::from(move |_| stack_push.emit(v))
                }}>
                    <Image class={class.clone()} src={image_type} />
                </button>
            }
        });

        section("Other", buttons)
    });

    let remover = safe_stack
        .iter()
        .copied()
        .map(|entry| {
            let (image_type, offset) = match entry.value {
                StackValue::Ability {
                    slot,
                    champion_id,
                    ability_id,
                } => (
                    ImageType::Ability(champion_id, ability_id.into()),
                    champion_id.get_ability_formula(slot),
                ),
                StackValue::Item(_, item_id) => (ImageType::from(item_id), item_id.formula()),
                StackValue::Rune(_, rune_id) => (ImageType::from(rune_id), rune_id.formula()),
                StackValue::BasicAttack => (ImageType::BasicAttack, &BASIC_ATTACK_OFFSET),
                StackValue::CritStrike => (ImageType::CritStrike, &CRITICAL_STRIKE_OFFSET),
                StackValue::OnhitMin => (ImageType::OnhitAttack, &ONHIT_EFFECT_OFFSET),
                StackValue::OnhitMax => (ImageType::OnhitAttack, &ONHIT_EFFECT_OFFSET),
                StackValue::Ignite => (ImageType::Ignite, &IGNITE_OFFSET),
            };

            let data_offset = encode_offset(&[offset]);
            let id = entry.id;

            html! {
                <button {data_offset} onclick={{
                    let stack_remove = stack_remove.clone();
                    Callback::from(move |_| stack_remove.emit(id))
                }}>
                    <Image class={class.clone()} src={image_type} />
                </button>
            }
        })
        .collect::<Html>();

    html! {
        <div class={classes!("grid", "grid-cols-3", "gap-4", "items-start")}>
            <div class={classes!("flex", "flex-col", "gap-2", "px-5", "py-4")}>
                {(*selector).clone()}
                {(*other).clone()}
            </div>
            <div class={classes!("flex", "gap-2", "flex-wrap")}>
                {remover}
                <button onclick={Callback::from(move |_: MouseEvent| clear_stack.emit(()))}>
                    {"Clear stack"}
                </button>
            </div>
            <div class={classes!("overflow-auto")}>
                <StackTable<T>
                    enemies={enemies.clone()}
                    stack={safe_stack}
                    level={level}
                />
            </div>
        </div>
    }
}
