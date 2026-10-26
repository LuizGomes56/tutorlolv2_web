use crate::{
    desktop::{ShortcutSettings, close_overlay},
    overlay::{
        model::{Panel, Widget},
        page::PanelAction,
    },
};
use strum::VariantArray;
use web_sys::HtmlInputElement;
use yew::{platform::spawn_local, prelude::*};

#[derive(PartialEq, Properties)]
pub struct PanelProps {
    pub handler: UseReducerHandle<Panel>,
    pub save_error: bool,
}

#[component]
pub fn PanelManager(props: &PanelProps) -> Html {
    let error = use_state(String::new);

    let all = {
        let handler = props.handler.clone();

        Callback::from(move |enabled| handler.dispatch(PanelAction::All(enabled)))
    };

    let close = {
        let error = error.clone();

        Callback::from(move |_| {
            let error = error.clone();

            spawn_local(async move {
                if let Err(value) = close_overlay().await {
                    error.set(format!("Could not close overlay: {value:?}"));
                }
            });
        })
    };

    html! {
        <section class={classes!("overlay-widget-settings")} aria-label={"Overlay widgets"}>
            <div class={classes!("overlay-settings-heading")}>
                <h2>{"Overlay widgets"}</h2>
                <button
                    type={"button"}
                    class={classes!("overlay-settings-button", "primary")}
                    onclick={close}
                >
                    {"Done"}
                </button>
            </div>
            <p>{"Drag a widget to move it. Select it to reveal its resize handles."}</p>
            <table class={classes!("overlay-widget-table")}>
                <thead>
                    <tr>
                        <th scope={"col"}>{"Widget"}</th>
                        <th scope={"col"}>{"Show"}</th>
                        <th scope={"col"}>{"Keep visible when closed"}</th>
                    </tr>
                </thead>
                <tbody>
                    {for Widget::VARIANTS.iter().map(|&widget| {
                        let handler = props.handler.clone();

                        let enabled = Callback::from(move |event: Event| {
                            handler.dispatch(PanelAction::Enabled(
                                widget,
                                event.target_unchecked_into::<HtmlInputElement>().checked(),
                            ));
                        });

                        let handler = props.handler.clone();

                        let keep = Callback::from(move |event: Event| {
                            handler.dispatch(PanelAction::KeepVisible(
                                widget,
                                event.target_unchecked_into::<HtmlInputElement>().checked(),
                            ));
                        });

                        html! {
                            <tr key={widget.index()}>
                                <th scope={"row"}>{widget.label()}</th>
                                <td>
                                    <input
                                        type={"checkbox"}
                                        aria-label={format!("Show {}", widget.label())}
                                        checked={props.handler.enabled[widget.index()]}
                                        onchange={enabled}
                                    />
                                </td>
                                <td>
                                    <input
                                        type={"checkbox"}
                                        aria-label={format!("Keep {} visible when closed", widget.label())}
                                        checked={props.handler.keep_visible[widget.index()]}
                                        disabled={!props.handler.enabled[widget.index()]}
                                        onchange={keep}
                                    />
                                </td>
                            </tr>
                        }
                    })}
                </tbody>
            </table>
            <div class={classes!("overlay-shortcut-controls")}>
                <button
                    type={"button"}
                    class={classes!("overlay-settings-button")}
                    onclick={all.reform(|_| true)}
                >
                    {"Show all"}
                </button>
                <button
                    type={"button"}
                    class={classes!("overlay-settings-button")}
                    onclick={all.reform(|_| false)}
                >
                    {"Hide all"}
                </button>
            </div>

            if props.save_error {
                <p role={"alert"} class={classes!("overlay-settings-error")}>
                    {"Widget preferences could not be saved. Allow local storage to keep your choices after restarting."}
                </p>
            }

            if !error.is_empty() {
                <p role={"alert"} class={classes!("overlay-settings-error")}>
                    {&*error}
                </p>
            }

            <ShortcutSettings />
        </section>
    }
}
