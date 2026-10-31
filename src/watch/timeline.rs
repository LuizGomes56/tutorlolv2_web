use tutorlolv2_protocol::ChunkInfo;

const SAMPLING_GAP_S: f32 = 2.;

/// Coverage is expressed in game seconds, including missing beginnings.
pub fn gaps(chunks: &[ChunkInfo], end: f32) -> Vec<(f32, f32)> {
    let mut ranges: Vec<_> = chunks
        .iter()
        .map(|c| (c.first_game_time, c.last_game_time))
        .collect();

    ranges.sort_by(|a, b| a.0.total_cmp(&b.0));

    let mut cursor = 0.;
    let mut missing = Vec::new();

    for (first, last) in ranges {
        if first > cursor + SAMPLING_GAP_S {
            missing.push((cursor, first));
        }

        cursor = cursor.max(last);
    }

    if end > cursor + SAMPLING_GAP_S {
        missing.push((cursor, end));
    }

    missing
}

/// Select a chunk in chronological order. A normal sampling interval between
/// adjacent chunks holds the preceding frame; missing coverage never does.
pub fn current_index(chunks: &[ChunkInfo], time: f32) -> Option<usize> {
    let current = chunks
        .iter()
        .rposition(|chunk| chunk.first_game_time <= time)?;

    let chunk = &chunks[current];

    if time <= chunk.last_game_time {
        return Some(current);
    }

    let next = chunks.get(current + 1)?;

    (next.first_game_time - chunk.last_game_time <= SAMPLING_GAP_S && time < next.first_game_time)
        .then_some(current)
}

pub fn neighbors(chunks: &[ChunkInfo], time: f32) -> Vec<u32> {
    let Some(current) = current_index(chunks, time) else {
        return Vec::new();
    };

    chunks[current.saturating_sub(1)..=(current + 1).min(chunks.len() - 1)]
        .iter()
        .map(|c| c.seq)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn chunk(seq: u32, first: f32, last: f32) -> ChunkInfo {
        ChunkInfo {
            seq,
            first_game_time: first,
            last_game_time: last,
            snapshot_count: 60,
            frames_format: 1,
        }
    }
    #[test]
    fn missing_beginning_and_internal_gaps_are_visible() {
        assert_eq!(
            gaps(&[chunk(1, 30., 60.), chunk(2, 90., 120.)], 120.),
            vec![(0., 30.), (60., 90.)]
        );
    }
    #[test]
    fn cache_window_is_bounded_at_both_ends() {
        let chunks = [
            chunk(3, 0., 60.),
            chunk(4, 61., 120.),
            chunk(5, 121., 180.),
            chunk(6, 181., 240.),
        ];
        assert_eq!(neighbors(&chunks, 0.), vec![3, 4]);
        assert_eq!(neighbors(&chunks, 90.), vec![3, 4, 5]);
        assert_eq!(neighbors(&chunks, 240.), vec![5, 6]);
        assert!(neighbors(&chunks, 300.).is_empty());
    }
}

#[cfg(test)]
mod sampling_tests {
    use super::*;
    fn chunk(seq: u32, first: f32, last: f32) -> ChunkInfo {
        ChunkInfo {
            seq,
            first_game_time: first,
            last_game_time: last,
            snapshot_count: 60,
            frames_format: 1,
        }
    }
    #[test]
    fn fractional_playback_holds_previous_frame_between_normal_samples() {
        let chunks = [
            chunk(0, 0.3, 59.3),
            chunk(1, 60.3, 119.3),
            chunk(2, 120.3, 179.3),
        ];
        for time in [59.3, 59.55, 59.8, 60.05] {
            assert_eq!(current_index(&chunks, time), Some(0), "time {time}");
            assert_eq!(neighbors(&chunks, time), vec![0, 1]);
        }
        assert_eq!(current_index(&chunks, 60.3), Some(1));
        assert_eq!(neighbors(&chunks, 60.3), vec![0, 1, 2]);
        assert!(gaps(&chunks, 179.3).is_empty());
    }
    #[test]
    fn real_gaps_and_unrecorded_edges_never_hold_a_frame() {
        let chunks = [chunk(0, 30.3, 59.3), chunk(1, 100.3, 159.3)];
        for time in [0., 30.05, 59.55, 60.05, 99.8, 159.55] {
            assert_eq!(current_index(&chunks, time), None, "time {time}");
            assert!(neighbors(&chunks, time).is_empty());
        }
        assert_eq!(current_index(&chunks, 59.3), Some(0));
        assert_eq!(current_index(&chunks, 100.3), Some(1));
        assert_eq!(gaps(&chunks, 159.3), vec![(0., 30.3), (59.3, 100.3)]);
    }
    #[test]
    fn coverage_and_selection_share_the_same_sampling_tolerance() {
        let normal = [chunk(0, 0., 59.3), chunk(1, 61.3, 120.)];
        assert_eq!(current_index(&normal, 60.), Some(0));
        assert!(gaps(&normal, 120.).is_empty());
        let missing = [chunk(0, 0., 59.3), chunk(1, 61.31, 120.)];
        assert_eq!(current_index(&missing, 60.), None);
        assert_eq!(gaps(&missing, 120.).len(), 1);
    }
}
