use {
    crate::{
        components::{
            dynamic::Dynamic,
            errorlog::errorlog,
            image::{Image, ImageType},
            stack::{Stack, StackInsert, StackRemover, StackTable},
            tables::{body::DamageRenderer, header::TableHeader},
        },
        hover::Docs,
        livegame::{Enemy, Game},
        overlay::{panel::PanelManager, storage::initialize_panel_events},
        utils::{Loading, Print, glue::get_desktop_data, hooks::on_keydown},
    },
    std::{rc::Rc, time::Duration},
    tutorlolv2::yew::ReduceApply,
    wasm_bindgen::{
        JsCast, JsValue,
        prelude::{Closure, wasm_bindgen},
    },
    web_sys::js_sys::Function,
    yew::{
        platform::{spawn_local, time::sleep},
        prelude::*,
    },
};

#[wasm_bindgen(module = "/public/invoke.js")]
unsafe extern "C" {
    #[wasm_bindgen(js_name = "blur_overlay")]
    pub fn blur_overlay();

    #[wasm_bindgen(js_name = "listen", catch)]
    pub async fn listen(event: String, callback: &Function) -> Result<JsValue, JsValue>;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Panel {
    pub damage_table: bool,
    pub recommendations_table: bool,
    pub stack_insert: bool,
    pub stack_remover: bool,
    pub stack_table: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PanelAction {
    DamageTable(bool),
    RecommendationsTable(bool),
    StackInsert(bool),
    StackRemover(bool),
    StackTable(bool),
}

impl tutorlolv2::yew::ReduceApply for Panel {
    type Action = PanelAction;

    fn apply(&mut self, action: Self::Action) {
        match action {
            Self::Action::DamageTable(value) => self.damage_table = value,
            Self::Action::RecommendationsTable(value) => self.recommendations_table = value,
            Self::Action::StackInsert(value) => self.stack_insert = value,
            Self::Action::StackRemover(value) => self.stack_remover = value,
            Self::Action::StackTable(value) => self.stack_table = value,
        }
    }
}

impl yew::Reducible for Panel {
    type Action = PanelAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut new = self;
        let new_mut = Rc::make_mut(&mut new);

        new_mut.apply(action);
        new
    }
}

impl Default for Panel {
    fn default() -> Self {
        Self {
            damage_table: true,
            recommendations_table: true,
            stack_insert: true,
            stack_remover: true,
            stack_table: true,
        }
    }
}

#[component]
pub fn Overlay() -> Html {
    let game_data = use_state(|| Err::<Game, _>(Loading.into()));
    let enemy_index = use_state(|| 0);
    let enemy_count = use_state(|| 0);
    let focused = use_state(|| false);

    let latest_enemy_index = use_mut_ref(|| 0usize);
    let latest_enemy_count = use_mut_ref(|| 0usize);
    let change_unlisten = use_mut_ref(|| None::<Function>);
    let change_callback = use_mut_ref(|| None::<Closure<dyn FnMut(bool)>>);

    let stack = use_reducer(Stack::default);
    let stack_push = Stack::use_push(&stack);

    let panel_manager = use_reducer_eq(Panel::default);

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
        let change_unlisten = change_unlisten.clone();
        let change_callback = change_callback.clone();

        use_effect_with((), move |_| {
            {
                let change_unlisten = change_unlisten.clone();
                let change_callback = change_callback.clone();
                spawn_local(async move {
                    let callback = Closure::wrap(Box::new(move |_| {
                        let count = *latest_enemy_count.borrow();
                        if count == 0 {
                            return;
                        }

                        let current = *latest_enemy_index.borrow();
                        let new = (current + 1) % count;
                        enemy_index.set(new);
                    }) as Box<dyn FnMut(bool)>);

                    let js_fn = callback.as_ref().unchecked_ref::<Function>().clone();

                    match listen("change".to_string(), &js_fn).await {
                        Ok(value) if !value.is_null() && !value.is_undefined() => {
                            if let Ok(unlisten) = value.dyn_into::<Function>() {
                                *change_unlisten.borrow_mut() = Some(unlisten);
                            }
                        }
                        _ => {}
                    }

                    *change_callback.borrow_mut() = Some(callback);
                })
            };

            move || {
                if let Some(unlisten) = change_unlisten.borrow_mut().take() {
                    let _ = unlisten.call0(&JsValue::NULL);
                }

                change_callback.borrow_mut().take();
            }
        });
    }

    {
        let focused = focused.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                let callback =
                    Closure::wrap(Box::new(move |v| focused.set(v)) as Box<dyn FnMut(bool)>);
                let _ = listen("focus".to_string(), callback.as_ref().unchecked_ref()).await;
                callback.forget();
            });
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

    {
        let focused = focused.clone();

        use_effect_with((), move |_| {
            on_keydown(27, move || {
                focused.set(false);
                blur_overlay();
            })
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
                    let damages =
                        DamageRenderer::new(&enemy.damages, champion_id, items_meta, runes_meta, None);

                    let enemy_id = enemy.champion_id;

                    html! {
                        <tr>
                            <td data_docs={Docs::global(enemy_id).encode()} class={classes!("w-12")}>
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
                    if *focused {
                        <Dynamic panel_id={"panel-manager"} resize={false} focused={*focused}>
                            <div data-panel-content={true}>
                                <PanelManager handler={&panel_manager} />
                            </div>
                        </Dynamic>
                    }
                    if panel_manager.damage_table {
                        <Dynamic panel_id={"damage-table"} focused={*focused}>
                            <div
                                data-panel-content={true}
                                class={classes!("overflow-auto", "w-fit", "origin-top-left")}
                            >
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
                    if panel_manager.recommendations_table {
                        <Dynamic panel_id={"recommendations-table"} focused={*focused}>
                            <div
                                data-panel-content={true}
                                class={classes!("flex", "flex-col", "gap-1", "max-w-fit", "max-h-fit")}
                            >
                                {recommendation()}
                            </div>
                        </Dynamic>
                    }
                    if *focused {
                        if panel_manager.stack_insert {
                            <Dynamic panel_id={"stack-insert"} resize={false} focused={*focused}>
                                <div data-panel-content={true}>
                                    <StackInsert
                                        callback={stack_push}
                                        {items_meta}
                                        {runes_meta}
                                        {champion_id}
                                    />
                                </div>
                            </Dynamic>
                        }
                        if panel_manager.stack_remover {
                            <Dynamic panel_id={"stack-remover"} resize={false} focused={*focused}>
                                <div
                                    data-panel-content={true}
                                    class={classes!("min-h-fit", "min-w-min")}
                                >
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
                    }
                    if panel_manager.stack_table {
                        <Dynamic panel_id={"stack-table"} focused={*focused}>
                            <div
                                data-panel-content={true}
                                class={classes!("min-h-fit", "min-w-min")}
                            >
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
                if *focused {
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
            "flex", "flex-col", "gap-4",
            "overflow-hidden", "flex-1",
            "h-full", "w-full",
            if *focused { "bg-black/25" } else { "bg-transparent" },
        )}>
            {data}
        </div>
    }
}
