//! Pure river-meander math and tests.

/// Compute the horizontal centre of the river at a given vertical position `y`.
///
/// The river is defined over a grid whose top is `river_top` and whose dam row is `dam_y`.
/// `seed` is a per-wave phase offset. The result is in column units (can be fractional).
pub fn river_center(
    y: f32,
    river_top: f32,
    dam_y: f32,
    seed: f32,
    frequency: f32,
    amplitude: f32,
    grid_width: u16,
) -> f32 {
    if dam_y <= river_top {
        return grid_width as f32 / 2.0;
    }
    let normalized = (y - river_top) / (dam_y - river_top);
    let phase = seed + normalized * frequency;
    let half = grid_width as f32 / 2.0;
    half + amplitude * phase.sin()
}

/// Return the integer columns occupied by the river at a given centre and width.
/// The result is clamped to `[0, grid_width)`.
pub fn river_columns(center: f32, width: u16, grid_width: u16) -> Vec<usize> {
    let center_i = center.round() as i32;
    let half = (width as i32) / 2;
    let max_left = (grid_width as i32).saturating_sub(width as i32);
    let left = (center_i - half).clamp(0, max_left) as usize;
    let right = (left + width as usize).min(grid_width as usize);
    (left..right).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn center_stays_within_amplitude() {
        for y in 10..=20 {
            let c = river_center(y as f32, 10.0, 20.0, 0.0, 2.0, 3.0, 40);
            assert!(
                c >= 17.0 && c <= 23.0,
                "centre {} out of bounds at y {}",
                c,
                y
            );
        }
    }

    #[test]
    fn columns_are_within_grid() {
        assert_eq!(river_columns(-2.0, 3, 20), vec![0, 1, 2]);
        assert_eq!(river_columns(21.0, 3, 20), vec![17, 18, 19]);
        assert_eq!(river_columns(10.0, 3, 20), vec![9, 10, 11]);
    }

    #[test]
    fn meander_changes_with_seed() {
        let c1 = river_center(15.0, 10.0, 20.0, 0.0, 2.0, 5.0, 40);
        let c2 = river_center(15.0, 10.0, 20.0, 1.0, 2.0, 5.0, 40);
        assert!((c1 - c2).abs() > 0.1);
    }

    #[test]
    fn zero_length_river_returns_center() {
        let c = river_center(5.0, 10.0, 10.0, 1.0, 2.0, 5.0, 40);
        assert_eq!(c, 20.0);
    }
}
