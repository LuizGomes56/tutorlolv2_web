use crate::components::image::Svg;
use web_sys::{HtmlDialogElement, HtmlInputElement};
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct SelectorDialogProps {
    pub title: AttrValue,
    pub kind: AttrValue,
    pub is_open: UseStateHandle<bool>,
    pub children: Children,
    #[prop_or_default]
    pub oninteraction: Callback<MouseEvent>,
}

/// Native modal behavior supplies focus trapping, inert background and focus restoration.
#[component]
pub fn SelectorDialog(props: &SelectorDialogProps) -> Html {
    let dialog_ref = use_node_ref();
    {
        let dialog_ref = dialog_ref.clone();
        use_effect_with(*props.is_open, move |is_open| {
            if let Some(dialog) = dialog_ref.cast::<HtmlDialogElement>() {
                if *is_open && !dialog.open() {
                    let _ = dialog.show_modal();
                } else if !*is_open && dialog.open() {
                    dialog.close();
                }
            }
        });
    }
    let close = {
        let is_open = props.is_open.clone();
        Callback::from(move |_| is_open.set(false))
    };
    let onclose = {
        let is_open = props.is_open.clone();
        Callback::from(move |_: Event| is_open.set(false))
    };
    let onclick = {
        let is_open = props.is_open.clone();
        let dialog_ref = dialog_ref.clone();
        let oninteraction = props.oninteraction.clone();
        Callback::from(move |event: MouseEvent| {
            oninteraction.emit(event.clone());
            // Yew delegates events, so current_target is the app root.
            if let Some(dialog) = dialog_ref.cast::<HtmlDialogElement>()
                && event.target() == Some(dialog.into())
            {
                is_open.set(false);
            }
        })
    };
    let title_id = format!("{}-selector-title", props.kind);
    html! {
        <dialog
            ref={dialog_ref}
            class={classes!("selector-dialog", format!("selector-{}", props.kind))}
            aria-labelledby={title_id.clone()}
            {onclose}
            {onclick}
        >
            <div class="selector-shell">
                <header class="selector-header">
                    <div class="selector-title-group">
                        <h2 id={title_id}>{&props.title}</h2>
                    </div>
                    <span class="selector-header-hint"><kbd>{"Shift"}</kbd>{" + hover for details"}</span>
                    <button type="button" class="selector-close" onclick={close.clone()} aria-label="Close selector">{"×"}</button>
                </header>
                {for props.children.iter()}
                <footer class="selector-footer">
                    <span class="selector-key-hint"><kbd>{"Esc"}</kbd>{" to close"}</span>
                    <button type="button" class="selector-done" onclick={close}>{"Done"}</button>
                </footer>
            </div>
        </dialog>
    }
}

#[derive(PartialEq, Properties)]
pub struct SearchProps {
    pub query: UseStateHandle<String>,
    pub placeholder: AttrValue,
}

#[component]
pub fn Search(props: &SearchProps) -> Html {
    let oninput = {
        let query = props.query.clone();
        Callback::from(move |e: InputEvent| {
            query.set(e.target_unchecked_into::<HtmlInputElement>().value())
        })
    };
    let onclear = {
        let query = props.query.clone();
        Callback::from(move |_| query.set(String::new()))
    };
    html! {
        <label class="selector-search">
            <Svg src="/svgs/search.svg" />
            <input type="search" value={(*props.query).clone()} {oninput} placeholder={props.placeholder.clone()} aria-label={props.placeholder.clone()} autofocus={true} autocomplete="off" />
            if !props.query.is_empty() {
                <button type="button" onclick={onclear} aria-label="Clear search">{"×"}</button>
            }
        </label>
    }
}
