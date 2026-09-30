//! Pure dam coverage logic and tests.

/// Returns true if every column in `river_columns` is covered by at least one
/// dam chunk. A chunk at slot `s` covers columns `[s*chunk_width, (s+1)*chunk_width)`.
pub fn covers(chunks: &[bool], chunk_width: usize, river_columns: &[usize]) -> bool {
    river_columns.iter().all(|&col| {
        let slot = col / chunk_width;
        chunks.get(slot).copied().unwrap_or(false)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_chunk_covers_river() {
        // 10 slots, chunks at slot 4 (columns 16-19) and 5 (20-23).
        let mut chunks = vec![false; 10];
        chunks[4] = true;
        chunks[5] = true;
        assert!(covers(&chunks, 4, &[18, 19, 20]));
    }

    #[test]
    fn missing_chunk_is_leak() {
        let mut chunks = vec![false; 10];
        chunks[2] = true;
        assert!(!covers(&chunks, 4, &[18, 19, 20]));
    }

    #[test]
    fn empty_river_is_covered() {
        let chunks = vec![false; 10];
        assert!(covers(&chunks, 4, &[]));
    }

    #[test]
    fn out_of_bounds_column_is_not_covered() {
        let chunks = vec![true; 3];
        assert!(!covers(&chunks, 4, &[20]));
    }
}
