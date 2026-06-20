use {
    crate::{
        livegame::{CurrentPlayer, Enemy, Game, Scoreboard},
        utils::traits::Print,
    },
    tutorlolv2::realtime::RealtimeError,
    wasm_bindgen::{JsValue, prelude::wasm_bindgen},
    web_sys::js_sys::Uint8Array,
};

#[wasm_bindgen(module = "/public/invoke.js")]
unsafe extern "C" {
    #[wasm_bindgen(js_name = "invoke_get_live_game", catch)]
    pub async fn get_live_game() -> Result<Uint8Array, JsValue>;
}

async fn realtime(bytes: Vec<u8>) -> Result<Game, Box<dyn core::error::Error>> {
    match bytes.is_empty() {
        true => Err("Desktop application required to use the overlay feature".into()),
        false => {
            let game = serde_json::from_slice(bytes.as_slice())?;
            let data = tutorlolv2::realtime(&game).map_err(|e| match e {
                RealtimeError::UnrecognizedCurrentPlayer(p) => {
                    format!("Unable to recognize current player with name {p:?}")
                }
            })?;

            Ok(Game {
                current_player: CurrentPlayer {
                    riot_id: data.current_player.riot_id.into(),
                    base_stats: data.current_player.base_stats,
                    bonus_stats: data.current_player.bonus_stats,
                    current_stats: data.current_player.current_stats,
                    level: data.current_player.level,
                    team: data.current_player.team,
                    adaptive_type: data.current_player.adaptive_type,
                    position: data.current_player.position,
                    champion_id: data.current_player.champion_id,
                    game_map: data.current_player.game_map,
                },
                enemies: data
                    .enemies
                    .into_iter()
                    .map(|enemy| Enemy {
                        riot_id: enemy.riot_id.into(),
                        damages: enemy.damages,
                        siml_items: enemy.siml_items,
                        base_stats: enemy.base_stats,
                        bonus_stats: enemy.bonus_stats,
                        current_stats: enemy.current_stats,
                        real_armor: enemy.real_armor,
                        real_magic_resist: enemy.real_magic_resist,
                        level: enemy.level,
                        champion_id: enemy.champion_id,
                        team: enemy.team,
                        position: enemy.position,
                    })
                    .collect(),
                scoreboard: data
                    .scoreboard
                    .into_iter()
                    .map(|score| Scoreboard {
                        riot_id: score.riot_id.into(),
                        assists: score.assists,
                        creep_score: score.creep_score,
                        deaths: score.deaths,
                        kills: score.kills,
                        champion_id: score.champion_id,
                        position: score.position,
                        team: score.team,
                    })
                    .collect(),
                items_meta: data.items_meta.into(),
                runes_meta: data.runes_meta.into(),
                game_time: data.game_time,
                ability_levels: data.ability_levels,
                dragons: data.dragons,
            })
        }
    }
}

pub async fn get_data() -> Result<Game, Box<dyn core::error::Error>> {
    let bytes = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/example.json")).to_vec();
    return realtime(bytes).await;

    let bytes = get_live_game().await;

    "Called get_data() function".log();

    match bytes {
        Ok(response) => {
            "ok".log();
            let bytes = response.to_vec();
            realtime(bytes).await
        }
        Err(e) => {
            format!("[tauri] Error: {e:?}").log();
            Err("[tauri]: Can't access Riot API or you're not playing League right now".into())
        }
    }
}
