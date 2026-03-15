use crate::{
    components::{
        dynamic::Dynamic,
        errorlog::errorlog,
        image::{Image, ImageType},
        stack::StackSelector,
        tables::{empty::EmptyTable, header::TableHeader},
    },
    livegame::{Enemy, Game},
    utils::{Loading, Print, encode_offset, glue::get_data, hooks::on_keydown},
};
use std::time::Duration;
use tutorlolv2_gen::CastId;
use wasm_bindgen::{
    JsCast,
    prelude::{Closure, wasm_bindgen},
};
use web_sys::js_sys::Function;
use yew::{
    platform::{spawn_local, time::sleep},
    prelude::*,
};

#[wasm_bindgen(module = "/public/events.js")]
unsafe extern "C" {
    #[wasm_bindgen(js_name = "mouse_events")]
    pub fn mouse_events();

}

#[wasm_bindgen(module = "/public/invoke.js")]
unsafe extern "C" {
    #[wasm_bindgen(js_name = "blur_overlay")]
    pub fn blur_overlay(callback: &Function);
}

#[component]
pub fn Overlay() -> Html {
    let game_data = use_state(|| Err::<Game, _>(Loading.into()));
    let enemy_index = use_state(|| 0);
    let enemy_count = use_state(|| 0);
    let focused = use_state(|| false);

    use_effect_with((), |_| mouse_events());

    {
        let focused = focused.clone();
        use_effect_with((), move |_| {
            let callback = Closure::wrap(Box::new(move || focused.set(false)) as Box<dyn FnMut()>);
            blur_overlay(callback.as_ref().unchecked_ref());
            callback.forget();
        });
    }

    {
        use_effect_with(
            (enemy_index.clone(), enemy_count.clone()),
            move |(enemy_index, enemy_count)| {
                let enemy_index = enemy_index.clone();
                let enemy_count = enemy_count.clone();
                on_keydown(186, move || {
                    let new = (*enemy_index + 1) % *enemy_count;
                    enemy_index.set(new);
                })
            },
        );
    }

    {
        let game_data = game_data.clone();
        let enemy_count = enemy_count.clone();
        use_effect_with((), |_| {
            spawn_local(async move {
                loop {
                    let data = get_data().await;

                    if let Ok(ref game) = data {
                        enemy_count.set(game.enemies.len());
                    }

                    game_data.set(data);
                    sleep(Duration::from_millis(1000)).await;
                }
            });
        });
    };

    let data = match &*game_data {
        Ok(data) => {
            let Game {
                current_player,
                enemies,
                scoreboard,
                items_meta,
                runes_meta,
                game_time,
                ability_levels,
                dragons,
            } = data;

            let damages = enemies
                .get(*enemy_index)
                .or_else(|| {
                    enemies
                        .iter()
                        .find(|enemy| enemy.position == current_player.position)
                })
                .or_else(|| enemies.first())
                .map(|enemy| {
                    let damages =
                        enemy
                            .damages
                            .to_html(current_player.champion_id, items_meta, runes_meta);
                    let enemy_id = enemy.champion_id;
                    html! {
                        <tr>
                            <td
                                class={classes!("w-12")}
                                data_offset={encode_offset(&[enemy_id.formula()])}
                            >
                                <Image src={ImageType::from(enemy_id)} />
                            </td>
                            {damages}
                        </tr>
                    }
                })
                .unwrap_or_default();

            html! {
                <Dynamic panel_id={"damage-table"} focused={*focused}>
                    <div
                        data-panel-content={true}
                        class={classes!("overflow-auto", "w-fit", "origin-top-left")}
                    >
                        <table class={classes!("data-table", "overlay")}>
                            <TableHeader
                                champion_id={current_player.champion_id}
                                items_meta={items_meta.clone()}
                                runes_meta={runes_meta.clone()}
                            />
                            <tbody>{damages}</tbody>
                        </table>
                    </div>
                </Dynamic>
            }
        }
        Err(e) => {
            e.log();
            Default::default()
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
