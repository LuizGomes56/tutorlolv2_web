use {
    crate::{
        components::{
            dynamic::Dynamic,
            errorlog::errorlog,
            image::{Image, ImageType},
            stack::{Stack, StackInsert, StackRemover, StackTable},
            tables::{body::DamageRenderer, header::TableHeader},
        },
        desktop::{use_overlay_focus, watch_overlay_change},
        hover::{self, Docs},
        livegame::{Enemy, Game},
        overlay::{
            model::{Panel, Widget},
            panel::PanelManager,
            storage::{initialize_panel_events, load_widgets, save_widgets},
        },
        utils::{Loading, Print, glue::get_desktop_data},
    },
    std::{rc::Rc, time::Duration},
    tutorlolv2::CtxVar,
    wasm_bindgen::{JsCast, JsValue, prelude::Closure},
    yew::{
        platform::{spawn_local, time::sleep},
        prelude::*,
    },
};

pub enum PanelAction {
    Enabled(Widget, bool),
    KeepVisible(Widget, bool),
    All(bool),
}

impl yew::Reducible for Panel {
    type Action = PanelAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut new = self;
        let panel = Rc::make_mut(&mut new);

        match action {
            PanelAction::Enabled(widget, value) => panel.enabled[widget.index()] = value,
            PanelAction::KeepVisible(widget, value) => panel.keep_visible[widget.index()] = value,
            PanelAction::All(value) => panel.enabled = [value; 5],
        }
        new
    }
}

