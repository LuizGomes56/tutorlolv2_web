//! Shared replay buffer and source boundary. Native frames are already decompressed.
use crate::utils::BINCODE_CONFIG;
use std::{collections::BTreeMap, rc::Rc};
use tutorlolv2::riot::FrameChunk;
use tutorlolv2_protocol::{ChunkInfo, FRAMES_FORMAT};

self_cell::self_cell! {
    pub struct CachedChunk {
        owner: Vec<u8>,
        #[covariant]
        dependent: FrameChunk,
    }
}

impl CachedChunk {
    pub fn decode(bytes: Vec<u8>, format: u16) -> Result<Rc<Self>, String> {
        Self::try_new(bytes, |bytes| {
            let (mut chunk, consumed): (FrameChunk<'_>, _) =
                bincode::borrow_decode_from_slice(bytes, BINCODE_CONFIG)
                    .map_err(|e| e.to_string())?;

            if format != FRAMES_FORMAT || consumed != bytes.len() || chunk.format != format {
                return Err("Unsupported or damaged replay chunk. Try refreshing.".into());
            }

            // The accepted recorder contract permits <=2-second game-time jitter.
            // Normalize selection order while preserving every captured sample.
            chunk.frames.sort_by(|a, b| {
                a.1.game_data
                    .game_time
                    .total_cmp(&b.1.game_data.game_time)
                    .then(a.0.cmp(&b.0))
            });

            Ok(chunk)
        })
        .map(Rc::new)
    }

    pub fn validate(&self, metadata: &ChunkInfo) -> Result<(), String> {
        let frames = &self.borrow_dependent().frames;

        if metadata.frames_format != FRAMES_FORMAT
            || self.borrow_dependent().format != metadata.frames_format
            || frames.len() != usize::from(metadata.snapshot_count)
            || frames
                .iter()
                .any(|(_, f)| !f.game_data.game_time.is_finite())
        {
            return Err("Invalid replay chunk metadata.".into());
        }

        Ok(())
    }
}

pub type ChunkCache = BTreeMap<u32, Rc<CachedChunk>>;

pub fn retain_neighbors(cache: &mut ChunkCache, chunks: &[ChunkInfo], time: f32) {
    let needed = super::timeline::neighbors(chunks, time);
    cache.retain(|seq, _| needed.contains(seq));
}

/// Metadata records capture endpoints; accepted jitter can extend decoded coverage.
pub fn observed_ranges(chunks: &[ChunkInfo], cache: &ChunkCache) -> Vec<ChunkInfo> {
    chunks
        .iter()
        .map(|metadata| {
            let mut range = metadata.clone();

            range.first_game_time = metadata.first_game_time.min(metadata.last_game_time);
            range.last_game_time = metadata.first_game_time.max(metadata.last_game_time);

            if let Some(chunk) = cache
                .get(&metadata.seq)
                .filter(|chunk| chunk.validate(metadata).is_ok())
            {
                let frames = &chunk.borrow_dependent().frames;

                if let Some((_, frame)) = frames.first() {
                    range.first_game_time = range.first_game_time.min(frame.game_data.game_time);
                }

                if let Some((_, frame)) = frames.last() {
                    range.last_game_time = range.last_game_time.max(frame.game_data.game_time);
                }
            }

            range
        })
        .collect()
}

/// Return sample-and-hold only when the next captured sample is adjacent.
pub fn sample_index(times: &[f32], time: f32) -> Option<usize> {
    let index = times.iter().rposition(|value| *value <= time)?;

    if times[index] == time {
        return Some(index);
    }

    let next = *times.get(index + 1)?;
    (next - times[index] <= 2. && time < next).then_some(index)
}

pub fn selected_index(times: &[f32], time: f32, next_chunk: Option<f32>) -> Option<usize> {
    sample_index(times, time).or_else(|| {
        let index = times.len().checked_sub(1)?;
        let last = times[index];

        (time >= last && next_chunk.is_some_and(|next| next - last <= 2.0 && time < next))
            .then_some(index)
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReplaySource {
    Capability(String),
    #[cfg(feature = "tauri")]
    Owner(String),
}

impl ReplaySource {
    pub async fn load(
        &self,
        metadata: &ChunkInfo,
        abort: &web_sys::AbortController,
    ) -> Result<Rc<CachedChunk>, String> {
        let chunk = match self {
            Self::Capability(code) => {
                let response =
                    crate::watch::api::get(&format!("{code}/frames/{}", metadata.seq), abort)
                        .await?;

                if !response.ok() {
                    return Err(crate::watch::api::status_error(response.status()));
                }

                CachedChunk::decode(
                    crate::watch::api::bytes(&response).await?,
                    metadata.frames_format,
                )?
            }

            #[cfg(feature = "tauri")]
            Self::Owner(id) => {
                use crate::history::source::{HistorySource, IpcHistory};

                CachedChunk::decode(
                    IpcHistory::frames(id, metadata.seq)
                        .await
                        .map_err(|e| e.message)?,
                    metadata.frames_format,
                )?
            }
        };

        chunk.validate(metadata)?;
        Ok(chunk)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sample_hold_requires_a_known_adjacent_sample() {
        assert_eq!(sample_index(&[1., 2., 10.], 1.5), Some(0));
        assert_eq!(sample_index(&[1., 2., 10.], 3.), None);
        assert_eq!(sample_index(&[1., 2.], 2.5), None);
        assert_eq!(sample_index(&[1., 2.], 0.), None);
    }
    #[test]
    fn accepted_capture_jitter_is_normalized_not_rejected() {
        let bytes = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/example.bincode"));
        let (frame, _): (tutorlolv2::riot::RiotRealtime<'_>, _) =
            bincode::borrow_decode_from_slice(bytes, BINCODE_CONFIG).unwrap();
        let frames = [100., 102., 101., 100., 99.]
            .into_iter()
            .enumerate()
            .map(|(index, time)| {
                let mut frame = frame.clone();
                frame.game_data.game_time = time;
                (index as i64, frame)
            })
            .collect();
        let chunk = FrameChunk {
            format: FRAMES_FORMAT,
            frames,
        };
        let cached = CachedChunk::decode(
            bincode::encode_to_vec(chunk, BINCODE_CONFIG).unwrap(),
            FRAMES_FORMAT,
        )
        .unwrap();
        cached
            .validate(&ChunkInfo {
                seq: 0,
                first_game_time: 100.,
                last_game_time: 99.,
                snapshot_count: 5,
                frames_format: FRAMES_FORMAT,
            })
            .unwrap();
        let times = cached
            .borrow_dependent()
            .frames
            .iter()
            .map(|(_, frame)| frame.game_data.game_time)
            .collect::<Vec<_>>();
        assert_eq!(times, vec![99., 100., 100., 101., 102.]);
        assert_eq!(selected_index(&times, 100.75, None), Some(2));
        let metadata = ChunkInfo {
            seq: 0,
            first_game_time: 100.,
            last_game_time: 99.,
            snapshot_count: 5,
            frames_format: FRAMES_FORMAT,
        };
        let mut cache = ChunkCache::new();
        cache.insert(0, cached);
        let ranges = observed_ranges(&[metadata], &cache);
        assert_eq!(
            super::super::timeline::current_index(&ranges, 102.),
            Some(0)
        );
    }

    #[cfg(feature = "tauri")]
    #[test]
    fn sources_do_not_share_identity() {
        assert_ne!(
            ReplaySource::Owner("same".into()),
            ReplaySource::Capability("same".into())
        );
    }
}
