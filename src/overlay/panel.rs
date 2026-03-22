// use crate::overlay::page::{Panel, PanelAction};
// use web_sys::HtmlInputElement;
// use yew::prelude::*;

// #[derive(PartialEq, Properties)]
// pub struct PanelCheckboxProps {
//     pub onchange: Callback<Event>,
//     pub tag: AttrValue,
//     pub checked: bool,
// }

// #[component]
// pub fn PanelCheckbox(props: &PanelCheckboxProps) -> Html {
//     let PanelCheckboxProps {
//         ref onchange,
//         ref tag,
//         checked,
//     } = *props;

//     html! {
//         <label
//             title={tag}
//             class={classes!(
//                 "rounded", "p-4",
//                 match checked {
//                     true => classes!("bg-std-800"),
//                     false => classes!("bg-std-900"),
//                 }
//             )}>
//             <span>{"?"}</span>
//             <input
//                 type={"checkbox"}
//                 class={classes!("absolute", "w-full", "h-full", "opacity-0")}
//                 {checked}
//                 {onchange}
//             />
//         </label>
//     }
// }

// #[derive(PartialEq, Properties)]
// pub struct PanelProps {
//     pub handler: UseReducerHandle<Panel>,
// }

// #[component]
// pub fn PanelManager(props: &PanelProps) -> Html {
//     let PanelProps { handler } = props;

//     #[hook]
//     fn use_panel(handler: &UseReducerHandle<Panel>, f: fn(bool) -> PanelAction) -> Callback<Event> {
//         let handler = handler.clone();
//         use_callback((), move |e: Event, _| {
//             let target = e.target_unchecked_into::<HtmlInputElement>();
//             handler.dispatch(f(target.checked()));
//         })
//     }

//     let damage_table = use_panel(handler, PanelAction::DamageTable);
//     let recommendations_table = use_panel(handler, PanelAction::RecommendationsTable);
//     let stack_insert = use_panel(handler, PanelAction::StackInsert);
//     let stack_remover = use_panel(handler, PanelAction::StackRemover);
//     let stack_table = use_panel(handler, PanelAction::StackTable);

//     html! {
//         <div class={classes!("grid", "grid-cols-5", "gap-2", "w-fit")}>
//             <PanelCheckbox
//                 tag={"Damage table"}
//                 onchange={damage_table}
//                 checked={handler.damage_table}
//             />
//             <PanelCheckbox
//                 tag={"Recommendations table"}
//                 onchange={recommendations_table}
//                 checked={handler.recommendations_table}
//             />
//             <PanelCheckbox
//                 tag={"Stack insert"}
//                 onchange={stack_insert}
//                 checked={handler.stack_insert}
//             />
//             <PanelCheckbox
//                 tag={"Stack remover"}
//                 onchange={stack_remover}
//                 checked={handler.stack_remover}
//             />
//             <PanelCheckbox
//                 tag={"Stack table"}
//                 onchange={stack_table}
//                 checked={handler.stack_table}
//             />
//         </div>
//     }
// }

use crate::overlay::page::{Panel, PanelAction};
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
struct PanelItem {
    label: AttrValue,
    tag: AttrValue,
    placeholder: AttrValue,
    checked: bool,
    ontoggle: Callback<bool>,
}

fn panel_toggle(
    handler: &UseReducerHandle<Panel>,
    action: fn(bool) -> PanelAction,
) -> Callback<bool> {
    let handler = handler.clone();

    Callback::from(move |checked: bool| {
        handler.dispatch(action(checked));
    })
}

fn set_all_panels(handler: &UseReducerHandle<Panel>, checked: bool) {
    handler.dispatch(PanelAction::DamageTable(checked));
    handler.dispatch(PanelAction::RecommendationsTable(checked));
    handler.dispatch(PanelAction::StackInsert(checked));
    handler.dispatch(PanelAction::StackRemover(checked));
    handler.dispatch(PanelAction::StackTable(checked));
}

#[derive(PartialEq, Properties)]
pub struct PanelCheckboxProps {
    pub ontoggle: Callback<bool>,
    pub label: AttrValue,
    pub tag: AttrValue,
    pub placeholder: AttrValue,
    pub checked: bool,
}

