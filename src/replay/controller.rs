//! One bounded replay controller for capability and native owner sources.
use super::{
    cache::{self, ChunkCache, ReplaySource},
    timeline,
};
use crate::livegame::Game;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use tutorlolv2_protocol::ChunkInfo;
use web_sys::AbortController;
use yew::{platform::spawn_local, prelude::*};

#[derive(Clone, PartialEq)]
pub struct ReplayRequest {
    pub source: ReplaySource,
    pub chunks: Vec<ChunkInfo>,
    pub time: f32,
    pub enabled: bool,
    pub revision: u32,
}

#[hook]
pub fn use_replay_controller(
    request: ReplayRequest,
    cache: Rc<RefCell<ChunkCache>>,
    game: UseStateHandle<Option<Rc<Game>>>,
    message: UseStateHandle<String>,
    on_access_error: Callback<String>,
) {
    use_effect_with(request, move |request| {
        let active = Rc::new(Cell::new(true));
        let running = active.clone();
        let abort = AbortController::new().unwrap();
        let task_abort = abort.clone();
        let request = request.clone();

        spawn_local(async move {
            if !request.enabled {
                return;
            }

            let ranges = cache::observed_ranges(&request.chunks, &cache.borrow());
            cache::retain_neighbors(&mut cache.borrow_mut(), &ranges, request.time);

            let Some(index) = timeline::current_index(&ranges, request.time) else {
                game.set(None);
                message.set(
                    "No recording at this time. Choose a covered part of the timeline.".into(),
                );
                return;
            };

            let current = request.chunks[index].seq;
            let needed = timeline::neighbors(&ranges, request.time);
            let mut order = vec![current];

            order.extend(needed.into_iter().filter(|seq| *seq != current));
            message.set("Loading replay…".into());

            for seq in order {
                let metadata = request
                    .chunks
                    .iter()
                    .find(|chunk| chunk.seq == seq)
                    .unwrap();

                let cached = cache
                    .borrow()
                    .get(&seq)
                    .filter(|chunk| chunk.validate(metadata).is_ok())
                    .cloned();

                let result = if let Some(chunk) = cached {
                    Ok(chunk)
                } else {
                    request.source.load(metadata, &task_abort).await
                };

                if !running.get() {
                    return;
                }

                match result {
                    Ok(chunk) => {
                        cache.borrow_mut().insert(seq, chunk.clone());

                        if seq != current {
                            continue;
                        }

                        let frames = &chunk.borrow_dependent().frames;
                        let times = frames
                            .iter()
                            .map(|(_, frame)| frame.game_data.game_time)
                            .collect::<Vec<_>>();

                        let next = ranges.get(index + 1).map(|chunk| chunk.first_game_time);
                        let Some((_, frame)) = cache::selected_index(&times, request.time, next)
                            .and_then(|index| frames.get(index))
                        else {
                            game.set(None);
                            message.set(
                                "No recording at this time. Choose a covered part of the timeline."
                                    .into(),
                            );
                            continue;
                        };

                        match crate::utils::transport::realtime_from(frame).await {
                            Ok(value) if running.get() => {
                                game.set(Some(Rc::new(value)));
                                message.set(String::new());
                            }
                            Err(error) if running.get() => {
                                game.set(None);
                                message.set(error.to_string());
                            }
                            _ => return,
                        }
                    }
                    Err(error) => {
                        if error.starts_with("This match") || error.starts_with("Code not found") {
                            on_access_error.emit(error);
                            return;
                        }

                        if seq == current {
                            game.set(None);
                            message.set(error);
                        }
                    }
                }
            }
        });

        move || {
            active.set(false);
            abort.abort();
        }
    });
}
