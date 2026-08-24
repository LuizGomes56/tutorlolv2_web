use std::rc::Rc;
use tutorlolv2::{ChampionId, ItemId, RuneId, TypeMetadata};
use yew::prelude::*;

use crate::{
    components::{
        image::{Image, ImageType},
        stack::StackValue,
    },
    model::AbilityKind,
};

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
                "flex", "flex-wrap", "w-fit",
                "gap-2", "items-start",
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

    fn button(onclick: Callback<MouseEvent>, src: ImageType, max: bool) -> Html {
        html! {
            <button
                type={"button"}
                class={classes!("btn-stack", "group")}
                {onclick}
            >
                <Image
                    class={classes!(
                        "w-8", "h-8", "pointer-events-none",
                        "overflow-hidden", "rounded"
                    )}
                    {src}
                />
                if max {
                    <span class={classes!("img-letter", "text-sm", "z-10")}>
                        {"MAX"}
                    </span>
                }
            </button>
        }
    }

    let other = use_memo(callback.clone(), |callback| {
        let buttons = [
            (ImageType::BasicAttack, StackValue::BasicAttack, false),
            (ImageType::CritStrike, StackValue::CritStrike, false),
            (ImageType::OnhitAttack, StackValue::OnhitMin, false),
            (ImageType::OnhitAttack, StackValue::OnhitMax, true),
            (ImageType::Ignite, StackValue::Ignite, false),
        ]
        .into_iter()
        .map(|(src, v, max)| {
            let onclick = {
                let callback = callback.clone();
                Callback::from(move |_| callback.emit(v))
            };

            button(onclick, src, max)
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

                let onclick = {
                    let callback = callback.clone();
                    Callback::from(move |_| {
                        callback.emit(StackValue::Ability { slot, ability_id });
                    })
                };

                button(
                    onclick,
                    ImageType::Ability(champion_id, AbilityKind::Normal(ability_id)),
                    false,
                )
            });

        section("Abilities", buttons)
    });

    let items = use_memo(
        (callback.clone(), items_meta.clone()),
        |(callback, items_meta)| {
            section("Items", {
                let cursor = std::cell::Cell::new(0usize);

                items_meta.iter().map({
                    let callback = callback.clone();

                    move |metadata| {
                        let item_id = metadata.kind;
                        let has_max = item_id.deals_max_damage();

                        let base = cursor.get();
                        cursor.set(base + if has_max { 2 } else { 1 });

                        let onclick = |j: usize| {
                            let callback = callback.clone();
                            Callback::from(move |_| callback.emit(StackValue::Item(j, item_id)))
                        };

                        html! {
                            <>
                                {button(onclick(base), ImageType::from(item_id), false)}
                                if has_max {
                                    {button(onclick(base + 1), ImageType::from(item_id), true)}
                                }
                            </>
                        }
                    }
                })
            })
        },
    );

    let runes = use_memo(
        (callback.clone(), runes_meta.clone()),
        |(callback, runes_meta)| {
            section(
                "Runes",
                runes_meta.iter().enumerate().map(|(i, metadata)| {
                    let id = metadata.kind;

                    let onclick = {
                        let callback = callback.clone();
                        Callback::from(move |_| callback.emit(StackValue::Rune(i, id)))
                    };

                    button(onclick, ImageType::from(id), false)
                }),
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
                        {"Click icons to add effects"}
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
