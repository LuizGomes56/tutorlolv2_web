//! Compact observed bounds outlive decoded neighbor buffers, never the recording metadata.
use std::collections::BTreeMap;
use tutorlolv2_protocol::ChunkInfo;

#[derive(Clone, Debug, PartialEq)]
pub struct ObservedExtent {
    pub metadata: ChunkInfo,
    pub first: f32,
    pub last: f32,
}

#[derive(Default)]
pub(crate) struct ExtentStore {
    values: BTreeMap<u32, ObservedExtent>,
    revision: u64,
}

impl ExtentStore {
    pub(crate) fn matches(&self, metadata: &ChunkInfo) -> bool {
        self.values
            .get(&metadata.seq)
            .is_some_and(|extent| extent.metadata == *metadata)
    }

    pub(crate) fn observe_chunk(
        &mut self,
        metadata: &ChunkInfo,
        chunk: &super::cache::CachedChunk,
    ) -> bool {
        if chunk.validate(metadata).is_err() {
            return false;
        }
        let frames = &chunk.borrow_dependent().frames;
        let (Some((_, first)), Some((_, last))) = (frames.first(), frames.last()) else {
            return false;
        };
        self.observe(ObservedExtent {
            metadata: metadata.clone(),
            first: first.game_data.game_time,
            last: last.game_data.game_time,
        })
    }

    fn observe(&mut self, value: ObservedExtent) -> bool {
        if !value.first.is_finite()
            || !value.last.is_finite()
            || value.first > value.last
            || self.values.get(&value.metadata.seq) == Some(&value)
        {
            return false;
        }

        self.values.insert(value.metadata.seq, value);
        self.revision = self.revision.wrapping_add(1);

        true
    }

    pub(crate) fn timeline(&self, chunks: &[ChunkInfo], end: f32) -> (Vec<ChunkInfo>, f32) {
        let ranges: Vec<_> = chunks
            .iter()
            .map(|metadata| {
                let mut range = metadata.clone();
                range.first_game_time = metadata.first_game_time.min(metadata.last_game_time);
                range.last_game_time = metadata.first_game_time.max(metadata.last_game_time);

                if let Some(observed) = self
                    .values
                    .get(&metadata.seq)
                    .filter(|observed| observed.metadata == *metadata)
                {
                    range.first_game_time = range.first_game_time.min(observed.first);
                    range.last_game_time = range.last_game_time.max(observed.last);
                }

                range
            })
            .collect();
        let end = ranges
            .iter()
            .fold(end, |end, range| end.max(range.last_game_time));

        (ranges, end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn chunk(seq: u32, first: f32, last: f32) -> ChunkInfo {
        ChunkInfo {
            seq,
            first_game_time: first,
            last_game_time: last,
            snapshot_count: 5,
            frames_format: 1,
        }
    }
    #[test]
    fn loaded_jitter_extends_slider_end_and_removes_false_hatching() {
        let metadata = chunk(0, 100., 99.);
        let mut store = ExtentStore::default();
        store.observe(ObservedExtent {
            metadata: metadata.clone(),
            first: 99.,
            last: 102.,
        });
        let (ranges, end) = store.timeline(&[metadata], 100.);
        assert_eq!(end, 102.);
        assert_eq!(
            (ranges[0].first_game_time, ranges[0].last_game_time),
            (99., 102.)
        );
        assert_eq!(super::super::timeline::gaps(&ranges, end), vec![(0., 99.)]);
        assert_eq!(
            super::super::timeline::current_index(&ranges, 102.),
            Some(0)
        );
    }
    #[test]
    fn observed_extents_preserve_real_gaps_and_ignore_changed_metadata() {
        let metadata = chunk(0, 100., 99.);
        let mut store = ExtentStore::default();
        store.observe(ObservedExtent {
            metadata: metadata.clone(),
            first: 99.,
            last: 102.,
        });
        let (ranges, end) = store.timeline(&[metadata.clone(), chunk(1, 200., 210.)], 210.);
        assert_eq!(
            super::super::timeline::gaps(&ranges, end),
            vec![(0., 99.), (102., 200.)]
        );
        let mut changed = metadata;
        changed.snapshot_count = 6;
        let (_, end) = store.timeline(&[changed], 100.);
        assert_eq!(end, 100.);
    }
}

/// Scope compact observations to one source, independently of decoded neighbor eviction.
#[yew::hook]
pub(crate) fn use_extents(
    source: super::cache::ReplaySource,
) -> (
    std::rc::Rc<std::cell::RefCell<ExtentStore>>,
    yew::Callback<()>,
) {
    use yew::prelude::*;
    let revision = use_state(|| 0u64);
    let store = use_memo(source, |_| std::cell::RefCell::new(ExtentStore::default()));
    let changed = Callback::from(move |()| revision.set(revision.wrapping_add(1)));
    (store, changed)
}
