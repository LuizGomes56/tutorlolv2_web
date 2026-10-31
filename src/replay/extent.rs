//! Compact observed bounds outlive decoded neighbor buffers, never the recording metadata.
use super::cache::ReplaySource;
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};
use tutorlolv2_protocol::ChunkInfo;
use yew::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct ObservedExtent {
    pub metadata: ChunkInfo,
    pub first: f32,
    pub last: f32,
}

#[derive(Default)]
struct ExtentStore {
    values: BTreeMap<u32, ObservedExtent>,
    revision: u64,
}

impl ExtentStore {
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

    fn timeline(&self, chunks: &[ChunkInfo], end: f32) -> (Vec<ChunkInfo>, f32) {
        (chunks.to_vec(), end)
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
