use crate::{
    components::{
        image::{Image, ImageType},
        stack::{Stack, StackAction, StackEntry, StackTable, StackValue},
        tables::body::Victim,
    },
    model::AbilityKind,
    utils::{Print, encode_offset},
};
use serde_json::Value;
use std::{collections::HashMap, rc::Rc};
use tutorlolv2_gen::{
    BASIC_ATTACK_OFFSET, CRITICAL_STRIKE_OFFSET, CastId, ChampionId, IGNITE_OFFSET, ItemId,
    ONHIT_EFFECT_OFFSET, RuneId, TypeMetadata,
};
use yew::prelude::*;

fn section(title: &str, iterator: impl ExactSizeIterator<Item = Html>) -> Option<Html> {
    (iterator.len() > 0).then_some(html! {
        <div class={classes!(
            "flex", "flex-col", "gap-2",
            "pt-4", "border-t", "border-std-800/70",
            "first:pt-0", "first:border-t-0"
        )}>
            <div class={classes!("flex", "items-baseline", "justify-between")}>
                <h2 class={classes!(
                    "text-xs", "uppercase", "tracking-wider",
                    "text-std-400", "font-semibold"
                )}>
                    {title}
                </h2>
            </div>
            <div class={classes!(
                "flex", "gap-2", "flex-wrap",
                "items-start"
            )}>
                {iterator.collect::<Html>()}
            </div>
        </div>
    })
}

#[derive(PartialEq, Properties)]
pub struct StackInsertProps {
    #[prop_or_default]
    pub callback: Callback<StackValue>,
    pub items_meta: Rc<[TypeMetadata<ItemId>]>,
    pub runes_meta: Rc<[TypeMetadata<RuneId>]>,
    pub champion_id: ChampionId,
}

#[component]
pub fn StackInsert(props: &StackInsertProps) -> Html {
    let StackInsertProps {
        ref callback,
        ref items_meta,
        ref runes_meta,
        champion_id,
    } = *props;

    fn button(onclick: Callback<MouseEvent>, data_offset: String, src: ImageType) -> Html {
        html! {
            <button
                type={"button"}
                class={classes!(
                    "group",
                    "relative",
                    "flex",
                    "items-center",
                    "justify-center",
                    "w-10",
                    "h-10",
                    "rounded-lg",
                    "border",
                    "border-std-800",
                    "bg-std-900/60",
                    "hover:bg-std-800/60",
                    "hover:border-blue-500/75",
                    "transition-all",
                    "duration-150",
                    "focus-visible:outline",
                    "focus-visible:outline-2",
                    "focus-visible:outline-blue-400"
                )}
                {onclick}
            >
                <div {data_offset}>
                    <Image
                        class={classes!(
                            "w-8", "h-8", "pointer-events-none",
                            "overflow-hidden", "rounded"
                        )}
                        {src}
                    />
                </div>
            </button>
        }
    }

    let other = use_memo(callback.clone(), |callback| {
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
        .map(|(src, offset, v)| {
            let data_offset = encode_offset(&[offset]);

            let onclick = {
                let callback = callback.clone();
                Callback::from(move |_| callback.emit(v))
            };

            button(onclick, data_offset, src)
        });

        section("Other", buttons)
    });

    let abilities = use_memo((callback.clone(), champion_id), |(callback, ..)| {
        let buttons = champion_id
            .abilities()
            .iter()
            .enumerate()
            .map(|(slot, metadata)| {
                let ability_id = metadata.kind;
                let data_offset = encode_offset(&[champion_id.get_ability_formula(slot)]);

                let onclick = {
                    let callback = callback.clone();
                    Callback::from(move |_| {
                        callback.emit(StackValue::Ability {
                            slot,
                            champion_id,
                            ability_id,
                        });
                    })
                };

                button(
                    onclick,
                    data_offset,
                    ImageType::Ability(champion_id, AbilityKind::Normal(ability_id)),
                )
            });

        section("Abilities", buttons)
    });

    fn make_buttons<T>(
        callback: &Callback<StackValue>,
        meta: &Rc<[TypeMetadata<T>]>,
        f: fn(usize, T) -> StackValue,
    ) -> impl ExactSizeIterator<Item = Html>
    where
        T: CastId + PartialEq + Copy,
        ImageType: From<T>,
    {
        meta.iter().enumerate().map(move |(i, metadata)| {
            let id = metadata.kind;
            let data_offset = encode_offset(&[id.formula()]);

            let onclick = {
                let callback = callback.clone();
                Callback::from(move |_| callback.emit(f(i, id)))
            };

            button(onclick, data_offset, ImageType::from(id))
        })
    }

    let items = use_memo(
        (callback.clone(), items_meta.clone()),
        |(callback, items_meta)| {
            section(
                "Items",
                make_buttons(callback, items_meta, StackValue::Item),
            )
        },
    );

    let runes = use_memo(
        (callback.clone(), runes_meta.clone()),
        |(callback, runes_meta)| {
            section(
                "Runes",
                make_buttons(callback, runes_meta, StackValue::Rune),
            )
        },
    );

    html! {
        <div class={classes!(
            "flex", "flex-col",
            "gap-4", "py-4", "px-4", "2xl:pl-4", "2xl:pr-0"
        )}>
            <div class={classes!(
                "flex", "items-start", "justify-between",
                "gap-3"
            )}>
                <div class={classes!("flex", "flex-col", "gap-1")}>
                    <h1 class={classes!("font-semibold", "text-std-100")}>
                        {"Add to combo list"}
                    </h1>
                    <p class={classes!("text-xs", "text-std-400")}>
                        {"Click icons to add effects. Use the middle tray to remove."}
                    </p>
                </div>
            </div>

            <div class={classes!(
                "flex", "flex-col", "gap-4"
            )}>
                {(*abilities).clone()}
                {(*items).clone()}
                {(*runes).clone()}
                {(*other).clone()}
            </div>
        </div>
    }
}

