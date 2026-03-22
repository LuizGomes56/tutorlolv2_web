use crate::{
    components::{
        image::{Image, ImageType},
        stack::{Stack, StackValue, Tray, TrayAction, TrayEntry},
    },
    utils::{Print, encode_offset},
};
use serde_json::Value;
use std::{collections::HashMap, rc::Rc};
use tutorlolv2_gen::{
    BASIC_ATTACK_OFFSET, CRITICAL_STRIKE_OFFSET, CastId, ChampionId, IGNITE_OFFSET, ItemId,
    ONHIT_EFFECT_OFFSET, RuneId, TypeMetadata,
};
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct StackRemoverProps {
    pub stack: UseReducerHandle<Stack>,
    pub items_meta: Rc<[TypeMetadata<ItemId>]>,
    pub runes_meta: Rc<[TypeMetadata<RuneId>]>,
    pub champion_id: ChampionId,
    #[prop_or(true)]
    pub hmax: bool,
}

#[component]
pub fn StackRemover(props: &StackRemoverProps) -> Html {
    let StackRemoverProps {
        ref stack,
        ref items_meta,
        ref runes_meta,
        champion_id,
        hmax,
    } = *props;

    let combo_index = use_state(|| 0);

    {
        let stack = stack.clone();
        let items_meta = items_meta.clone();
        let runes_meta = runes_meta.clone();
        let combo_index = combo_index.clone();

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
                let values = Tray::new(
                    stored
                        .iter()
                        .map(|&v| TrayEntry::new(v))
                        .collect::<Vec<_>>(),
                );

                return stack.dispatch(TrayAction::Replace(Stack {
                    champion_id,
                    values,
                }));
            }

            stack.dispatch(TrayAction::Replace(Stack::new(
                &combo_index,
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
            (
                stack.clone(),
                combo_index,
                items_meta.clone(),
                runes_meta.clone(),
                champion_id,
            ),
            move |champion_id, (stack, combo_index, items_meta, runes_meta, ..)| {
                stack.dispatch(TrayAction::Replace(Stack::new(
                    combo_index,
                    champion_id,
                    items_meta,
                    runes_meta,
                )));
                combo_index.set(**combo_index + 1);
            },
        )
    };

    let stack_remove = {
        let stack = stack.clone();
        use_callback((), move |id, _| stack.dispatch(TrayAction::RemoveById(id)))
    };

    let clear_stack = {
        let stack = stack.clone();
        use_callback((), move |_: MouseEvent, _| {
            stack.dispatch(TrayAction::Clear)
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
                    stack.dispatch(TrayAction::Replace(next));
                }
            },
        );
    }

    let safe_stack = stack.reconcile(champion_id, items_meta, runes_meta);
    let len = safe_stack.len();

    let remover = safe_stack
        .iter()
        .map(|entry| {
            let (image_type, offset, max) = match entry.value {
                StackValue::Ability { slot, ability_id } => (
                    ImageType::Ability(champion_id, ability_id.into()),
                    champion_id.get_ability_formula(slot),
                    false,
                ),
                StackValue::Item(i, item_id) => (
                    ImageType::from(item_id),
                    item_id.formula(),
                    i % 2 == 1 && item_id.deals_max_damage(),
                ),
                StackValue::Rune(_, rune_id) => {
                    (ImageType::from(rune_id), rune_id.formula(), false)
                }
                StackValue::BasicAttack => (ImageType::BasicAttack, &BASIC_ATTACK_OFFSET, false),
                StackValue::CritStrike => (ImageType::CritStrike, &CRITICAL_STRIKE_OFFSET, false),
                StackValue::OnhitMin => (ImageType::OnhitAttack, &ONHIT_EFFECT_OFFSET, false),
                StackValue::OnhitMax => (ImageType::OnhitAttack, &ONHIT_EFFECT_OFFSET, true),
                StackValue::Ignite => (ImageType::Ignite, &IGNITE_OFFSET, false),
            };

            let data_offset = encode_offset(&[offset]);
            let id = entry.id;

            let onclick = {
                let stack_remove = stack_remove.clone();
                Callback::from(move |_| stack_remove.emit(id))
            };

            let class = classes!(
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
            );

            let inner = html! {
                <>
                    <Image
                        class={classes!("w-8", "h-8", "pointer-events-none")}
                        src={image_type}
                    />
                    if max {
                        <span class={classes!("img-letter", "text-sm", "z-10")}>
                            {"MAX"}
                        </span>
                    }
                </>
            };

            html! {
                <button
                    key={id}
                    type={"button"}
                    {class}
                    {onclick}
                    {data_offset}
                >
                    {inner}
                </button>
            }
        })
        .collect::<Html>();

    html! {
        <div class={classes!(
            "flex", "flex-col", "py-4", "min-h-0", "gap-4",
            "px-4", "2xl:px-0", hmax.then_some("h-full")
        )}>
            <div class={classes!("flex", "justify-between", "gap-3")}>
                <div class={classes!("flex", "flex-col", "gap-1")}>
                    <span class={classes!("whitespace-nowrap", "font-semibold", "text-std-100")}>
                        {"Defined combo"}
                    </span>
                    <span class={classes!("whitespace-nowrap", "text-xs", "text-std-400")}>
                        {"Click icons to remove"}
                    </span>
                </div>
                <div class={classes!("grid", "grid-cols-2", "gap-2")}>
                    <span class={classes!(
                        "px-3", "py-1.5",
                        "rounded-lg",
                        "border", "border-std-800",
                        "bg-std-900/70",
                        "text-xs", "font-mono", "font-semibold",
                        "text-std-200", "text-center"
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
                "flex-1", "w-full",
                "rounded-lg", "border", "border-dashed",
                "border-std-800", "p-2",
                "text-sm", "text-std-400",
                "flex", "gap-2", "flex-wrap",
                "content-start", "overflow-auto",
                hmax.then_some("h-full")
            )}>
                {remover}
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