#[component]
pub fn PanelCheckbox(props: &PanelCheckboxProps) -> Html {
    let onchange = {
        let ontoggle = props.ontoggle.clone();

        Callback::from(move |e: Event| {
            let input = e.target_unchecked_into::<HtmlInputElement>();
            ontoggle.emit(input.checked());
        })
    };

    html! {
        <label
            title={props.tag.clone()}
            class={classes!(
                "group",
                "relative",
                "flex",
                "h-24",
                "w-20",
                "shrink-0",
                "cursor-pointer",
                "select-none",
                "flex-col",
                "items-center",
                "justify-center",
                "gap-2",
                "overflow-hidden",
                "border",
                "p-2",
                "text-center",
                "transition-all",
                "duration-150",
                "hover:shadow-lg",
                if props.checked {
                    "border-sky-400/50 bg-std-800"
                } else {
                    "border-white/10 bg-std-900 hover:border-white/20 hover:bg-std-800"
                }
            )}
        >
            <input
                type={"checkbox"}
                class={classes!("absolute", "inset-0", "z-10", "cursor-pointer", "opacity-0")}
                checked={props.checked}
                aria-label={props.tag.clone()}
                {onchange}
            />

            <span
                class={classes!(
                    "pointer-events-none",
                    "absolute",
                    "right-2",
                    "top-2",
                    "h-2.5",
                    "w-2.5",
                    "rounded-full",
                    if props.checked {
                        "bg-emerald-400"
                    } else {
                        "bg-white/20"
                    }
                )}
            />

            <div
                class={classes!(
                    "pointer-events-none",
                    "flex",
                    "h-10",
                    "w-10",
                    "items-center",
                    "justify-center",
                    "text-xs",
                    "font-bold",
                    "tracking-wide",
                    if props.checked {
                        "bg-sky-500/20 text-sky-100"
                    } else {
                        "bg-white/5 text-white/70"
                    }
                )}
            >
                {props.placeholder.clone()}
            </div>

            <div class={classes!("pointer-events-none", "text-xs", "leading-tight", "text-white/90")}>
                {props.label.clone()}
            </div>
        </label>
    }
}

#[derive(PartialEq, Properties)]
pub struct PanelProps {
    pub handler: UseReducerHandle<Panel>,
}

#[component]
pub fn PanelManager(props: &PanelProps) -> Html {
    let handler = props.handler.clone();

    let damage_table = panel_toggle(&handler, PanelAction::DamageTable);
    let recommendations_table = panel_toggle(&handler, PanelAction::RecommendationsTable);
    let stack_insert = panel_toggle(&handler, PanelAction::StackInsert);
    let stack_remover = panel_toggle(&handler, PanelAction::StackRemover);
    let stack_table = panel_toggle(&handler, PanelAction::StackTable);

    let enable_all = {
        let handler = handler.clone();
        Callback::from(move |_| set_all_panels(&handler, true))
    };

    let disable_all = {
        let handler = handler.clone();
        Callback::from(move |_| set_all_panels(&handler, false))
    };

    let active_count = [
        handler.damage_table,
        handler.recommendations_table,
        handler.stack_insert,
        handler.stack_remover,
        handler.stack_table,
    ]
    .into_iter()
    .filter(|checked| *checked)
    .count();

    let items = [
        PanelItem {
            label: "Damage".into(),
            tag: "Damage table".into(),
            placeholder: "DMG".into(),
            checked: handler.damage_table,
            ontoggle: damage_table,
        },
        PanelItem {
            label: "Recs".into(),
            tag: "Recommendations table".into(),
            placeholder: "REC".into(),
            checked: handler.recommendations_table,
            ontoggle: recommendations_table,
        },
        PanelItem {
            label: "Add".into(),
            tag: "Stack insert".into(),
            placeholder: "ADD".into(),
            checked: handler.stack_insert,
            ontoggle: stack_insert,
        },
        PanelItem {
            label: "Remove".into(),
            tag: "Stack remover".into(),
            placeholder: "DEL".into(),
            checked: handler.stack_remover,
            ontoggle: stack_remover,
        },
        PanelItem {
            label: "Stacks".into(),
            tag: "Stack table".into(),
            placeholder: "STK".into(),
            checked: handler.stack_table,
            ontoggle: stack_table,
        },
    ];

    html! {
        <section
            class={classes!(
                "w-fit",
                "p-4",
                "shadow-2xl",
                "backdrop-blur-md"
            )}
        >
            <div class={classes!("mb-3", "flex", "items-center", "justify-between", "gap-3")}>
                <div class={classes!("min-w-0")}>
                    <div class={classes!("text-xs", "font-semibold", "uppercase", "tracking-wide", "text-white/50")}>
                        {"Overlay widgets"}
                    </div>
                    <div class={classes!("text-sm", "text-white/90")}>
                        {format!("{active_count}/5 active")}
                    </div>
                </div>
                <div class={classes!("flex", "items-center", "gap-2")}>
                    <button
                        type={"button"}
                        class={classes!(
                            "border",
                            "border-white/10",
                            "bg-white/5",
                            "px-3",
                            "py-2",
                            "text-xs",
                            "text-white/80",
                            "transition",
                            "hover:bg-white/10"
                        )}
                        onclick={enable_all}
                    >
                        {"Show All"}
                    </button>

                    <button
                        type={"button"}
                        class={classes!(
                            "border",
                            "border-white/10",
                            "bg-white/5",
                            "px-3",
                            "py-2",
                            "text-xs",
                            "text-white/80",
                            "transition",
                            "hover:bg-white/10"
                        )}
                        onclick={disable_all}
                    >
                        {"Hide All"}
                    </button>
                </div>
            </div>
            <div
                class={classes!("flex", "flex-wrap", "gap-2")}
                role={"group"}
                aria-label={"Overlay widgets"}
            >
                {
                    for items.into_iter().map(|item| {
                        html! {
                            <PanelCheckbox
                                label={item.label}
                                tag={item.tag}
                                placeholder={item.placeholder}
                                checked={item.checked}
                                ontoggle={item.ontoggle}
                            />
                        }
                    })
                }
            </div>
        </section>
    }
}