#[derive(PartialEq, Properties)]
pub struct StackSelectorProps<T: Victim + PartialEq + 'static> {
    #[prop_or_default]
    pub callback: Option<Callback<usize>>,
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
        ref callback,
    } = *props;

    let stack = use_reducer(Stack::default);

    {
        let stack = stack.clone();
        let items_meta = items_meta.clone();
        let runes_meta = runes_meta.clone();

        type StackStore = HashMap<ChampionId, Vec<StackValue>>;

        use_effect_with(champion_id, move |_| {
            if let Some(window) = web_sys::window()
                && let Ok(local) = window.local_storage()
                && let Some(storage) = local
                && let Ok(store) = storage.get("stack")
                && let Some(value) = store
                && let Ok(de) = serde_json::from_str::<StackStore>(&value)
                && let Some(stored) = de.get(&champion_id)
            {
                let result = stored
                    .iter()
                    .map(|&v| StackEntry::new(v))
                    .collect::<Vec<_>>();
                return stack.dispatch(StackAction::Replace(Stack(result)));
            }

            stack.dispatch(StackAction::Replace(Stack::new(
                champion_id,
                &items_meta,
                &runes_meta,
            )));
        })
    }

    let save_stack = {
        use_callback(
            (stack.clone(), champion_id),
            move |_: MouseEvent, (stack, _)| {
                if let Some(window) = web_sys::window()
                    && let Ok(local) = window.local_storage()
                    && let Some(storage) = local
                {
                    let entries = stack.iter().map(|entry| entry.value).collect::<Vec<_>>();

                    let mut root: HashMap<ChampionId, Value> = storage
                        .get_item("stack")
                        .ok()
                        .flatten()
                        .and_then(|s| serde_json::from_str(&s).ok())
                        .unwrap_or_default();

                    root.insert(champion_id, serde_json::json!(entries));

                    if let Ok(result) = serde_json::to_string(&root) {
                        storage.set_item("stack", &result).log();
                    }
                }
            },
        )
    };

    let default_stack = {
        use_callback(
            (stack.clone(), items_meta.clone(), runes_meta.clone()),
            move |champion_id, (stack, items_meta, runes_meta)| {
                stack.dispatch(StackAction::Replace(Stack::new(
                    champion_id,
                    items_meta,
                    runes_meta,
                )));
            },
        )
    };

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
        use_callback((), move |_: MouseEvent, _| {
            stack.dispatch(StackAction::Clear)
        })
    };

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

    let safe_stack = stack.reconcile(champion_id, items_meta, runes_meta);
    let len = safe_stack.len();

    let remover = safe_stack
        .iter()
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

            let onclick = {
                let stack_remove = stack_remove.clone();
                Callback::from(move |_| stack_remove.emit(id))
            };

            html! {
                <button
                    key={id}
                    type={"button"}
                    class={classes!(
                        "group",
                        "relative",
                        "flex",
                        "items-center",
                        "justify-center",
                        "w-10",
                        "h-10",
                        "rounded-lg",
                        "border",
                        "border-std-800",
                        "bg-std-900/60",
                        "hover:bg-rose-500/10",
                        "hover:border-rose-500/75",
                        "transition-all",
                        "duration-150",
                        "focus-visible:outline",
                        "focus-visible:outline-2",
                        "focus-visible:outline-rose-400"
                    )}
                    {onclick}
                >
                    <div {data_offset}>
                        <Image
                            class={classes!("w-8", "h-8", "pointer-events-none")}
                            src={image_type}
                        />
                    </div>
                </button>
            }
        })
        .collect::<Html>();

    html! {
        <div class={classes!(
            "grid", "grid-cols-1",
            "items-start", "gap-4",
            "2xl:grid-cols-3"
        )}>
            <StackInsert
                callback={stack_push.clone()}
                items_meta={items_meta.clone()}
                runes_meta={runes_meta.clone()}
                {champion_id}
            />
            <div class={classes!(
                "flex", "flex-col", "py-4", "gap-4", "h-full", "px-4", "2xl:px-0"
            )}>
                <div class={classes!("flex", "items-center", "justify-between", "gap-3")}>
                    <div class={classes!("flex", "flex-col", "gap-1")}>
                        <div class={classes!("font-semibold", "text-std-100")}>
                            {"Defined combo"}
                        </div>
                        <div class={classes!("text-xs", "text-std-400")}>
                            {"Click icons to remove"}
                        </div>
                    </div>
                    <div class={classes!("flex", "items-center", "gap-2")}>
                        <span class={classes!(
                            "px-3", "py-1.5",
                            "rounded-lg",
                            "border", "border-std-800",
                            "bg-std-900/70",
                            "text-xs", "font-mono", "font-semibold",
                            "text-std-200"
                        )}>
                            {len}
                        </span>
                        <StackButton
                            onclick={save_stack}
                            class={classes!(
                                "hover:border-sky-500/75",
                                "focus-visible:outline-sky-400"
                            )}
                            title={"Save current combo definition for this champion"}
                            text={"Save"}
                        />
                        <StackButton
                            onclick={Callback::from(move |_| default_stack.emit(champion_id))}
                            class={classes!(
                                "hover:border-emerald-500/75",
                                "focus-visible:outline-emerald-400"
                            )}
                            title={"Add default combo definition for this champion"}
                            text={"Default"}
                        />
                        <StackButton
                            onclick={clear_stack}
                            text={"Clear"}
                            title={"Remove all selected items"}
                            class={classes!(
                                "hover:border-rose-500/75",
                                "focus-visible:outline-rose-400"
                            )}
                        />
                    </div>
                </div>
                <div class={classes!(
                    "min-h-32", "rounded-lg", "border", "border-dashed",
                    "border-std-800", "p-2", "h-full", "text-sm",
                    "text-std-400", "flex", "gap-2", "flex-wrap",
                    "content-start"
                )}>
                    {remover}
                </div>
            </div>
            <div class={classes!("overflow-auto")}>
                <StackTable<T>
                    callback={callback.clone()}
                    enemies={enemies.clone()}
                    stack={safe_stack}
                    {level}
                />
            </div>
        </div>
    }
}

#[derive(PartialEq, Properties)]
pub struct StackButtonProps {
    pub onclick: Callback<MouseEvent>,
    pub text: AttrValue,
    pub title: AttrValue,
    #[prop_or_default]
    pub class: Classes,
}

#[component]
pub fn StackButton(props: &StackButtonProps) -> Html {
    let StackButtonProps {
        onclick,
        text,
        title,
        class,
    } = props;

    let mut classes = classes!(
        "px-3",
        "py-1.5",
        "rounded-lg",
        "border",
        "border-std-800",
        "bg-std-900/70",
        "text-xs",
        "font-semibold",
        "text-std-200",
        "hover:bg-std-800/60",
        "transition-all",
        "duration-150",
        "focus-visible:outline",
        "focus-visible:outline-2",
    );

    classes.push(class);

    html! {
        <button
            type={"button"}
            {onclick}
            class={classes}
            {title}
            aria-label={title}
        >
            {text}
        </button>
    }
}
