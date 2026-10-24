use crate::{
    components::image::{Image, ImageType, Svg},
    hover::Docs,
    utils::tray::Tray,
};
use tutorlolv2::ValueId;
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct LoadoutProps<T: ValueId + PartialEq> {
    pub title: AttrValue,
    pub values: Tray<T>,
    pub onopen: Callback<MouseEvent>,
    pub onremove: Callback<u64>,
    #[prop_or(false)]
    pub runes: bool,
}

/// Shared inventory presentation in both calculator input panels.
#[component]
pub fn Loadout<T>(props: &LoadoutProps<T>) -> Html
where
    T: ValueId + PartialEq,
    ImageType: From<T>,
{
    let slots = props.values.iter().map(|entry| {
        let id = entry.id;
        let onremove = props.onremove.clone();
        let name = entry.value.name();
        html! {
            <button
                key={id}
                type="button"
                class="loadout-slot is-equipped"
                onclick={Callback::from(move |_| onremove.emit(id))}
                aria-label={format!("Remove {name}")}
                title={format!("{name} · Click to remove")}
                data_docs={Docs::global(entry.value).encode()}
            >
                <Image src={ImageType::from(entry.value)} />
                <span class="slot-remove" aria-hidden="true">{"×"}</span>
            </button>
        }
    });
    let empty_count = 6usize.saturating_sub(props.values.len());

    html! {
        <section class={classes!("loadout", props.runes.then_some("loadout-runes"))}>
            <button type="button" class="loadout-heading" onclick={props.onopen.clone()} aria-haspopup="dialog">
                <span>{&props.title}<span class="loadout-count">{props.values.len()}</span></span>
                <span class="loadout-edit">{"Edit"}<Svg src="/svgs/rchev.svg" /></span>
            </button>
            <div class="loadout-slots">
                {for slots}
                {for (0..empty_count).map(|i| html! {
                    <button
                        key={format!("empty-{i}")}
                        type="button"
                        class="loadout-slot is-empty"
                        onclick={props.onopen.clone()}
                        aria-label={format!("Add {}", if props.runes { "rune" } else { "item" })}
                        aria-haspopup="dialog"
                    ><span aria-hidden="true">{"+"}</span></button>
                })}
                if empty_count == 0 {
                    <button type="button" class="loadout-add" onclick={props.onopen.clone()} aria-haspopup="dialog">
                        {if props.runes { "+ Add rune" } else { "+ Add item" }}
                    </button>
                }
            </div>
        </section>
    }
}