#[component]
pub fn Overlay() -> Html {
    let game_data = use_state(|| Err::<Game, _>(Loading.into()));
    let enemy_index = use_state(|| 0);
    let enemy_count = use_state(|| 0);
    let (focused, focus_error) = use_overlay_focus();

    let latest_enemy_index = use_mut_ref(|| 0usize);
    let latest_enemy_count = use_mut_ref(|| 0usize);

    let stack = use_reducer(Stack::default);
    let stack_push = Stack::use_push(&stack);

    let panel_manager = use_reducer_eq(load_widgets);
    let widget_save_error = use_state(|| false);

    {
        let widget_save_error = widget_save_error.clone();

        use_effect_with(panel_manager.clone(), move |panels| {
            widget_save_error.set(!save_widgets(panels));
        });
    }

    use_effect_with((), |_| initialize_panel_events());

    {
        let latest_enemy_index = latest_enemy_index.clone();
        let latest_enemy_count = latest_enemy_count.clone();
        let enemy_index = enemy_index.clone();
        let enemy_count = enemy_count.clone();

        use_effect_with((enemy_index.clone(), enemy_count.clone()), move |_| {
            *latest_enemy_index.borrow_mut() = *enemy_index;
            *latest_enemy_count.borrow_mut() = *enemy_count;
        });
    }

    {
        let latest_enemy_index = latest_enemy_index.clone();
        let latest_enemy_count = latest_enemy_count.clone();
        let enemy_index = enemy_index.clone();

        use_effect_with((), move |_| {
            let callback = Closure::<dyn FnMut()>::new(move || {
                let count = *latest_enemy_count.borrow();

                if count > 0 {
                    enemy_index.set((*latest_enemy_index.borrow() + 1) % count);
                }
            });

            let failure = Closure::<dyn FnMut(String)>::new(move |value: String| {
                value.log();
            });

            let stop = watch_overlay_change(
                callback.as_ref().unchecked_ref(),
                failure.as_ref().unchecked_ref(),
            );

            move || {
                let _ = stop.call0(&JsValue::NULL);
                drop((callback, failure));
            }
        });
    }

    {
        let game_data = game_data.clone();
        let enemy_count = enemy_count.clone();

        use_effect_with((), |_| {
            spawn_local(async move {
                loop {
                    let data = get_desktop_data().await;

                    if let Ok(ref game) = data {
                        enemy_count.set(game.enemies.len());
                    }

                    game_data.set(data);

                    sleep(Duration::from_secs(1)).await;
                }
            });
        });
    }

    let data = match &*game_data {
        Ok(data) => {
            let Game {
                current_player,
                enemies,
                items_meta,
                runes_meta,
                ..
            } = data;

            let champion_id = current_player.champion_id;
            let enemy = enemies.get(*enemy_index).or_else(|| enemies.first());

            let damages = || {
                enemy
                    .map(|enemy| {
                        let damages = DamageRenderer::new(
                            &enemy.damages,
                            champion_id,
                            current_player.adaptive_type,
                            items_meta,
                            runes_meta,
                            None,
                        );

                        let enemy_id = enemy.champion_id;

                        html! {
                            <tr>
                                <td
                                    data_idents={hover::encode_idents(
                                        &CtxVar::ENEMY_VARS,
                                        &enemy.damages.ctx,
                                    )}
                                    data_docs={Docs::global(enemy_id).encode()}
                                    class={classes!("w-12")}
                                >
                                    <Image src={ImageType::from(enemy_id)} />
                                </td>
                                {damages}
                            </tr>
                        }
                    })
                    .unwrap_or_default()
            };

            let recommendation = || {
                enemy.map(|enemy| {
                    let base = enemy.total_damage();

                    enemy
                        .item_scores()
                        .into_iter()
                        .map(|(damage, item)| {
                            html! {
                                <div class={classes!("flex", "items-center", "gap-2")}>
                                    <span class={classes!("text-sm")}>
                                        {damage - base}
                                    </span>
                                    <Image
                                        class={classes!("w-6", "h-6")}
                                        src={ImageType::from(item)}
                                    />
                                </div>
                            }
                        })
                        .collect::<Html>()
                })
            };

            html! {
                <>
                    if panel_manager.visible(Widget::Damage, focused) {
                        <Dynamic
                            key={"damage-table"}
                            panel_id={"damage-table"}
                            {focused}
                        >
                            <div class={classes!("overflow-auto", "w-fit", "origin-top-left")}>
                                <table class={classes!("data-table", "overlay")}>
                                    <TableHeader
                                        {champion_id}
                                        {items_meta}
                                        {runes_meta}
                                    />
                                    <tbody>{damages()}</tbody>
                                </table>
                            </div>
                        </Dynamic>
                    }

                    if panel_manager.visible(Widget::Recommendations, focused) {
                        <Dynamic
                            key={"recommendations-table"}
                            panel_id={"recommendations-table"}
                            {focused}
                        >
                            <div class={classes!(
                                "grid", "grid-cols-6", "gap-1", "max-w-fit", "max-h-fit"
                            )}>
                                {recommendation()}
                            </div>
                        </Dynamic>
                    }

                    if panel_manager.visible(Widget::StackInsert, focused) {
                        <Dynamic
                            key={"stack-insert"}
                            panel_id={"stack-insert"}
                            {focused}
                        >
                            <div class={classes!("overlay-panel-editor")}>
                                <StackInsert
                                    callback={stack_push}
                                    {items_meta}
                                    {runes_meta}
                                    {champion_id}
                                />
                            </div>
                        </Dynamic>
                    }

                    if panel_manager.visible(Widget::StackRemover, focused) {
                        <Dynamic
                            key={"stack-remover"}
                            panel_id={"stack-remover"}
                            {focused}
                        >
                            <div class={classes!("overlay-panel-editor")}>
                                <StackRemover
                                    stack={&stack}
                                    {champion_id}
                                    {items_meta}
                                    {runes_meta}
                                    hmax={false}
                                />
                            </div>
                        </Dynamic>
                    }

                    if panel_manager.visible(Widget::Stacks, focused) {
                        <Dynamic
                            key={"stack-table"}
                            panel_id={"stack-table"}
                            {focused}
                        >
                            <div class={classes!("min-h-fit", "min-w-min")}>
                                <StackTable<Enemy>
                                    index={*enemy_index}
                                    {enemies}
                                    stack={stack.reconcile(champion_id, items_meta, runes_meta)}
                                    level={current_player.level}
                                />
                            </div>
                        </Dynamic>
                    }
                </>
            }
        }
        Err(e) => {
            e.log();

            html! {
                if focused {
                    <div class={classes!(
                        "absolute", "top-1/2", "left-1/2",
                        "-translate-x-1/2", "-translate-y-1/2"
                    )}>
                        {errorlog(e)}
                    </div>
                }
            }
        }
    };

    html! {
        <div class={classes!(
            "desktop-overlay",
            "flex", "flex-col", "gap-4",
            "overflow-hidden", "flex-1",
            "h-full", "w-full",
            if focused { "bg-black/25" } else { "bg-transparent" },
        )}>
            if focused {
                <Dynamic
                    key={"panel-manager"}
                    panel_id={"panel-manager"}
                    resize={false}
                    {focused}
                >
                    <div class={classes!("w-fit")}>
                        <PanelManager handler={&panel_manager} save_error={*widget_save_error} />

                        if !focus_error.is_empty() {
                            <p role={"alert"}>{&focus_error}</p>
                        }
                    </div>
                </Dynamic>
            }
            {data}
        </div>
    }
}
